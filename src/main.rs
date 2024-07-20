use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use clap::Parser;
use tokio::net::{TcpListener, TcpStream};
use tokio_postgres::NoTls;
use tracing::{error, info, info_span, Instrument};
use tracing_subscriber::EnvFilter;

use memryze::db::PgClient;
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

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("tokio_postgres=error".parse()?),
        )
        .with_target(true)
        .init();

    let (pg_client, pg_conn) = tokio_postgres::connect(&args.pg_uri, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = pg_conn.await {
            error!("connection error: {}", e);
        }
    });

    let pg_client = PgClient::prepare(pg_client).await?;
    let pg_client = Arc::new(pg_client);

    let listener = TcpListener::bind(args.addr).await?;

    loop {
        let (stream, addr) = listener.accept().await?;

        let pg_client = pg_client.clone();
        tokio::spawn(
            async move {
                if let Err(err) = handle(stream, addr, pg_client).await {
                    error!(?err);
                }
            }
            .instrument(info_span!("handle", %addr)),
        );
    }
}

async fn handle(
    mut stream: TcpStream,
    _addr: SocketAddr,
    pg_client: Arc<PgClient>,
) -> anyhow::Result<()> {
    info!("handling peer");

    let mut prot = Protocol::new(2048);

    let first_msg = prot
        .read_msg(&mut stream)
        .await
        .context("Error reading the first message")?;

    let Message::Handshake { version } = first_msg else {
        anyhow::bail!("First message was not handshake");
    };

    info!(version, "Client handshake received");

    let handshake_reply = Message::Handshake { version };
    prot.write_msg(&mut stream, &handshake_reply).await?;

    loop {
        let msg = prot.read_msg(&mut stream).await?;

        match msg {
            Message::AddQA { q, a } => {
                if let Err(err) = pg_client.insert_qa(&q, &a).await {
                    error!(?err, "Error inserting QA");
                    prot.write_msg(&mut stream, &Message::InternalError).await?;
                }
                prot.write_msg(&mut stream, &Message::AddQAResp).await?;
            }
            msg => {
                anyhow::bail!("Client sent wrong message: {:?}", msg);
            }
        }
    }
}
