use crate::error::Error;
use crate::traits::{DatabaseExecutor, Query, QueryExt, SqlBackend};
use crate::types::DatabaseValue;
use async_trait::async_trait;
use rusqlite::{params_from_iter, Connection};
use std::sync::{Arc, Mutex};

pub struct SqliteBackend;
impl SqlBackend for SqliteBackend {
    type Param = rusqlite::types::Value;
    fn convert(v: DatabaseValue) -> Self::Param {
        match v {
            DatabaseValue::Text(s) => rusqlite::types::Value::Text(s),
            DatabaseValue::Int(i) => rusqlite::types::Value::Integer(i),
            DatabaseValue::UInt(u) => rusqlite::types::Value::Integer(u as i64),
            DatabaseValue::Real(r) => rusqlite::types::Value::Real(r),
            DatabaseValue::Bool(b) => rusqlite::types::Value::Integer(if b { 1 } else { 0 }),
            DatabaseValue::Blob(b) => rusqlite::types::Value::Blob(b),
            DatabaseValue::Null => rusqlite::types::Value::Null,
        }
    }
}

pub struct SqliteExecutor {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteExecutor {
    pub fn new(conn: Connection) -> Self {
        Self {
            conn: Arc::new(Mutex::new(conn)),
        }
    }

    pub fn new_in_memory() -> Result<Self, Error> {
        let conn = Connection::open_in_memory()?;
        Ok(Self::new(conn))
    }
}

#[async_trait(?Send)]
impl DatabaseExecutor for SqliteExecutor {
    async fn query_all<T, Q>(&self, sql: Q) -> Result<Vec<T>, Error>
    where
        T: serde::de::DeserializeOwned,
        Q: Query + 'async_trait,
    {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::Other(format!("Mutex lock failed: {}", e)))?;
        let (sql_str, params) = sql.build_params::<SqliteBackend>()?;

        let mut stmt = conn.prepare(sql_str.as_ref())?;
        let column_names: Vec<String> = stmt
            .column_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        let rows = stmt.query_map(params_from_iter(params), |row| {
            let mut map = serde_json::Map::new();
            for (i, name) in column_names.iter().enumerate() {
                let val: serde_json::Value = match row.get_ref(i)? {
                    rusqlite::types::ValueRef::Null => serde_json::Value::Null,
                    rusqlite::types::ValueRef::Integer(i) => serde_json::Value::Number(i.into()),
                    rusqlite::types::ValueRef::Real(f) => serde_json::Number::from_f64(f)
                        .map(serde_json::Value::Number)
                        .unwrap_or(serde_json::Value::Null),
                    rusqlite::types::ValueRef::Text(t) => {
                        serde_json::Value::String(String::from_utf8_lossy(t).into_owned())
                    }
                    rusqlite::types::ValueRef::Blob(b) => serde_json::Value::String(hex::encode(b)),
                };
                map.insert(name.clone(), val);
            }
            Ok(serde_json::Value::Object(map))
        })?;

        let mut results = Vec::new();
        for row in rows {
            let val = row?;
            let t: T = serde_json::from_value(val).map_err(|e| Error::Other(e.to_string()))?;
            results.push(t);
        }

        Ok(results)
    }

    async fn query_first<T, Q>(&self, sql: Q) -> Result<Option<T>, Error>
    where
        T: serde::de::DeserializeOwned,
        Q: Query + 'async_trait,
    {
        let results: Vec<T> = self.query_all(sql).await?;
        Ok(results.into_iter().next())
    }

    async fn execute<Q>(&self, sql: Q) -> Result<(), Error>
    where
        Q: Query + 'async_trait,
    {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::Other(format!("Mutex lock failed: {}", e)))?;
        let (sql_str, params) = sql.build_params::<SqliteBackend>()?;

        conn.execute(sql_str.as_ref(), params_from_iter(params))?;
        Ok(())
    }

    async fn execute_batch<Q>(&self, sqls: Vec<Q>) -> Result<(), Error>
    where
        Q: Query + 'async_trait,
    {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| Error::Other(format!("Mutex lock failed: {}", e)))?;
        let tx = conn.transaction()?;
        for sql in sqls {
            let (sql_str, params) = sql.build_params::<SqliteBackend>()?;
            tx.execute(sql_str.as_ref(), params_from_iter(params))?;
        }
        tx.commit()?;
        Ok(())
    }
}
