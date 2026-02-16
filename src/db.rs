use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use worker::wasm_bindgen::JsValue;
use worker::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub username: String,
    pub password_hash: Option<String>,
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
    pub item_id: String,
    pub status: i32,
    pub score: Option<i32>,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
struct SchemaVersion {
    version: i32,
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
    ) -> Result<()>;

    async fn create_session(&self, user_id: i32, token: &str, expires_at: i64) -> Result<()>;
    async fn get_session(&self, token: &str) -> Result<Option<Session>>;
    async fn delete_session(&self, token: &str) -> Result<()>;

    async fn update_user_item(
        &self,
        user_id: i32,
        item_id: &str,
        status: i32,
        score: Option<i32>,
    ) -> Result<()>;
    async fn get_user_item(&self, user_id: i32, item_id: &str) -> Result<Option<UserItem>>;
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
            .map(|v| v.version)
            .unwrap_or(0);

        // Define migrations
        let migrations = vec![
            // Version 1: Initial schema
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
                    "CREATE TABLE IF NOT EXISTS user_items (
                        user_id INTEGER,
                        item_id TEXT,
                        status INTEGER,
                        score INTEGER,
                        updated_at INTEGER,
                        PRIMARY KEY (user_id, item_id),
                        FOREIGN KEY(user_id) REFERENCES users(id)
                    );",
                ],
            ),
            // Version 2: Add unique index on username
            (
                2,
                vec!["CREATE UNIQUE INDEX IF NOT EXISTS idx_users_username ON users(username);"],
            ),
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
    ) -> Result<User> {
        let created_at = Date::now().as_millis() as i64;
        let query = "INSERT INTO users (email, username, password_hash, github_id, created_at) VALUES (?, ?, ?, ?, ?) RETURNING *";

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

        let stmt = self.db.prepare(query).bind(&[
            JsValue::from_str(email),
            JsValue::from_str(username),
            password_val,
            github_val,
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
    ) -> Result<()> {
        if let Some(email) = new_email {
            let query = "UPDATE users SET username = ?, email = ? WHERE id = ?";
            self.db
                .prepare(query)
                .bind(&[
                    JsValue::from_str(new_username),
                    JsValue::from_str(email),
                    JsValue::from_f64(id as f64),
                ])?
                .run()
                .await?;
        } else {
            let query = "UPDATE users SET username = ? WHERE id = ?";
            self.db
                .prepare(query)
                .bind(&[
                    JsValue::from_str(new_username),
                    JsValue::from_f64(id as f64),
                ])?
                .run()
                .await?;
        }
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
        item_id: &str,
        status: i32,
        score: Option<i32>,
    ) -> Result<()> {
        let updated_at = Date::now().as_millis() as i64;
        // SQLite upsert
        let query = "INSERT INTO user_items (user_id, item_id, status, score, updated_at)
                     VALUES (?, ?, ?, ?, ?)
                     ON CONFLICT(user_id, item_id) DO UPDATE SET status = excluded.status, score = excluded.score, updated_at = excluded.updated_at";

        let score_val = if let Some(s) = score {
            JsValue::from_f64(s as f64)
        } else {
            JsValue::NULL
        };

        self.db
            .prepare(query)
            .bind(&[
                JsValue::from_f64(user_id as f64),
                JsValue::from_str(item_id),
                JsValue::from_f64(status as f64),
                score_val,
                JsValue::from_f64(updated_at as f64),
            ])?
            .run()
            .await?;
        Ok(())
    }

    async fn get_user_item(&self, user_id: i32, item_id: &str) -> Result<Option<UserItem>> {
        let query = "SELECT * FROM user_items WHERE user_id = ? AND item_id = ?";
        self.db
            .prepare(query)
            .bind(&[
                JsValue::from_f64(user_id as f64),
                JsValue::from_str(item_id),
            ])?
            .first(None)
            .await
    }
}
