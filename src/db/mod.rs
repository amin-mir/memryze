use std::{ops::Fn, pin::pin};

use futures::{stream::StreamExt, Future};
use tokio_postgres::{Client, Statement};

use crate::QA;

pub struct PgClient {
    client: Client,
    insert_qa_stmt: Statement,
    get_quiz_stmt: Statement,
}

impl PgClient {
    pub async fn prepare(client: Client) -> anyhow::Result<Self> {
        let insert_qa_stmt = client
            .prepare("INSERT INTO qa (q, a) VALUES ($1, $2)")
            .await?;

        let get_quiz_stmt = client
            .prepare(
                "SELECT * \
                FROM qa \
                WHERE correct_count < max
                AND (last_shown_at IS NULL OR last_shown_at < CURRENT_DATE) \
                ORDER BY created_at DESC, correct_count ASC \
                LIMIT 20",
            )
            .await?;

        Ok(Self {
            client,
            insert_qa_stmt,
            get_quiz_stmt,
        })
    }

    pub async fn insert_qa(&self, q: &str, a: &str) -> anyhow::Result<()> {
        self.client.execute(&self.insert_qa_stmt, &[&q, &a]).await?;
        Ok(())
    }

    // qas should have enough space for at least 20 QA because that is the limit
    // that we're using in the query.
    pub async fn get_quiz(&self, qas: &mut [QA]) -> anyhow::Result<usize> {
        let params: [&str; 0] = [];
        let row_iter = self.client.query_raw(&self.get_quiz_stmt, params).await?;

        // // Resize the qas so that it matches the rows len. This could help us reuse
        // // the memory allocated for previous quiz.
        // qas.resize_with(rows.len(), Default::default);
        // for i in 0..rows.len() {
        //     qas[i].q.clear();
        //     qas[i].q.push_str(rows[i].get(1));
        //     qas[i].a.clear();
        //     qas[i].a.push_str(rows[i].get(2));
        // }

        let mut row_iter = pin!(row_iter);

        let mut i = 0;
        while let Some(r) = row_iter.next().await {
            let r = r?;
            qas[i].q.clear();
            qas[i].q.push_str(r.get(1));
            qas[i].a.clear();
            qas[i].a.push_str(r.get(2));
            i += 1;
        }

        Ok(i)
    }
}
