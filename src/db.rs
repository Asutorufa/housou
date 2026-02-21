use crate::model::UserStatus;
use crate::utils;
use async_trait::async_trait;
use d1_orm::{Bindable, D1Database, Repository, impl_model};
use passkey_server::types::{PasskeyState, StoredPasskey};
use passkey_server::{PasskeyError, PasskeyStore};

use serde_derive::{Deserialize, Serialize};
use worker::wasm_bindgen::JsValue;
use worker::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub username: String,
    pub avatar_url: Option<String>,
    #[serde(skip_serializing)]
    pub password_hash: Option<String>,
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub github_id: Option<String>,
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub telegram_id: Option<String>,
    pub created_at: i64,
}

impl_model!(User, "users");

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Session {
    pub id: i32,
    pub user_id: i32,
    pub token: String,
    pub expires_at: i64,
}

impl_model!(Session, "sessions");

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserItem {
    pub user_id: i32,
    pub title: String, // Changed from item_id
    pub status: UserStatus,
    pub score: Option<i32>,
    pub updated_at: i64,
    pub begin_at: Option<i64>,
}

impl_model!(UserItem, "user_items_v2");

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserItemSummary {
    pub status: UserStatus,
    pub score: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct SchemaVersion {
    version: Option<i32>,
}

#[async_trait(?Send)]
pub trait Database {
    async fn migrate(&self) -> Result<()>;

    async fn create_user(
        &self,
        email: &str,
        username: &str,
        password_hash: Option<&str>,
        github_id: Option<&str>,
        telegram_id: Option<&str>,
        avatar_url: Option<&str>,
    ) -> Result<User>;
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>>;
    async fn get_user_by_id(&self, id: i32) -> Result<Option<User>>;
    async fn get_user_by_github_id(&self, github_id: &str) -> Result<Option<User>>;
    async fn get_user_by_telegram_id(&self, telegram_id: &str) -> Result<Option<User>>;
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>>;
    async fn update_user_profile(
        &self,
        id: i32,
        new_username: &str,
        new_email: Option<&str>,
        new_avatar_url: Option<&str>,
    ) -> Result<()>;
    async fn update_user_password(&self, id: i32, password_hash: &str) -> Result<()>;
    async fn update_user_github_id(&self, id: i32, github_id: Option<&str>) -> Result<()>;
    async fn update_user_telegram_id(&self, id: i32, telegram_id: Option<&str>) -> Result<()>;

    async fn create_session(&self, user_id: i32, token: &str, expires_at: i64) -> Result<()>;
    #[allow(dead_code)]
    async fn get_session(&self, token: &str) -> Result<Option<Session>>;
    async fn get_user_by_session_token(&self, token: &str) -> Result<Option<User>>;
    async fn delete_session(&self, token: &str) -> Result<()>;

    async fn update_user_item(
        &self,
        user_id: i32,
        title: &str,
        status: UserStatus,
        score: Option<i32>,
        begin_at: Option<i64>,
    ) -> Result<()>;
    async fn get_user_items_all(&self, user_id: i32) -> Result<Vec<UserItem>>;
    async fn get_user_items_by_range(
        &self,
        user_id: i32,
        start_ts: i64,
        end_ts: i64,
    ) -> Result<Vec<UserItem>>;
}

trait IntoJsValue {
    fn to_js(&self) -> JsValue;
}

impl<T: AsRef<str>> IntoJsValue for Option<T> {
    fn to_js(&self) -> JsValue {
        match self {
            Some(s) => JsValue::from_str(s.as_ref()),
            None => JsValue::NULL,
        }
    }
}

pub struct AppDatabase {
    db: D1Database,
}

impl AppDatabase {
    pub fn new(db: D1Database) -> Self {
        Self { db }
    }

    fn users(&self) -> Repository<'_, User> {
        Repository::new(&self.db)
    }

    fn sessions(&self) -> Repository<'_, Session> {
        Repository::new(&self.db)
    }

    fn user_items(&self) -> Repository<'_, UserItem> {
        Repository::new(&self.db)
    }

    fn validate_select_field(field: &str) -> Result<()> {
        match field {
            "id" | "email" | "username" | "github_id" | "telegram_id" => Ok(()),
            _ => Err(Error::RustError(format!(
                "Invalid field for selection: {}",
                field
            ))),
        }
    }

    fn validate_update_field(field: &str) -> Result<()> {
        match field {
            "password_hash" | "telegram_id" | "github_id" => Ok(()),
            _ => Err(Error::RustError(format!(
                "Invalid field for update: {}",
                field
            ))),
        }
    }

    async fn get_user_by_field(&self, field: &str, value: JsValue) -> Result<Option<User>> {
        Self::validate_select_field(field)?;
        // Use ORM
        self.users()
            .find_one(self.users().select().where_eq(field, value))
            .await
            .map_err(|e| Error::RustError(e.to_string()))
    }

    async fn update_user_field(&self, id: i32, field: &str, value: JsValue) -> Result<()> {
        Self::validate_update_field(field)?;
        // Use ORM
        let update = self.users().update().set(field, value).where_eq("id", id);

        self.users()
            .execute(update)
            .await
            .map(|_| ())
            .map_err(|e| Error::RustError(e.to_string()))
    }
}

