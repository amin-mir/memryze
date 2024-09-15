use std::error::Error;
use std::process;

use clap::{Parser, Subcommand};
use tokio::net::TcpStream;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use message::{Message, QA};
use prot;

#[derive(Debug, Parser)]
struct Args {
    #[arg(
        long,
        help = "Authentication token",
        default_value = "17bd4593ade7a97020ae0de719099f5b13fbf1b17a21f1882e4063f0e660e020"
    )]
    token: String,

    #[arg(
        long,
        help = "Postgres connection address",
        default_value = "127.0.0.1:8080"
    )]
    pg_addr: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    InsertQA {
        #[arg(help = "Question")]
        q: String,
        #[arg(help = "Answer")]
        a: String,
    },
    GetQuiz,
    CorrectReview {
        #[arg(help = "ID of qa")]
        id: i64,
    },
    WrongReview {
        #[arg(help = "ID of qa")]
        id: i64,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .init();

    let mut stream = TcpStream::connect(args.pg_addr).await?;

    let mut in_buf = vec![0u8; 512];
    let mut prim_out_buf = vec![0u8; 512];

    let handshake = Message::Handshake {
        version: 1,
        token: &args.token,
    };

    prot::write_msg(&mut stream, &mut prim_out_buf, &handshake).await?;

    let handshake_reply = prot::read_msg(&mut stream, &mut in_buf).await?;

    let Message::HandshakeResp = handshake_reply else {
        error!(?handshake_reply, "Handshake reply has the wrong type");
        process::exit(1);
    };

    info!("Received handshake from server");

    let mut qas: Vec<QA> = Vec::with_capacity(10);

    let msg = match args.command {
        Commands::InsertQA { ref q, ref a } => Message::AddQA { q, a },
        Commands::GetQuiz => Message::GetQuiz,
        Commands::CorrectReview { id } => Message::ReviewQA { id, correct: true },
        Commands::WrongReview { id } => Message::ReviewQA { id, correct: false },
    };

    prot::write_msg(&mut stream, &mut prim_out_buf, &msg).await?;

    let resp = prot::read_msg(&mut stream, &mut in_buf).await?;
    match resp {
        Message::AddQAResp => {
            info!(?resp, "AddQA successul");
        }
        Message::Quiz { count, qas_bytes } => {
            prot::deser_from_bytes(qas_bytes, count, &mut qas)?;
            info!(?qas, "Quiz");
        }
        Message::ReviewQAResp => {
            info!(?resp, "ReviewQA successful");
        }
        Message::InternalError => {
            error!("Internal server error");
        }
        _ => panic!("Invalid response from server"),
    }

    Ok(())
}
