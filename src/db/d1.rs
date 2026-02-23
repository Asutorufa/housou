use crate::db::DatabaseExecutor;
use crate::db::core::{DatabaseValue, QueryExt, SqlBackend};
use async_trait::async_trait;
use worker::*;

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
    async fn query_all<T, Q>(&self, sql: Q) -> worker::Result<Vec<T>>
    where
        T: serde::de::DeserializeOwned,
        Q: crate::db::core::Query + 'async_trait,
    {
        let Ok((sql_str, params)) = sql.build_params::<WasmBackend>() else {
            return Ok(Vec::new());
        };
        self.prepare(sql_str).bind(&params)?.all().await?.results()
    }

    async fn query_first<T, Q>(&self, sql: Q) -> worker::Result<Option<T>>
    where
        T: serde::de::DeserializeOwned,
        Q: crate::db::core::Query + 'async_trait,
    {
        let Ok((sql_str, params)) = sql.build_params::<WasmBackend>() else {
            return Ok(None);
        };
        self.prepare(sql_str).bind(&params)?.first(None).await
    }

    async fn execute<Q>(&self, sql: Q) -> worker::Result<()>
    where
        Q: crate::db::core::Query + 'async_trait,
    {
        let Ok((sql_str, params)) = sql.build_params::<WasmBackend>() else {
            return Ok(());
        };
        self.prepare(sql_str).bind(&params)?.run().await?;
        Ok(())
    }

    async fn execute_batch<Q>(&self, sqls: Vec<Q>) -> worker::Result<()>
    where
        Q: crate::db::core::Query + 'async_trait,
    {
        let mut statements = Vec::with_capacity(sqls.len());
        for sql in sqls {
            let Ok((sql_str, params)) = sql.build_params::<WasmBackend>() else {
                continue;
            };
            statements.push(self.prepare(sql_str).bind(&params)?);
        }
        self.batch(statements).await?;
        Ok(())
    }
}
