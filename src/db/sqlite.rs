use crate::db::sql::DatabaseValue;
use crate::db::{Database, DatabaseExecutor, Sql};
use crate::model::UserStatus;
use async_trait::async_trait;
use rusqlite::{Connection, Result as SqliteResult, params_from_iter};
use std::sync::{Arc, Mutex};
use worker::{Error, Result};

pub struct SqliteExecutor {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteExecutor {
    pub fn new(conn: Connection) -> Self {
        Self {
            conn: Arc::new(Mutex::new(conn)),
        }
    }

    pub fn new_in_memory() -> SqliteResult<Self> {
        let conn = Connection::open_in_memory()?;
        Ok(Self::new(conn))
    }
}

#[async_trait(?Send)]
impl DatabaseExecutor for SqliteExecutor {
    async fn query_all<T>(&self, sql: Sql<'_>) -> Result<Vec<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let conn = self.conn.lock().unwrap();
        let query = sql.sql();
        let values = sql.values();

        let params = values.into_iter().map(|v| match v {
            DatabaseValue::Text(s) => rusqlite::types::Value::Text(s),
            DatabaseValue::Int(i) => rusqlite::types::Value::Integer(i),
            DatabaseValue::Real(r) => rusqlite::types::Value::Real(r),
            DatabaseValue::Blob(b) => rusqlite::types::Value::Blob(b),
            DatabaseValue::Null => rusqlite::types::Value::Null,
        });

        let mut stmt = conn
            .prepare(&query)
            .map_err(|e| Error::RustError(e.to_string()))?;
        let column_names: Vec<String> = stmt
            .column_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        let rows = stmt
            .query_map(params_from_iter(params), |row| {
                let mut map = serde_json::Map::new();
                for (i, name) in column_names.iter().enumerate() {
                    let val: serde_json::Value = match row.get_ref(i)? {
                        rusqlite::types::ValueRef::Null => serde_json::Value::Null,
                        rusqlite::types::ValueRef::Integer(i) => {
                            serde_json::Value::Number(i.into())
                        }
                        rusqlite::types::ValueRef::Real(f) => serde_json::Number::from_f64(f)
                            .map(serde_json::Value::Number)
                            .unwrap_or(serde_json::Value::Null),
                        rusqlite::types::ValueRef::Text(t) => {
                            serde_json::Value::String(String::from_utf8_lossy(t).into_owned())
                        }
                        rusqlite::types::ValueRef::Blob(b) => {
                            serde_json::Value::String(hex::encode(b))
                        }
                    };
                    map.insert(name.clone(), val);
                }
                Ok(serde_json::Value::Object(map))
            })
            .map_err(|e| Error::RustError(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            let val = row.map_err(|e| Error::RustError(e.to_string()))?;
            let t: T = serde_json::from_value(val).map_err(|e| Error::RustError(e.to_string()))?;
            results.push(t);
        }

        Ok(results)
    }

    async fn query_first<T>(&self, sql: Sql<'_>) -> Result<Option<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let results: Vec<T> = self.query_all(sql).await?;
        Ok(results.into_iter().next())
    }

    async fn execute(&self, sql: Sql<'_>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let query = sql.sql();
        let values = sql.values();

        let params = values.into_iter().map(|v| match v {
            DatabaseValue::Text(s) => rusqlite::types::Value::Text(s),
            DatabaseValue::Int(i) => rusqlite::types::Value::Integer(i),
            DatabaseValue::Real(r) => rusqlite::types::Value::Real(r),
            DatabaseValue::Blob(b) => rusqlite::types::Value::Blob(b),
            DatabaseValue::Null => rusqlite::types::Value::Null,
        });

        conn.execute(&query, params_from_iter(params))
            .map_err(|e| Error::RustError(e.to_string()))?;
        Ok(())
    }

    async fn execute_batch(&self, sqls: Vec<Sql<'_>>) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn
            .transaction()
            .map_err(|e| Error::RustError(e.to_string()))?;
        for sql in sqls {
            let query = sql.sql();
            let values = sql.values();
            let params = values.into_iter().map(|v| match v {
                DatabaseValue::Text(s) => rusqlite::types::Value::Text(s),
                DatabaseValue::Int(i) => rusqlite::types::Value::Integer(i),
                DatabaseValue::Real(r) => rusqlite::types::Value::Real(r),
                DatabaseValue::Blob(b) => rusqlite::types::Value::Blob(b),
                DatabaseValue::Null => rusqlite::types::Value::Null,
            });
            tx.execute(&query, params_from_iter(params))
                .map_err(|e| Error::RustError(e.to_string()))?;
        }
        tx.commit().map_err(|e| Error::RustError(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::AppDatabase;
    use passkey_server::PasskeyStore;

    #[tokio::test(flavor = "current_thread")]
    async fn test_sqlite_workflow() -> Result<()> {
        let executor =
            SqliteExecutor::new_in_memory().map_err(|e| Error::RustError(e.to_string()))?;
        let db = AppDatabase::new(executor);

        // Run migrations
        db.migrate()
            .await
            .map_err(|e| Error::RustError(e.to_string()))?;

        // Create user
        let user = db
            .create_user(
                "test@example.com",
                "testuser",
                Some("hash"),
                None,
                None,
                None,
            )
            .await?;

        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.username, "testuser");

        // Get user by field
        let user2 = db
            .get_user_by_email("test@example.com")
            .await
            .map_err(|e| Error::RustError(e.to_string()))?
            .expect("User not found");
        assert_eq!(user2.id, user.id);

        // Update user
        db.update_user_field(
            user.id,
            "telegram_id",
            DatabaseValue::Text("12345".to_string()),
        )
        .await
        .map_err(|e| Error::RustError(e.to_string()))?;
        let user3 = db
            .get_user_by_id(user.id)
            .await
            .map_err(|e| Error::RustError(e.to_string()))?
            .expect("User not found");
        assert_eq!(user3.telegram_id, Some("12345".to_string()));

        // Sessions
        db.create_session(user.id, "token123", crate::utils::now_utc_ms() + 10000)
            .await?;
        let auth_user = db
            .get_user_by_session_token("token123")
            .await?
            .expect("Session not found");
        assert_eq!(auth_user.id, user.id);

        // User items
        db.update_user_item(user.id, "Anime Title", UserStatus::Watching, None, None)
            .await
            .map_err(|e| Error::RustError(e.to_string()))?;
        let items = db
            .get_user_items_all(user.id)
            .await
            .map_err(|e| Error::RustError(e.to_string()))?;
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Anime Title");

        // --- Passkey Tests ---
        // Create passkey
        let cred_id = "cred123";
        let public_key = "pubkey_json";
        let pk_name = "My Phone";
        let now = crate::utils::now_utc_ms();

        db.create_passkey(user.id.to_string(), cred_id, public_key, pk_name, 0, now)
            .await
            .map_err(|e| Error::RustError(e.to_string()))?;

        // Get passkey
        let pk = db
            .get_passkey(cred_id)
            .await
            .map_err(|e| Error::RustError(e.to_string()))?
            .expect("Passkey not found");
        assert_eq!(pk.user_id, user.id.to_string());
        assert_eq!(pk.public_key, public_key);
        assert_eq!(pk.name, pk_name);

        // List passkeys
        let pks = db
            .list_passkeys(user.id.to_string())
            .await
            .map_err(|e| Error::RustError(e.to_string()))?;
        assert_eq!(pks.len(), 1);
        assert_eq!(pks[0].cred_id, cred_id);

        // Update counter and name
        db.update_passkey_counter(cred_id, 1, now + 100)
            .await
            .map_err(|e| Error::RustError(e.to_string()))?;
        db.update_passkey_name(cred_id, "My New Phone")
            .await
            .map_err(|e| Error::RustError(e.to_string()))?;

        let pk_updated = db
            .get_passkey(cred_id)
            .await
            .map_err(|e| Error::RustError(e.to_string()))?
            .expect("Passkey not found");
        assert_eq!(pk_updated.counter, 1);
        assert_eq!(pk_updated.name, "My New Phone");

        // Passkey State Management
        let state_id = "state_id_123";
        let state_json = "{\"challenge\":\"abc\"}";
        let expires_at = now + 60000;

        db.save_state(state_id, state_json, expires_at)
            .await
            .map_err(|e| Error::RustError(e.to_string()))?;

        let state = db
            .get_state(state_id)
            .await
            .map_err(|e| Error::RustError(e.to_string()))?
            .expect("State not found");
        assert_eq!(state.id, state_id);
        assert_eq!(state.state_json, state_json);

        db.delete_state(state_id)
            .await
            .map_err(|e| Error::RustError(e.to_string()))?;
        let state_deleted = db
            .get_state(state_id)
            .await
            .map_err(|e| Error::RustError(e.to_string()))?;
        assert!(state_deleted.is_none());

        // Cleanup
        db.delete_passkey(user.id.to_string(), cred_id)
            .await
            .map_err(|e| Error::RustError(e.to_string()))?;
        let pk_deleted = db
            .get_passkey(cred_id)
            .await
            .map_err(|e| Error::RustError(e.to_string()))?;
        assert!(pk_deleted.is_none());

        Ok(())
    }
}
