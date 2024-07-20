use tokio_postgres::{Client, Statement};

pub struct PgClient {
    client: Client,
    insert_qa_stmt: Statement,
}

impl PgClient {
    pub async fn prepare(client: Client) -> anyhow::Result<Self> {
        let insert_qa_stmt = client
            .prepare("INSERT INTO qa (q, a) VALUES ($1, $2)")
            .await?;

        Ok(Self {
            client,
            insert_qa_stmt,
        })
    }

    pub async fn insert_qa(&self, q: &str, a: &str) -> anyhow::Result<()> {
        self.client.execute(&self.insert_qa_stmt, &[&q, &a]).await?;
        Ok(())
    }
}
