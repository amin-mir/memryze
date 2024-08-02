use std::error::Error;

use tokio_postgres::NoTls;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (client, connection) =
        tokio_postgres::connect("postgres://postgres:pswd@localhost:5432/memryze", NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    client
        .simple_query(
            "CREATE TABLE IF NOT EXISTS qa (
                id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
                q TEXT NOT NULL UNIQUE,
                a TEXT NOT NULL,
                max INTEGER NOT NULL DEFAULT 3,
                correct_count INTEGER NOT NULL DEFAULT 0,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                last_shown_at TIMESTAMP
            );
            CREATE INDEX IF NOT EXISTS idx_qa_created_at ON qa (created_at);
            CREATE INDEX IF NOT EXISTS idx_qa_correct_count_last_shown_at_created_at
            ON qa (correct_count, last_shown_at, created_at);",
        )
        .await?;

    let insert_qa_stmt = client
        .prepare("INSERT INTO qa (q, a) VALUES ($1, $2)")
        .await?;

    let err = client
        .execute(
            &insert_qa_stmt,
            &[&"is this place free?", &"onks tä paikka vapaa?"],
        )
        .await
        .unwrap_err();

    println!(
        "duplicate insert error: code={:?}, src={:?}",
        err.code(),
        err.as_db_error()
    );

    Ok(())
}