#[async_trait(?Send)]
impl Database for AppDatabase {
    async fn migrate(&self) -> Result<()> {
        // Create schema_migrations table if not exists
        // Migration logic kept as raw SQL for simplicity and safety during refactor
        self.db
            .prepare(
                "CREATE TABLE IF NOT EXISTS schema_migrations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                version INTEGER NOT NULL UNIQUE,
                applied_at INTEGER NOT NULL
            );",
            )
            .run()
            .await?;

        // Get current version
        let current_version: i32 = self
            .db
            .prepare("SELECT MAX(version) as version FROM schema_migrations")
            .first::<SchemaVersion>(None)
            .await?
            .and_then(|v| v.version)
            .unwrap_or(0);

        // Define migrations
        let migrations = vec![
            // Version 1: Initial schema + Updates
            (
                1,
                vec![
                    "CREATE TABLE IF NOT EXISTS users (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        email TEXT UNIQUE,
                        username TEXT,
                        password_hash TEXT,
                        github_id TEXT UNIQUE,
                        created_at INTEGER
                    );",
                    "CREATE TABLE IF NOT EXISTS sessions (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        user_id INTEGER,
                        token TEXT UNIQUE,
                        expires_at INTEGER,
                        FOREIGN KEY(user_id) REFERENCES users(id)
                    );",
                    "CREATE UNIQUE INDEX IF NOT EXISTS idx_users_username ON users(username);",
                    "CREATE TABLE IF NOT EXISTS user_items_v2 (
                        user_id INTEGER,
                        title TEXT,
                        status INTEGER,
                        score INTEGER,
                        updated_at INTEGER,
                        PRIMARY KEY (user_id, title),
                        FOREIGN KEY(user_id) REFERENCES users(id)
                    );",
                    "CREATE INDEX IF NOT EXISTS idx_user_items_v2_user_id ON user_items_v2(user_id);",
                ],
            ),
            (2, vec!["ALTER TABLE users ADD COLUMN avatar_url TEXT;"]),
            (
                3,
                vec![
                    "CREATE TABLE IF NOT EXISTS passkeys (
                        user_id INTEGER NOT NULL,
                        cred_id TEXT PRIMARY KEY,
                        passkey_json TEXT NOT NULL,
                        name TEXT NOT NULL,
                        created_at INTEGER NOT NULL,
                        last_used_at INTEGER NOT NULL,
                        counter INTEGER NOT NULL,
                        FOREIGN KEY(user_id) REFERENCES users(id)
                    );",
                    "CREATE INDEX IF NOT EXISTS idx_passkeys_user_id ON passkeys(user_id);",
                    "CREATE TABLE IF NOT EXISTS passkey_states (
                        id TEXT PRIMARY KEY,
                        state_json TEXT NOT NULL,
                        expires_at INTEGER NOT NULL
                    );",
                ],
            ),
            (
                4,
                vec![
                    "ALTER TABLE users ADD COLUMN telegram_id TEXT;",
                    "CREATE UNIQUE INDEX IF NOT EXISTS idx_users_telegram_id ON users(telegram_id);",
                ],
            ),
            (
                5,
                vec![
                    "ALTER TABLE user_items_v2 ADD COLUMN begin_at INTEGER;",
                    "CREATE INDEX IF NOT EXISTS idx_user_items_v2_begin_at ON user_items_v2(begin_at);",
                ],
            ),
        ];

        // Apply pending migrations
        const INSERT_MIGRATION_QUERY: &str =
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?, ?)";

        for (version, queries) in migrations {
            if version > current_version {
                console_log!("Applying migration version {}", version);
                let mut statements = Vec::with_capacity(queries.len() + 1);
                for query in queries {
                    statements.push(self.db.prepare(query));
                }

                let now = utils::now_utc_ms();
                statements.push(self.db.prepare(INSERT_MIGRATION_QUERY).bind(&[
                    JsValue::from_f64(version as f64),
                    JsValue::from_f64(now as f64),
                ])?);

                self.db.batch(statements).await?;
            }
        }

        Ok(())
    }

    async fn create_user(
        &self,
        email: &str,
        username: &str,
        password_hash: Option<&str>,
        github_id: Option<&str>,
        telegram_id: Option<&str>,
        avatar_url: Option<&str>,
    ) -> Result<User> {
        let created_at = utils::now_utc_ms();

        let insert = self
            .users()
            .insert()
            .set("email", email)
            .set("username", username)
            .set("password_hash", password_hash.to_js())
            .set("github_id", github_id.to_js())
            .set("telegram_id", telegram_id.to_js())
            .set("avatar_url", avatar_url.to_js())
            .set("created_at", created_at as f64)
            .returning("*");

        let result = self
            .users()
            .insert_one(insert)
            .await
            .map_err(|e| Error::RustError(e.to_string()))?;
        result.ok_or_else(|| Error::RustError("Failed to create user".to_string()))
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        self.get_user_by_field("email", JsValue::from_str(email))
            .await
    }

    async fn get_user_by_id(&self, id: i32) -> Result<Option<User>> {
        self.users()
            .find_by_id(id)
            .await
            .map_err(|e| Error::RustError(e.to_string()))
    }

    async fn get_user_by_github_id(&self, github_id: &str) -> Result<Option<User>> {
        self.get_user_by_field("github_id", JsValue::from_str(github_id))
            .await
    }

    async fn get_user_by_telegram_id(&self, telegram_id: &str) -> Result<Option<User>> {
        self.get_user_by_field("telegram_id", JsValue::from_str(telegram_id))
            .await
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        self.get_user_by_field("username", JsValue::from_str(username))
            .await
    }

    async fn update_user_profile(
        &self,
        id: i32,
        new_username: &str,
        new_email: Option<&str>,
        new_avatar_url: Option<&str>,
    ) -> Result<()> {
        let mut update = self
            .users()
            .update()
            .where_eq("id", id)
            .set("username", new_username)
            .set("avatar_url", new_avatar_url.to_js());

        if let Some(email) = new_email {
            update = update.set("email", email);
        }

        self.users()
            .execute(update)
            .await
            .map(|_| ())
            .map_err(|e| Error::RustError(e.to_string()))
    }

    async fn update_user_password(&self, id: i32, password_hash: &str) -> Result<()> {
        self.update_user_field(id, "password_hash", JsValue::from_str(password_hash))
            .await
    }

    async fn update_user_telegram_id(&self, id: i32, telegram_id: Option<&str>) -> Result<()> {
        self.update_user_field(id, "telegram_id", telegram_id.to_js())
            .await
    }

    async fn update_user_github_id(&self, id: i32, github_id: Option<&str>) -> Result<()> {
        self.update_user_field(id, "github_id", github_id.to_js())
            .await
    }

    async fn create_session(&self, user_id: i32, token: &str, expires_at: i64) -> Result<()> {
        let insert = self
            .sessions()
            .insert()
            .set("user_id", user_id)
            .set("token", token)
            .set("expires_at", expires_at as f64);

        self.sessions()
            .execute(insert)
            .await
            .map(|_| ())
            .map_err(|e| Error::RustError(e.to_string()))
    }

    async fn get_session(&self, token: &str) -> Result<Option<Session>> {
        let now = utils::now_utc_ms();
        let query = self
            .sessions()
            .select()
            .where_eq("token", token)
            .where_gt("expires_at", now as f64)
            .limit(1);
        self.sessions()
            .find_one(query)
            .await
            .map_err(|e| Error::RustError(e.to_string()))
    }

    async fn get_user_by_session_token(&self, token: &str) -> Result<Option<User>> {
        // Complex join: SELECT users.* FROM users INNER JOIN sessions ON ...
        // My simple builder handles basic joins if I implemented `join` in `Select`.
        // I implemented `join` in `Select`.
        let now = utils::now_utc_ms();
        let query = self
            .users()
            .select()
            .join("INNER JOIN sessions ON users.id = sessions.user_id")
            .where_eq("sessions.token", token)
            .where_gt("sessions.expires_at", now as f64)
            .limit(1);

        self.users()
            .find_one(query)
            .await
            .map_err(|e| Error::RustError(e.to_string()))
    }

    async fn delete_session(&self, token: &str) -> Result<()> {
        let delete = self.sessions().delete().where_eq("token", token);
        self.sessions()
            .execute(delete)
            .await
            .map(|_| ())
            .map_err(|e| Error::RustError(e.to_string()))
    }

    async fn update_user_item(
        &self,
        user_id: i32,
        title: &str,
        status: UserStatus,
        score: Option<i32>,
        begin_at: Option<i64>,
    ) -> Result<()> {
        let updated_at = utils::now_utc_ms();

        let score_val = if let Some(s) = score {
            JsValue::from_f64(s as f64)
        } else {
            JsValue::NULL
        };

        let begin_val = if let Some(b) = begin_at {
            JsValue::from_f64(b as f64)
        } else {
            JsValue::NULL
        };

        let insert = self
            .user_items()
            .insert()
            .set("user_id", user_id)
            .set("title", title)
            .set("status", status as i32)
            .set("score", score_val)
            .set("updated_at", updated_at as f64)
            .set("begin_at", begin_val)
            .on_conflict(
                "(user_id, title) DO UPDATE SET
                        status = excluded.status,
                        score = excluded.score,
                        updated_at = excluded.updated_at,
                        begin_at = COALESCE(excluded.begin_at, user_items_v2.begin_at)",
            );

        self.user_items()
            .execute(insert)
            .await
            .map(|_| ())
            .map_err(|e| Error::RustError(e.to_string()))
    }

    async fn get_user_items_all(&self, user_id: i32) -> Result<Vec<UserItem>> {
        let query = self
            .user_items()
            .select()
            .where_eq("user_id", user_id)
            .where_raw("status != ?", JsValue::from_f64(0.0));

        self.user_items()
            .find_all(query)
            .await
            .map_err(|e| Error::RustError(e.to_string()))
    }

    async fn get_user_items_by_range(
        &self,
        user_id: i32,
        start_ts: i64,
        end_ts: i64,
    ) -> Result<Vec<UserItem>> {
        let query = "SELECT * FROM user_items_v2 
                     WHERE user_id = ? AND status != 0 
                     AND (begin_at IS NULL OR (begin_at >= ? AND begin_at <= ?))";
        let results = self
            .db
            .prepare(query)
            .bind(&[
                JsValue::from_f64(user_id as f64),
                JsValue::from_f64(start_ts as f64),
                JsValue::from_f64(end_ts as f64),
            ])?
            .all()
            .await?;

        let rows: Vec<UserItem> = results.results()?;
        Ok(rows)
    }
}

