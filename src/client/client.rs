use std::error::Error;
use std::process;

use clap::{Parser, Subcommand};
use tokio::net::TcpStream;
// use tokio::time::{self, Duration};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use memryze::{Message, Protocol};

const ADDR: &str = "127.0.0.1:8080";

#[derive(Debug, Parser)]
struct Args {
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let mut stream = TcpStream::connect(ADDR).await?;

    let mut prot = Protocol::new(2048);

    let handshake = Message::Handshake { version: 1 };

    prot.write_msg(&mut stream, &handshake).await?;

    let handshake_reply = prot.read_msg(&mut stream).await?;

    let Message::Handshake { version } = handshake_reply else {
        error!(?handshake_reply, "Handshake reply has the wrong type");
        process::exit(1);
    };

    info!(version, "Received handshake from server");

    let msg = match args.command {
        Commands::InsertQA { ref q, ref a } => Message::AddQA { q, a },
        _ => panic!("Unsupported command"),
    };

    prot.write_msg(&mut stream, &msg).await?;

    let add_qa_reply = prot.read_msg(&mut stream).await?;

    info!(reply = ?add_qa_reply, "AddQA reply");

    Ok(())
}
