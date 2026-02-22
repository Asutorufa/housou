use crate::db::{DatabaseExecutor, Sql};
use async_trait::async_trait;
use worker::*;

#[async_trait(?Send)]
impl DatabaseExecutor for D1Database {
    async fn query_all<T>(&self, sql: Sql<'_>) -> Result<Vec<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        self.prepare(sql.sql())
            .bind(&sql.params())?
            .all()
            .await?
            .results()
    }

    async fn query_first<T>(&self, sql: Sql<'_>) -> Result<Option<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        self.prepare(sql.sql())
            .bind(&sql.params())?
            .first(None)
            .await
    }

    async fn execute(&self, sql: Sql<'_>) -> Result<()> {
        self.prepare(sql.sql()).bind(&sql.params())?.run().await?;
        Ok(())
    }

    async fn execute_batch(&self, sqls: Vec<Sql<'_>>) -> Result<()> {
        let mut statements = Vec::with_capacity(sqls.len());
        for sql in sqls {
            statements.push(self.prepare(sql.sql()).bind(&sql.params())?);
        }
        self.batch(statements).await?;
        Ok(())
    }
}