// PasskeyStore implementation

/// Helper for deserializing DB rows into StoredPasskey.
/// The DB column is "passkey_json" but our struct field is "public_key".
#[derive(Debug, Serialize, Deserialize)]
pub struct PasskeyRow {
    user_id: i32,
    cred_id: String,
    passkey_json: String,
    name: String,
    created_at: i64,
    last_used_at: i64,
    counter: i64,
}

impl_model!(PasskeyRow, "passkeys", "cred_id");

impl From<PasskeyRow> for StoredPasskey {
    fn from(r: PasskeyRow) -> Self {
        Self {
            user_id: r.user_id.to_string(),
            cred_id: r.cred_id,
            public_key: r.passkey_json,
            name: r.name,
            created_at: r.created_at,
            last_used_at: r.last_used_at,
            counter: r.counter,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PasskeyStateRow {
    pub id: String,
    pub state_json: String,
    pub expires_at: i64,
}

impl_model!(PasskeyStateRow, "passkey_states", "id");

impl From<PasskeyStateRow> for PasskeyState {
    fn from(r: PasskeyStateRow) -> Self {
        Self {
            id: r.id,
            state_json: r.state_json,
            expires_at: r.expires_at,
        }
    }
}

fn db_err(e: impl std::fmt::Display) -> PasskeyError {
    PasskeyError::DatabaseError(e.to_string())
}

#[cfg_attr(not(feature = "send"), async_trait(?Send))]
#[cfg_attr(feature = "send", async_trait)]
impl PasskeyStore for AppDatabase {
    async fn create_passkey(
        &self,
        user_id: String,
        cred_id: &str,
        public_key: &str,
        name: &str,
        counter: i64,
        created_at: i64,
    ) -> passkey_server::error::Result<()> {
        let user_id_int = user_id
            .parse::<i32>()
            .map_err(|_| PasskeyError::InternalError("Invalid user ID".into()))?;

        let insert = self
            .passkeys()
            .insert()
            .set("user_id", user_id_int)
            .set("cred_id", cred_id)
            .set("passkey_json", public_key)
            .set("name", name)
            .set("created_at", created_at as f64)
            .set("last_used_at", created_at as f64)
            .set("counter", counter as f64);

        self.passkeys()
            .execute(insert)
            .await
            .map(|_| ())
            .map_err(db_err)
    }

    async fn get_passkey(
        &self,
        cred_id: &str,
    ) -> passkey_server::error::Result<Option<StoredPasskey>> {
        let row = self.passkeys().find_by_id(cred_id).await.map_err(db_err)?;
        Ok(row.map(Into::into))
    }

    async fn list_passkeys(
        &self,
        user_id: String,
    ) -> passkey_server::error::Result<Vec<StoredPasskey>> {
        let user_id_int = user_id
            .parse::<i32>()
            .map_err(|_| PasskeyError::InternalError("Invalid user ID".into()))?;

        let query = self.passkeys().select().where_eq("user_id", user_id_int);
        let rows = self.passkeys().find_all(query).await.map_err(db_err)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn delete_passkey(
        &self,
        user_id: String,
        cred_id: &str,
    ) -> passkey_server::error::Result<()> {
        let user_id_int = user_id
            .parse::<i32>()
            .map_err(|_| PasskeyError::InternalError("Invalid user ID".into()))?;

        let delete = self
            .passkeys()
            .delete()
            .where_eq("user_id", user_id_int)
            .where_eq("cred_id", cred_id);

        self.passkeys()
            .execute(delete)
            .await
            .map(|_| ())
            .map_err(db_err)
    }

    async fn update_passkey_counter(
        &self,
        cred_id: &str,
        new_counter: i64,
        last_used_at: i64,
    ) -> passkey_server::error::Result<()> {
        let update = self
            .passkeys()
            .update()
            .set("counter", new_counter as f64)
            .set("last_used_at", last_used_at as f64)
            .where_eq("cred_id", cred_id);

        self.passkeys()
            .execute(update)
            .await
            .map(|_| ())
            .map_err(db_err)
    }

    async fn update_passkey_name(
        &self,
        cred_id: &str,
        new_name: &str,
    ) -> passkey_server::error::Result<()> {
        let update = self
            .passkeys()
            .update()
            .set("name", new_name)
            .where_eq("cred_id", cred_id);

        self.passkeys()
            .execute(update)
            .await
            .map(|_| ())
            .map_err(db_err)
    }

    async fn save_state(
        &self,
        id: &str,
        state_json: &str,
        expires_at: i64,
    ) -> passkey_server::error::Result<()> {
        let now = utils::now_utc_ms();

        // Transaction/Batch manual
        // cleanup
        let delete = self
            .passkey_states()
            .delete()
            .where_lt("expires_at", now as f64);
        let (del_sql, del_bind) = delete.to_sql();

        // insert
        let insert = self
            .passkey_states()
            .insert()
            .set("id", id)
            .set("state_json", state_json)
            .set("expires_at", expires_at as f64)
            .on_conflict(
                "(id) DO UPDATE SET state_json=excluded.state_json, expires_at=excluded.expires_at",
            );
        let (ins_sql, ins_bind) = insert.to_sql();

        let s1 = self.db.prepare(&del_sql).bind(&del_bind).map_err(db_err)?;
        let s2 = self.db.prepare(&ins_sql).bind(&ins_bind).map_err(db_err)?;

        self.db.batch(vec![s1, s2]).await.map_err(db_err)?;
        Ok(())
    }

    async fn get_state(&self, id: &str) -> passkey_server::error::Result<Option<PasskeyState>> {
        let now = utils::now_utc_ms();
        let query = self
            .passkey_states()
            .select()
            .where_eq("id", id)
            .where_gt("expires_at", now as f64);

        let row: Option<PasskeyStateRow> = self
            .passkey_states()
            .find_one(query)
            .await
            .map_err(db_err)?;
        Ok(row.map(Into::into))
    }

    async fn delete_state(&self, id: &str) -> passkey_server::error::Result<()> {
        self.passkey_states().delete_by_id(id).await.map_err(db_err)
    }
}

// Add helpers for repositories
impl AppDatabase {
    fn passkeys(&self) -> Repository<'_, PasskeyRow> {
        Repository::new(&self.db)
    }

    fn passkey_states(&self) -> Repository<'_, PasskeyStateRow> {
        Repository::new(&self.db)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_select_field() {
        // Allowed fields
        assert!(AppDatabase::validate_select_field("id").is_ok());
        assert!(AppDatabase::validate_select_field("email").is_ok());
        assert!(AppDatabase::validate_select_field("username").is_ok());
        assert!(AppDatabase::validate_select_field("github_id").is_ok());
        assert!(AppDatabase::validate_select_field("telegram_id").is_ok());

        // Disallowed fields
        assert!(AppDatabase::validate_select_field("password_hash").is_err());
        assert!(AppDatabase::validate_select_field("created_at").is_err());
        assert!(AppDatabase::validate_select_field("avatar_url").is_err());
        assert!(AppDatabase::validate_select_field("1; DROP TABLE users").is_err());
    }

    #[test]
    fn test_validate_update_field() {
        // Allowed fields
        assert!(AppDatabase::validate_update_field("password_hash").is_ok());
        assert!(AppDatabase::validate_update_field("telegram_id").is_ok());
        assert!(AppDatabase::validate_update_field("github_id").is_ok());

        // Disallowed fields
        assert!(AppDatabase::validate_update_field("email").is_err());
        assert!(AppDatabase::validate_update_field("username").is_err());
        assert!(AppDatabase::validate_update_field("id").is_err());
        assert!(AppDatabase::validate_update_field("avatar_url").is_err());
        assert!(AppDatabase::validate_update_field("1; DROP TABLE users").is_err());
    }
}
