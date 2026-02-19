use crate::model::UserStatus;
use async_trait::async_trait;
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Session {
    pub id: i32,
    pub user_id: i32,
    pub token: String,
    pub expires_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserItem {
    pub user_id: i32,
    pub title: String, // Changed from item_id
    pub status: UserStatus,
    pub score: Option<i32>,
    pub updated_at: i64,
}

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
    ) -> Result<()>;
    async fn get_user_item(&self, user_id: i32, title: &str) -> Result<Option<UserItem>>;
    async fn get_user_items_by_titles(
        &self,
        user_id: i32,
        titles: &[String],
    ) -> Result<Vec<UserItem>>;
}

pub struct AppDatabase {
    db: D1Database,
}

impl AppDatabase {
    pub fn new(db: D1Database) -> Self {
        Self { db }
    }
}

#[async_trait(?Send)]
impl Database for AppDatabase {
    async fn migrate(&self) -> Result<()> {
        // Create schema_migrations table if not exists
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

                let now = Date::now().as_millis() as i64;
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
        let created_at = Date::now().as_millis() as i64;
        let query = "INSERT INTO users (email, username, password_hash, github_id, telegram_id, avatar_url, created_at) VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING *";

        let password_val = if let Some(h) = password_hash {
            JsValue::from_str(h)
        } else {
            JsValue::NULL
        };
        let github_val = if let Some(g) = github_id {
            JsValue::from_str(g)
        } else {
            JsValue::NULL
        };
        let telegram_val = if let Some(t) = telegram_id {
            JsValue::from_str(t)
        } else {
            JsValue::NULL
        };
        let avatar_val = if let Some(a) = avatar_url {
            JsValue::from_str(a)
        } else {
            JsValue::NULL
        };

        let stmt = self.db.prepare(query).bind(&[
            JsValue::from_str(email),
            JsValue::from_str(username),
            password_val,
            github_val,
            telegram_val,
            avatar_val,
            JsValue::from_f64(created_at as f64),
        ])?;

        let result: Option<User> = stmt.first(None).await?;
        result.ok_or_else(|| Error::RustError("Failed to create user".to_string()))
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let query = "SELECT * FROM users WHERE email = ?";
        self.db
            .prepare(query)
            .bind(&[JsValue::from_str(email)])?
            .first(None)
            .await
    }

    async fn get_user_by_id(&self, id: i32) -> Result<Option<User>> {
        let query = "SELECT * FROM users WHERE id = ?";
        self.db
            .prepare(query)
            .bind(&[JsValue::from_f64(id as f64)])?
            .first(None)
            .await
    }

    async fn get_user_by_github_id(&self, github_id: &str) -> Result<Option<User>> {
        let query = "SELECT * FROM users WHERE github_id = ?";
        self.db
            .prepare(query)
            .bind(&[JsValue::from_str(github_id)])?
            .first(None)
            .await
    }

    async fn get_user_by_telegram_id(&self, telegram_id: &str) -> Result<Option<User>> {
        let query = "SELECT * FROM users WHERE telegram_id = ?";
        self.db
            .prepare(query)
            .bind(&[JsValue::from_str(telegram_id)])?
            .first(None)
            .await
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let query = "SELECT * FROM users WHERE username = ?";
        self.db
            .prepare(query)
            .bind(&[JsValue::from_str(username)])?
            .first(None)
            .await
    }

    async fn update_user_profile(
        &self,
        id: i32,
        new_username: &str,
        new_email: Option<&str>,
        new_avatar_url: Option<&str>,
    ) -> Result<()> {
        let mut updates = Vec::new();
        let mut bindings = Vec::new();

        updates.push("username = ?");
        bindings.push(JsValue::from_str(new_username));

        if let Some(email) = new_email {
            updates.push("email = ?");
            bindings.push(JsValue::from_str(email));
        }

        // Always update avatar_url, setting to NULL if None (explicit clear)
        updates.push("avatar_url = ?");
        if let Some(avatar) = new_avatar_url {
            bindings.push(JsValue::from_str(avatar));
        } else {
            bindings.push(JsValue::NULL);
        }

        let query = format!("UPDATE users SET {} WHERE id = ?", updates.join(", "));
        bindings.push(JsValue::from_f64(id as f64));

        self.db.prepare(&query).bind(&bindings)?.run().await?;
        Ok(())
    }

    async fn update_user_password(&self, id: i32, password_hash: &str) -> Result<()> {
        let query = "UPDATE users SET password_hash = ? WHERE id = ?";
        self.db
            .prepare(query)
            .bind(&[
                JsValue::from_str(password_hash),
                JsValue::from_f64(id as f64),
            ])?
            .run()
            .await?;
        Ok(())
    }

    async fn update_user_telegram_id(&self, id: i32, telegram_id: Option<&str>) -> Result<()> {
        let query = "UPDATE users SET telegram_id = ? WHERE id = ?";
        let tel_val = if let Some(t) = telegram_id {
            JsValue::from_str(t)
        } else {
            JsValue::NULL
        };
        self.db
            .prepare(query)
            .bind(&[tel_val, JsValue::from_f64(id as f64)])?
            .run()
            .await?;
        Ok(())
    }

