use crate::model::UserStatus;
use async_trait::async_trait;
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
        avatar_url: Option<&str>,
    ) -> Result<User>;
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>>;
    async fn get_user_by_id(&self, id: i32) -> Result<Option<User>>;
    async fn get_user_by_github_id(&self, github_id: &str) -> Result<Option<User>>;
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>>;
    async fn update_user_profile(
        &self,
        id: i32,
        new_username: &str,
        new_email: Option<&str>,
        new_avatar_url: Option<&str>,
    ) -> Result<()>;
    async fn update_user_password(&self, id: i32, password_hash: &str) -> Result<()>;

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
        ];

        // Apply pending migrations
        for (version, queries) in migrations {
            if version > current_version {
                console_log!("Applying migration version {}", version);
                for query in queries {
                    self.db.prepare(query).run().await?;
                }

                let now = Date::now().as_millis() as i64;
                self.db
                    .prepare("INSERT INTO schema_migrations (version, applied_at) VALUES (?, ?)")
                    .bind(&[
                        JsValue::from_f64(version as f64),
                        JsValue::from_f64(now as f64),
                    ])?
                    .run()
                    .await?;
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
        avatar_url: Option<&str>,
    ) -> Result<User> {
        let created_at = Date::now().as_millis() as i64;
        let query = "INSERT INTO users (email, username, password_hash, github_id, avatar_url, created_at) VALUES (?, ?, ?, ?, ?, ?) RETURNING *";

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
        let email_part = if let Some(email) = new_email {
            ("email = ?", JsValue::from_str(email))
        } else {
            ("", JsValue::NULL)
        };

        let avatar_part = if let Some(avatar) = new_avatar_url {
            ("avatar_url = ?", JsValue::from_str(avatar))
        } else {
            ("", JsValue::NULL)
        };

        let mut query = "UPDATE users SET username = ?".to_string();
        let mut bindings = vec![JsValue::from_str(new_username)];

        if !email_part.0.is_empty() {
            query.push_str(", ");
            query.push_str(email_part.0);
            bindings.push(email_part.1);
        }

        if !avatar_part.0.is_empty() {
            query.push_str(", ");
            query.push_str(avatar_part.0);
            bindings.push(avatar_part.1);
        } else {
            // If new_avatar_url is None, we might want to clear it?
            // Or just leave it as is if it's not provided in the update.
            // Let's assume Option<&str> means "set to this value (which could be None to clear)".
            // Actually, let's treat it as "update if provided".
            // If the user wants to clear, they send empty string? No, let's use Option properly.
            // But how do we distinguish "don't update" vs "set to null"?
            // Usually we set to null if it's explicitly passed as None in a Patch.
            // For now, let's just update it every time.
            query.push_str(", avatar_url = ?");
            bindings.push(JsValue::NULL);
        }

        query.push_str(" WHERE id = ?");
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

        let all_results: Vec<UserItem> = self.db.batch(statements).await?
            .into_iter()
            .map(|res| res.results())
            .collect::<Result<Vec<_>>>()?
            .concat();

        Ok(all_results)
    }
}
