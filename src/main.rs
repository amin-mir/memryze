use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use clap::Parser;
use tokio::net::{TcpListener, TcpStream};
use tokio_postgres::{Client, NoTls};

use memryze::{Message, Protocol};

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(short, long, default_value = "127.0.0.1:8080")]
    addr: String,

    #[arg(
        short,
        long,
        default_value = "postgres://postgres:pswd@localhost:5432/memryze"
    )]
    pg_uri: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let (pg_client, pg_conn) = tokio_postgres::connect(&args.pg_uri, NoTls).await?;
    let pg_client = Arc::new(pg_client);

    tokio::spawn(async move {
        if let Err(e) = pg_conn.await {
            eprintln!("connection error: {}", e);
        }
    });

    let listener = TcpListener::bind(args.addr).await?;

    loop {
        let (stream, addr) = listener.accept().await?;

        let pg_client = pg_client.clone();
        tokio::spawn(async move {
            if let Err(err) = handle(stream, addr, pg_client).await {
                println!("Error handling the connection: {}", err);
            }
        });
    }
}

async fn handle(
    mut stream: TcpStream,
    addr: SocketAddr,
    pg_client: Arc<Client>,
) -> anyhow::Result<()> {
    println!("handling peer {addr}");

    let mut prot = Protocol::new(2048);

    let first_msg = prot
        .read_msg(&mut stream)
        .await
        .context("error reading the first message")?;

    let Message::Handshake { version } = first_msg else {
        anyhow::bail!("First message was not handshake");
    };

    println!("client handshake version: {version}");

    let handshake_reply = Message::Handshake { version };
    prot.write_msg(&mut stream, &handshake_reply).await?;

    // TODO: wrap pg_client in a struct that prepares all the statements at
    // the beginning so that we don't have to prepare separatly in each client.
    let insert_qa_stmt = pg_client
        .prepare("INSERT INTO qa (q, a) VALUES ($1, $2)")
        .await?;

    loop {
        let msg = prot.read_msg(&mut stream).await?;

        match msg {
            Message::AddQA { q, a } => {
                // TODO: return internal server error if the query fails.
                pg_client.execute(&insert_qa_stmt, &[&q, &a]).await?;
                prot.write_msg(&mut stream, &Message::AddQAResp).await?;
            }
            msg => {
                anyhow::bail!("client sent wrong message: {:?}", msg);
            }
        }
    }
}