    async fn update_user_github_id(&self, id: i32, github_id: Option<&str>) -> Result<()> {
        let query = "UPDATE users SET github_id = ? WHERE id = ?";
        let gh_val = if let Some(g) = github_id {
            JsValue::from_str(g)
        } else {
            JsValue::NULL
        };
        self.db
            .prepare(query)
            .bind(&[gh_val, JsValue::from_f64(id as f64)])?
            .run()
            .await?;
        Ok(())
    }

    async fn create_session(&self, user_id: i32, token: &str, expires_at: i64) -> Result<()> {
        let query = "INSERT INTO sessions (user_id, token, expires_at) VALUES (?, ?, ?)";
        self.db
            .prepare(query)
            .bind(&[
                JsValue::from_f64(user_id as f64),
                JsValue::from_str(token),
                JsValue::from_f64(expires_at as f64),
            ])?
            .run()
            .await?;
        Ok(())
    }

    async fn get_session(&self, token: &str) -> Result<Option<Session>> {
        let query = "SELECT * FROM sessions WHERE token = ? AND expires_at > ?";
        let now = Date::now().as_millis() as i64;
        self.db
            .prepare(query)
            .bind(&[JsValue::from_str(token), JsValue::from_f64(now as f64)])?
            .first(None)
            .await
    }

    async fn get_user_by_session_token(&self, token: &str) -> Result<Option<User>> {
        let query = "SELECT users.* FROM users INNER JOIN sessions ON users.id = sessions.user_id WHERE sessions.token = ? AND sessions.expires_at > ?";
        let now = Date::now().as_millis() as i64;
        self.db
            .prepare(query)
            .bind(&[JsValue::from_str(token), JsValue::from_f64(now as f64)])?
            .first(None)
            .await
    }

    async fn delete_session(&self, token: &str) -> Result<()> {
        let query = "DELETE FROM sessions WHERE token = ?";
        self.db
            .prepare(query)
            .bind(&[JsValue::from_str(token)])?
            .run()
            .await?;
        Ok(())
    }

    async fn update_user_item(
        &self,
        user_id: i32,
        title: &str,
        status: UserStatus,
        score: Option<i32>,
    ) -> Result<()> {
        let updated_at = Date::now().as_millis() as i64;
        // SQLite upsert
        let query = "INSERT INTO user_items_v2 (user_id, title, status, score, updated_at)
                     VALUES (?, ?, ?, ?, ?)
                     ON CONFLICT(user_id, title) DO UPDATE SET status = excluded.status, score = excluded.score, updated_at = excluded.updated_at";

        let score_val = if let Some(s) = score {
            JsValue::from_f64(s as f64)
        } else {
            JsValue::NULL
        };

        self.db
            .prepare(query)
            .bind(&[
                JsValue::from_f64(user_id as f64),
                JsValue::from_str(title),
                JsValue::from_f64(status as i32 as f64),
                score_val,
                JsValue::from_f64(updated_at as f64),
            ])?
            .run()
            .await?;
        Ok(())
    }

    async fn get_user_item(&self, user_id: i32, title: &str) -> Result<Option<UserItem>> {
        let query = "SELECT * FROM user_items_v2 WHERE user_id = ? AND title = ?";
        self.db
            .prepare(query)
            .bind(&[JsValue::from_f64(user_id as f64), JsValue::from_str(title)])?
            .first(None)
            .await
    }

    async fn get_user_items_by_titles(
        &self,
        user_id: i32,
        titles: &[String],
    ) -> Result<Vec<UserItem>> {
        if titles.is_empty() {
            return Ok(Vec::new());
        }

        // Chunk to avoid "too many SQL variables" error (D1 limit is 100 per query)
        let mut statements = Vec::new();
        for chunk in titles.chunks(50) {
            let placeholders = chunk.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let query = format!(
                "SELECT * FROM user_items_v2 WHERE user_id = ? AND title IN ({placeholders})"
            );

            let mut bindings = Vec::with_capacity(chunk.len() + 1);
            bindings.push(JsValue::from_f64(user_id as f64));
            for title in chunk {
                bindings.push(JsValue::from_str(title));
            }

            statements.push(self.db.prepare(&query).bind(&bindings)?);
        }

        let all_results: Vec<UserItem> = self
            .db
            .batch(statements)
            .await?
            .into_iter()
            .map(|res| res.results())
            .collect::<Result<Vec<_>>>()?
            .concat();

        Ok(all_results)
    }
}

// PasskeyStore implementation

