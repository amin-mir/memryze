use std::env;

use rand::rngs::OsRng;
use rand::Rng;
use tokio_postgres::NoTls;
use tracing::error;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pg_uri = env::args()
        .nth(1)
        .unwrap_or("postgres://postgres:pswd@localhost:5432/memryze".to_string());

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

    let token = generate_token();

    let row = pg_client
        .query_one(
            "INSERT INTO customer (token) VALUES ($1) RETURNING id",
            &[&token],
        )
        .await?;
    let id: i64 = row.get("id");

    println!("Inserted (customer, token): ({}, {})", id, token);

    Ok(())
}

fn generate_token() -> String {
    let token: [u8; 32] = OsRng.gen();
    hex_encode(&token)
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX_CHARS: &[u8] = b"0123456789ABCDEF";
    let mut hex_string = String::with_capacity(bytes.len() * 2);

    for &byte in bytes {
        hex_string.push(HEX_CHARS[(byte >> 4) as usize] as char);
        hex_string.push(HEX_CHARS[(byte & 0x0f) as usize] as char);
    }

    hex_string
}
