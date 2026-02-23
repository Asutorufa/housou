use crate::error::Error;
use crate::traits::{DatabaseExecutor, Query, QueryExt, SqlBackend};
use crate::types::DatabaseValue;
use async_trait::async_trait;
use worker::D1Database;

pub struct WasmBackend;
impl SqlBackend for WasmBackend {
    type Param = worker::wasm_bindgen::JsValue;
    fn convert(value: DatabaseValue) -> Self::Param {
        match value {
            DatabaseValue::Text(s) => worker::wasm_bindgen::JsValue::from_str(&s),
            DatabaseValue::Int(i) => worker::wasm_bindgen::JsValue::from_f64(i as f64),
            DatabaseValue::UInt(u) => worker::wasm_bindgen::JsValue::from_f64(u as f64),
            DatabaseValue::Real(r) => worker::wasm_bindgen::JsValue::from_f64(r),
            DatabaseValue::Bool(b) => worker::wasm_bindgen::JsValue::from_bool(b),
            DatabaseValue::Blob(b) => js_sys::Uint8Array::from(&b[..]).into(),
            DatabaseValue::Null => worker::wasm_bindgen::JsValue::NULL,
        }
    }
}

#[async_trait(?Send)]
impl DatabaseExecutor for D1Database {
    async fn query_all<T, Q>(&self, sql: Q) -> Result<Vec<T>, Error>
    where
        T: serde::de::DeserializeOwned,
        Q: Query + 'async_trait,
    {
        let (sql_str, params) = sql.build_params::<WasmBackend>()?;
        self.prepare(sql_str.as_ref())
            .bind(&params)?
            .all()
            .await?
            .results()
            .map_err(Error::from)
    }

    async fn query_first<T, Q>(&self, sql: Q) -> Result<Option<T>, Error>
    where
        T: serde::de::DeserializeOwned,
        Q: Query + 'async_trait,
    {
        let (sql_str, params) = sql.build_params::<WasmBackend>()?;
        self.prepare(sql_str.as_ref())
            .bind(&params)?
            .first(None)
            .await
            .map_err(Error::from)
    }

    async fn execute<Q>(&self, sql: Q) -> Result<(), Error>
    where
        Q: Query + 'async_trait,
    {
        let (sql_str, params) = sql.build_params::<WasmBackend>()?;
        self.prepare(sql_str.as_ref()).bind(&params)?.run().await?;
        Ok(())
    }

    async fn execute_batch<Q>(&self, sqls: Vec<Q>) -> Result<(), Error>
    where
        Q: Query + 'async_trait,
    {
        let mut statements = Vec::with_capacity(sqls.len());
        for sql in sqls {
            let (sql_str, params) = sql.build_params::<WasmBackend>()?;
            statements.push(self.prepare(sql_str.as_ref()).bind(&params)?);
        }
        self.batch(statements).await?;
        Ok(())
    }
}