/// Helper for deserializing DB rows into StoredPasskey.
/// The DB column is "passkey_json" but our struct field is "public_key".
#[derive(Debug, Deserialize)]
struct PasskeyRow {
    user_id: i32,
    cred_id: String,
    passkey_json: String,
    name: String,
    created_at: i64,
    last_used_at: i64,
    counter: i64,
}

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
        let query = "INSERT INTO passkeys (user_id, cred_id, passkey_json, name, created_at, last_used_at, counter) VALUES (?, ?, ?, ?, ?, ?, ?)";
        self.db
            .prepare(query)
            .bind(&[
                JsValue::from_f64(user_id_int as f64),
                JsValue::from_str(cred_id),
                JsValue::from_str(public_key),
                JsValue::from_str(name),
                JsValue::from_f64(created_at as f64),
                JsValue::from_f64(created_at as f64),
                JsValue::from_f64(counter as f64),
            ])
            .map_err(db_err)?
            .run()
            .await
            .map_err(db_err)?;
        Ok(())
    }

    async fn get_passkey(
        &self,
        cred_id: &str,
    ) -> passkey_server::error::Result<Option<StoredPasskey>> {
        let query = "SELECT * FROM passkeys WHERE cred_id = ?";
        let row: Option<PasskeyRow> = self
            .db
            .prepare(query)
            .bind(&[JsValue::from_str(cred_id)])
            .map_err(db_err)?
            .first(None)
            .await
            .map_err(db_err)?;
        Ok(row.map(Into::into))
    }

    async fn list_passkeys(
        &self,
        user_id: String,
    ) -> passkey_server::error::Result<Vec<StoredPasskey>> {
        let user_id_int = user_id
            .parse::<i32>()
            .map_err(|_| PasskeyError::InternalError("Invalid user ID".into()))?;
        let query = "SELECT * FROM passkeys WHERE user_id = ?";
        let results = self
            .db
            .prepare(query)
            .bind(&[JsValue::from_f64(user_id_int as f64)])
            .map_err(db_err)?
            .all()
            .await
            .map_err(db_err)?;
        let rows: Vec<PasskeyRow> = results.results().map_err(db_err)?;
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
        let query = "DELETE FROM passkeys WHERE user_id = ? AND cred_id = ?";
        self.db
            .prepare(query)
            .bind(&[
                JsValue::from_f64(user_id_int as f64),
                JsValue::from_str(cred_id),
            ])
            .map_err(db_err)?
            .run()
            .await
            .map_err(db_err)?;
        Ok(())
    }

    async fn update_passkey_counter(
        &self,
        cred_id: &str,
        new_counter: i64,
        last_used_at: i64,
    ) -> passkey_server::error::Result<()> {
        let query = "UPDATE passkeys SET counter = ?, last_used_at = ? WHERE cred_id = ?";
        self.db
            .prepare(query)
            .bind(&[
                JsValue::from_f64(new_counter as f64),
                JsValue::from_f64(last_used_at as f64),
                JsValue::from_str(cred_id),
            ])
            .map_err(db_err)?
            .run()
            .await
            .map_err(db_err)?;
        Ok(())
    }

    async fn update_passkey_name(
        &self,
        cred_id: &str,
        new_name: &str,
    ) -> passkey_server::error::Result<()> {
        let query = "UPDATE passkeys SET name = ? WHERE cred_id = ?";
        self.db
            .prepare(query)
            .bind(&[JsValue::from_str(new_name), JsValue::from_str(cred_id)])
            .map_err(db_err)?
            .run()
            .await
            .map_err(db_err)?;
        Ok(())
    }

    async fn save_state(
        &self,
        id: &str,
        state_json: &str,
        expires_at: i64,
    ) -> passkey_server::error::Result<()> {
        let now = Date::now().as_millis() as i64;

        let cleanup_stmt = self
            .db
            .prepare("DELETE FROM passkey_states WHERE expires_at < ?")
            .bind(&[JsValue::from_f64(now as f64)])
            .map_err(db_err)?;

        let insert_stmt = self
            .db
            .prepare("INSERT OR REPLACE INTO passkey_states (id, state_json, expires_at) VALUES (?, ?, ?)")
            .bind(&[
                JsValue::from_str(id),
                JsValue::from_str(state_json),
                JsValue::from_f64(expires_at as f64),
            ])
            .map_err(db_err)?;

        self.db
            .batch(vec![cleanup_stmt, insert_stmt])
            .await
            .map_err(db_err)?;
        Ok(())
    }

    async fn get_state(&self, id: &str) -> passkey_server::error::Result<Option<PasskeyState>> {
        let query = "SELECT * FROM passkey_states WHERE id = ? AND expires_at > ?";
        let now = Date::now().as_millis() as i64;
        self.db
            .prepare(query)
            .bind(&[JsValue::from_str(id), JsValue::from_f64(now as f64)])
            .map_err(db_err)?
            .first(None)
            .await
            .map_err(db_err)
    }

    async fn delete_state(&self, id: &str) -> passkey_server::error::Result<()> {
        let query = "DELETE FROM passkey_states WHERE id = ?";
        self.db
            .prepare(query)
            .bind(&[JsValue::from_str(id)])
            .map_err(db_err)?
            .run()
            .await
            .map_err(db_err)?;
        Ok(())
    }
}
