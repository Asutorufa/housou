use crate::db::core::{DatabaseValue, MigrationInfo, QueryExt, SqlBackend};
use crate::db::models::UserItemUpdate;
use crate::db::sql::Sql;
use crate::db::{Database, DatabaseExecutor, Migration, SessionUpdate, UserUpdate};
use crate::model::UserStatus;
use async_trait::async_trait;
use rusqlite::{Connection, Result as SqliteResult, params_from_iter};
use std::sync::{Arc, Mutex};
use worker::{Error, Result};

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

    pub fn new_in_memory() -> SqliteResult<Self> {
        let conn = Connection::open_in_memory()?;
        Ok(Self::new(conn))
    }
}

#[async_trait(?Send)]
impl DatabaseExecutor for SqliteExecutor {
    async fn query_all<T, Q>(&self, sql: Q) -> worker::Result<Vec<T>>
    where
        T: serde::de::DeserializeOwned,
        Q: crate::db::core::Query + 'async_trait,
    {
        let conn = self.conn.lock().unwrap();
        let Ok((sql_str, params)) = sql.build_params::<SqliteBackend>() else {
            return Ok(Vec::new());
        };

        let mut stmt = conn
            .prepare(sql_str.as_ref())
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

    async fn query_first<T, Q>(&self, sql: Q) -> worker::Result<Option<T>>
    where
        T: serde::de::DeserializeOwned,
        Q: crate::db::core::Query + 'async_trait,
    {
        let results: Vec<T> = self.query_all(sql).await?;
        Ok(results.into_iter().next())
    }

    async fn execute<Q>(&self, sql: Q) -> worker::Result<()>
    where
        Q: crate::db::core::Query + 'async_trait,
    {
        let conn = self.conn.lock().unwrap();
        let Ok((sql_str, params)) = sql.build_params::<SqliteBackend>() else {
            return Ok(());
        };

        conn.execute(sql_str.as_ref(), params_from_iter(params))
            .map_err(|e| Error::RustError(e.to_string()))?;
        Ok(())
    }

    async fn execute_batch<Q>(&self, sqls: Vec<Q>) -> worker::Result<()>
    where
        Q: crate::db::core::Query + 'async_trait,
    {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn
            .transaction()
            .map_err(|e| Error::RustError(e.to_string()))?;
        for sql in sqls {
            let Ok((sql_str, params)) = sql.build_params::<SqliteBackend>() else {
                continue;
            };
            tx.execute(sql_str.as_ref(), params_from_iter(params))
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
        let user_by_email = db
            .get_user(UserUpdate::email("test@example.com".to_string()))
            .await?
            .expect("User not found");
        assert_eq!(user_by_email.id, user.id);

        let user_by_id = db
            .get_user(UserUpdate::id(user.id))
            .await?
            .expect("User not found");
        assert_eq!(user_by_id.id, user.id);

        // Update user
        db.update_user(
            user.id,
            vec![UserUpdate::telegram_id(Some("12345".to_string()))],
        )
        .await
        .map_err(|e| Error::RustError(e.to_string()))?;
        let user3 = db
            .get_user(UserUpdate::id(user.id))
            .await?
            .expect("User should exist after update");
        assert_eq!(user3.telegram_id, Some("12345".to_string()));

        // Sessions
        db.create_session(user.id, "token123", crate::utils::now_utc_ms() + 10000)
            .await?;
        let auth_user = db
            .get_user_by_session_token(SessionUpdate::token("token123".to_string()))
            .await?
            .expect("Session not found");
        assert_eq!(auth_user.id, user.id);

        // Update item
        db.update_user_item(
            user.id,
            "Anime Title",
            vec![
                UserItemUpdate::status(UserStatus::Completed),
                UserItemUpdate::score(Some(10)),
                UserItemUpdate::updated_at(crate::utils::now_utc_ms()),
            ],
        )
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

    #[tokio::test(flavor = "current_thread")]
    async fn test_update_user_no_effective_fields_is_noop() -> Result<()> {
        let executor =
            SqliteExecutor::new_in_memory().map_err(|e| Error::RustError(e.to_string()))?;
        let db = AppDatabase::new(executor);
        db.migrate().await?;

        let user = db
            .create_user("noop@example.com", "noop", Some("hash"), None, None, None)
            .await?;

        db.update_user(user.id, vec![UserUpdate::id(user.id)])
            .await?;

        let user_after = db
            .get_user(UserUpdate::id(user.id))
            .await?
            .expect("User should still exist");
        assert_eq!(user_after.id, user.id);
        assert_eq!(user_after.username, "noop");
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_update_user_item_no_effective_fields_is_noop() -> Result<()> {
        let executor =
            SqliteExecutor::new_in_memory().map_err(|e| Error::RustError(e.to_string()))?;
        let db = AppDatabase::new(executor);
        db.migrate().await?;

        let user = db
            .create_user(
                "item-noop@example.com",
                "item_noop",
                Some("hash"),
                None,
                None,
                None,
            )
            .await?;

        db.update_user_item(
            user.id,
            "Noop Title",
            vec![
                UserItemUpdate::user_id(user.id),
                UserItemUpdate::title("Noop Title".to_string()),
            ],
        )
        .await?;

        let items = db.get_user_items_all(user.id).await?;
        assert!(items.is_empty());
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_migration_steps() -> Result<()> {
        let executor =
            SqliteExecutor::new_in_memory().map_err(|e| Error::RustError(e.to_string()))?;
        let db = AppDatabase::new(executor);
        db.execute(Sql::CreateMigrationsTable).await?;

        // Test CreateTable
        let steps: &'static [Sql<'static>] = Box::leak(Box::new([Sql::AdHoc {
            info: MigrationInfo::Table("test_table"),
            sql: "CREATE TABLE test_table (id INTEGER PRIMARY KEY);".into(),
        }]));
        let migration = Migration { version: 1, steps };
        db.apply_migration(&migration).await?;
        assert!(db.has_table("test_table").await?);

        // Test CreateTable again (should be no-op due to has_table check)
        let migration_v2 = Migration { version: 2, steps };
        db.apply_migration(&migration_v2).await?;
        assert!(db.has_table("test_table").await?);

        // Test CreateIndex
        let steps_idx: &'static [Sql<'static>] = Box::leak(Box::new([Sql::AdHoc {
            info: MigrationInfo::Index("test_idx"),
            sql: "CREATE INDEX test_idx ON test_table(id);".into(),
        }]));
        let migration_v3 = Migration {
            version: 3,
            steps: steps_idx,
        };
        db.apply_migration(&migration_v3).await?;
        assert!(db.has_index("test_idx").await?);

        // Test CreateIndex again (should be no-op due to has_index check)
        let migration_v4 = Migration {
            version: 4,
            steps: steps_idx,
        };
        db.apply_migration(&migration_v4).await?;
        assert!(db.has_index("test_idx").await?);

        // Test AddColumnIfMissing
        let steps_col: &'static [Sql<'static>] = Box::leak(Box::new([Sql::AdHoc {
            info: MigrationInfo::Column {
                table: "test_table",
                column: "new_col",
            },
            sql: "ALTER TABLE test_table ADD COLUMN new_col TEXT;".into(),
        }]));
        let migration_v5 = Migration {
            version: 5,
            steps: steps_col,
        };
        db.apply_migration(&migration_v5).await?;
        assert!(db.has_column("test_table", "new_col").await?);

        // Test AddColumnIfMissing again (should be no-op due to has_column check)
        let migration_v6 = Migration {
            version: 6,
            steps: steps_col,
        };
        db.apply_migration(&migration_v6).await?;
        assert!(db.has_column("test_table", "new_col").await?);

        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_migrate_handles_existing_columns_without_version_records() -> Result<()> {
        let executor =
            SqliteExecutor::new_in_memory().map_err(|e| Error::RustError(e.to_string()))?;
        let db = AppDatabase::new(executor);

        db.execute(Sql::AdHoc {
            info: MigrationInfo::Table("users"),
            sql: "CREATE TABLE users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                email TEXT UNIQUE,
                username TEXT,
                password_hash TEXT,
                github_id TEXT UNIQUE,
                telegram_id TEXT,
                avatar_url TEXT,
                created_at INTEGER
            );"
            .into(),
        })
        .await?;
        db.execute(Sql::AdHoc {
            info: MigrationInfo::Table("user_items_v2"),
            sql: "CREATE TABLE user_items_v2 (
                user_id INTEGER,
                title TEXT,
                status INTEGER,
                score INTEGER,
                updated_at INTEGER,
                begin_at INTEGER,
                PRIMARY KEY (user_id, title)
            );"
            .into(),
        })
        .await?;

        db.migrate().await?;
        db.migrate().await?;
        Ok(())
    }
}
