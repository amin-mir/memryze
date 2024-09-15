use std::pin::pin;

use futures::stream::StreamExt;
use tokio_postgres::{Client, Statement};

use message::QA;

pub struct PgClient {
    client: Client,
    custid_from_tkn_stmt: Statement,
    insert_qa_stmt: Statement,
    get_quiz_stmt: Statement,
    correct_review_stmt: Statement,
    wrong_review_stmt: Statement,
}

impl PgClient {
    pub async fn prepare(client: Client) -> anyhow::Result<Self> {
        let custid_from_tkn_stmt = client
            .prepare("SELECT id FROM customer WHERE token = $1")
            .await?;

        let insert_qa_stmt = client
            .prepare("INSERT INTO qa (q, a) VALUES ($1, $2)")
            .await?;

        let get_quiz_stmt = client
            .prepare(
                "SELECT id, q, a \
                FROM qa \
                WHERE customer_id = $1
                AND correct_count < max \
                AND (last_shown_at IS NULL OR last_shown_at < CURRENT_DATE) \
                ORDER BY created_at DESC \
                LIMIT 20",
            )
            .await?;

        let correct_review_stmt = client
            .prepare(
                "UPDATE qa \
                SET correct_count = correct_count + 1, \
                    last_shown_at = CURRENT_TIMESTAMP \
                WHERE id = $1",
            )
            .await?;

        let wrong_review_stmt = client
            .prepare(
                "UPDATE qa \
                SET last_shown_at = CURRENT_TIMESTAMP \
                WHERE id = $1",
            )
            .await?;

        Ok(Self {
            client,
            custid_from_tkn_stmt,
            insert_qa_stmt,
            get_quiz_stmt,
            correct_review_stmt,
            wrong_review_stmt,
        })
    }

    pub async fn customer_id_from_token(&self, token: &str) -> anyhow::Result<i64> {
        let row = self
            .client
            .query_one(&self.custid_from_tkn_stmt, &[&token])
            .await?;
        let id: i64 = row.get("id");
        Ok(id)
    }

    pub async fn insert_qa(&self, customer_id: i64, q: &str, a: &str) -> anyhow::Result<()> {
        self.client
            .execute(&self.insert_qa_stmt, &[&q, &a, &customer_id])
            .await?;
        Ok(())
    }

    // qas should have enough space for at least 20 QA because that is the limit
    // that we're using in the query.
    pub async fn get_quiz(&self, customer_id: i64, qas: &mut [QA]) -> anyhow::Result<usize> {
        let row_iter = self
            .client
            .query_raw(&self.get_quiz_stmt, &[customer_id])
            .await?;

        let mut row_iter = pin!(row_iter);

        let mut i = 0;
        while let Some(r) = row_iter.next().await {
            let r = r?;
            qas[i].id = r.get(0);

            qas[i].q.clear();
            qas[i].q.push_str(r.get(1));

            qas[i].a.clear();
            qas[i].a.push_str(r.get(2));
            i += 1;
        }

        Ok(i)
    }

    pub async fn review_qa(&self, id: i64, correct: bool) -> anyhow::Result<()> {
        let stmt = if correct {
            &self.correct_review_stmt
        } else {
            &self.wrong_review_stmt
        };

        let n = self.client.execute(stmt, &[&id]).await?;
        anyhow::ensure!(n == 1);

        Ok(())
    }
}
