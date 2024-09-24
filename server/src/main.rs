use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use clap::Parser;
use tokio::net::{TcpListener, TcpStream};
use tokio_postgres::NoTls;
use tracing::{debug, error, info, info_span, Instrument};
use tracing_subscriber::EnvFilter;

use memryze::db::PgClient;
use message::{Message, QA};
use prot;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(short, long, default_value = "127.0.0.1:8080")]
    addr: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pg_uri = env::var("POSTGRES_URI")
        .unwrap_or("postgres://postgres:pswd@localhost:5432/memryze".to_owned());

    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("tokio_postgres=error".parse()?),
        )
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .init();

    let (pg_client, pg_conn) = tokio_postgres::connect(&pg_uri, NoTls).await?;

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
                match handle(stream, addr, pg_client).await {
                    Ok(()) => (),
                    Err(err @ prot::Error::StreamClosed) => debug!(%err),
                    Err(err) => error!(?err),
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
) -> prot::Result<()> {
    let mut in_buf = vec![0u8; 512];
    let mut prim_out_buf = vec![0u8; 512];
    let mut sec_out_buf = vec![0u8; 2048];

    let first_msg = prot::read_msg(&mut stream, &mut in_buf).await?;

    let Message::Handshake { version, token } = first_msg else {
        let err = anyhow::anyhow!("First message was not handshake");
        return Err(prot::Error::Other(err));
    };

    let customer_id = pg_client
        .customer_id_from_token(token)
        .await
        .context("Fetching customer id from token")?;

    info!(version, customer_id, "Client handshake received");

    prot::write_msg(&mut stream, &mut prim_out_buf, &Message::HandshakeResp).await?;

    let mut qas: Vec<QA> = vec![QA::default(); 20];
    loop {
        let msg = prot::read_msg(&mut stream, &mut in_buf).await?;

        match msg {
            Message::AddQA { q, a } => match pg_client.insert_qa(customer_id, &q, &a).await {
                Ok(_) => {
                    prot::write_msg(&mut stream, &mut prim_out_buf, &Message::AddQAResp).await?
                }
                Err(err) => {
                    error!(?err, "Error inserting QA");
                    prot::write_msg(&mut stream, &mut prim_out_buf, &Message::InternalError)
                        .await?;
                }
            },
            Message::GetQuiz => {
                match pg_client.get_quiz(customer_id, &mut qas).await {
                    Ok(n) => {
                        // If n = 0 the payload will be `[0x05, 0x00, 0x00]` and the client
                        // will receive qas as an empty slice of bytes.
                        debug!(count = n, qas = ?&qas[0..n], "fetched qas from db");
                        let qas_bytes = prot::ser_slice(&qas[0..n], &mut sec_out_buf)?;
                        prot::write_msg(
                            &mut stream,
                            &mut prim_out_buf,
                            &Message::Quiz {
                                count: n as u16,
                                qas_bytes,
                            },
                        )
                        .await?;
                    }
                    Err(err) => {
                        error!(?err, "Error fetching a quiz");
                        prot::write_msg(&mut stream, &mut prim_out_buf, &Message::InternalError)
                            .await?;
                    }
                };
            }
            Message::ReviewQA { id, correct } => match pg_client.review_qa(id, correct).await {
                Err(err) => {
                    error!(?err, "Error reviewing QA");
                    prot::write_msg(&mut stream, &mut prim_out_buf, &Message::InternalError)
                        .await?;
                }
                Ok(()) => {
                    prot::write_msg(&mut stream, &mut prim_out_buf, &Message::ReviewQAResp).await?;
                }
            },
            msg => {
                let err = anyhow::anyhow!("Client sent wrong message: {:?}", msg);
                return Err(prot::Error::Other(err));
            }
        }
    }
}
