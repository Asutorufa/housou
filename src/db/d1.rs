use crate::db::DatabaseExecutor;
use crate::db::sql::{DatabaseValue, Query, QueryExt, Sql, SqlBackend};
use async_trait::async_trait;
use worker::*;

pub struct WasmBackend;
impl SqlBackend for WasmBackend {
    type Param = worker::wasm_bindgen::JsValue;
    fn convert(value: DatabaseValue) -> Self::Param {
        match value {
            DatabaseValue::Text(s) => worker::wasm_bindgen::JsValue::from_str(&s),
            DatabaseValue::Int(i) => worker::wasm_bindgen::JsValue::from_f64(i as f64),
            DatabaseValue::Real(r) => worker::wasm_bindgen::JsValue::from_f64(r),
            DatabaseValue::Blob(b) => js_sys::Uint8Array::from(&b[..]).into(),
            DatabaseValue::Null => worker::wasm_bindgen::JsValue::NULL,
        }
    }
}

#[async_trait(?Send)]
impl DatabaseExecutor for D1Database {
    async fn query_all<T>(&self, sql: Sql<'_>) -> Result<Vec<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        self.prepare(sql.sql().as_ref())
            .bind(&sql.params::<WasmBackend>())?
            .all()
            .await?
            .results()
    }

    async fn query_first<T>(&self, sql: Sql<'_>) -> Result<Option<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        self.prepare(sql.sql().as_ref())
            .bind(&sql.params::<WasmBackend>())?
            .first(None)
            .await
    }

    async fn execute(&self, sql: Sql<'_>) -> Result<()> {
        self.prepare(sql.sql().as_ref())
            .bind(&sql.params::<WasmBackend>())?
            .run()
            .await?;
        Ok(())
    }

    async fn execute_batch(&self, sqls: Vec<Sql<'_>>) -> Result<()> {
        let mut statements = Vec::with_capacity(sqls.len());
        for sql in sqls {
            statements.push(
                self.prepare(sql.sql().as_ref())
                    .bind(&sql.params::<WasmBackend>())?,
            );
        }
        self.batch(statements).await?;
        Ok(())
    }
}
