use std::env;
use std::error::Error;

use tokio_postgres::NoTls;

const CREATE_TABLES_QUERY: &str = r#"
CREATE TABLE IF NOT EXISTS customer (
    id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    token text NOT NULL UNIQUE
);

CREATE INDEX IF NOT EXISTS idx_user_token ON customer (token);

CREATE TABLE IF NOT EXISTS qa (
    id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    q TEXT NOT NULL UNIQUE,
    a TEXT NOT NULL,
    customer_id BIGINT NOT NULL REFERENCES customer (id),
    max INTEGER NOT NULL DEFAULT 3,
    correct_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_shown_at TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_qa_created_at ON qa (created_at);
CREATE INDEX IF NOT EXISTS idx_qa_correct_count_last_shown_at_created_at
    ON qa (customer_id, correct_count, last_shown_at, created_at);
"#;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let pg_uri = env::var("POSTGRES_URI")
        .unwrap_or("postgres://postgres:pswd@localhost:5432/memryze".to_owned());

    let (client, connection) = tokio_postgres::connect(&pg_uri, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    client.simple_query(CREATE_TABLES_QUERY).await?;

    let insert_qa_stmt = client
        .prepare("INSERT INTO qa (q, a, customer_id) VALUES ($1, $2, $3)")
        .await?;

    // TODO: insert a customer before this otherwise this will fail.

    let res = client
        .execute(
            &insert_qa_stmt,
            &[&"is this place free?", &"onks tä paikka vapaa?", &1i64],
        )
        .await;

    if let Err(err) = res {
        // err.code();
        // err.as_db_error();
        println!("Inserting qa failed: {err:?}");
    }

    Ok(())
}
