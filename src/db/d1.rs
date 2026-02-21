use crate::db::models::{SchemaVersion, User, UserItem};
use crate::db::sql::DatabaseValue;
use crate::db::{AppDatabase, Database, DatabaseExecutor, Sql};
use crate::model::UserStatus;
use crate::utils;
use async_trait::async_trait;
use worker::*;

#[async_trait(?Send)]
impl DatabaseExecutor for D1Database {
    async fn query_all<T>(&self, sql: Sql<'_>) -> Result<Vec<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        self.prepare(sql.sql())
            .bind(&sql.params())?
            .all()
            .await?
            .results()
    }

    async fn query_first<T>(&self, sql: Sql<'_>) -> Result<Option<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        self.prepare(sql.sql())
            .bind(&sql.params())?
            .first(None)
            .await
    }

    async fn execute(&self, sql: Sql<'_>) -> Result<()> {
        self.prepare(sql.sql()).bind(&sql.params())?.run().await?;
        Ok(())
    }

    async fn execute_batch(&self, sqls: Vec<Sql<'_>>) -> Result<()> {
        let mut statements = Vec::with_capacity(sqls.len());
        for sql in sqls {
            statements.push(self.prepare(sql.sql()).bind(&sql.params())?);
        }
        self.batch(statements).await?;
        Ok(())
    }
}

#[async_trait(?Send)]
impl<E: DatabaseExecutor> Database for AppDatabase<E> {
    async fn migrate(&self) -> Result<()> {
        // Create schema_migrations table if not exists
        self.execute(Sql::CreateMigrationsTable).await?;

        // Get current version
        let current_version: i32 = self
            .query_first::<SchemaVersion>(Sql::GetSchemaVersion)
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

        for (version, queries) in migrations {
            if version > current_version {
                crate::log!("Applying migration version {}", version);
                let mut batch_queries = Vec::with_capacity(queries.len() + 1);
                for query in queries {
                    batch_queries.push(Sql::Raw { sql: query });
                }

                let now = utils::now_utc_ms();
                batch_queries.push(Sql::InsertMigration {
                    version,
                    applied_at: now,
                });

                self.db.execute_batch(batch_queries).await?;
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
        let sql = Sql::CreateUser {
            email,
            username,
            password_hash,
            github_id,
            telegram_id,
            avatar_url,
            created_at,
        };

        self.query_first(sql)
            .await?
            .ok_or_else(|| Error::RustError("Failed to create user".to_string()))
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        self.get_user_by_field("email", DatabaseValue::Text(email.to_string()))
            .await
    }

    async fn get_user_by_id(&self, id: i32) -> Result<Option<User>> {
        self.get_user_by_field("id", DatabaseValue::Int(id as i64))
            .await
    }

    async fn get_user_by_github_id(&self, github_id: &str) -> Result<Option<User>> {
        self.get_user_by_field("github_id", DatabaseValue::Text(github_id.to_string()))
            .await
    }

    async fn get_user_by_telegram_id(&self, telegram_id: &str) -> Result<Option<User>> {
        self.get_user_by_field("telegram_id", DatabaseValue::Text(telegram_id.to_string()))
            .await
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        self.get_user_by_field("username", DatabaseValue::Text(username.to_string()))
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
        bindings.push(DatabaseValue::Text(new_username.to_string()));

        if let Some(email) = new_email {
            updates.push("email = ?");
            bindings.push(DatabaseValue::Text(email.to_string()));
        }

        // Always update avatar_url, setting to NULL if None (explicit clear)
        updates.push("avatar_url = ?");
        bindings.push(
            new_avatar_url
                .map(|s| DatabaseValue::Text(s.to_string()))
                .unwrap_or(DatabaseValue::Null),
        );

        let sql = Sql::UpdateUserProfile {
            id,
            updates: &updates.join(", "),
            params: bindings,
        };

        self.execute(sql).await
    }

    async fn update_user_password(&self, id: i32, password_hash: &str) -> Result<()> {
        self.update_user_field(
            id,
            "password_hash",
            DatabaseValue::Text(password_hash.to_string()),
        )
        .await
    }

    async fn update_user_telegram_id(&self, id: i32, telegram_id: Option<&str>) -> Result<()> {
        let value = telegram_id
            .map(|s| DatabaseValue::Text(s.to_string()))
            .unwrap_or(DatabaseValue::Null);
        self.update_user_field(id, "telegram_id", value).await
    }

    async fn update_user_github_id(&self, id: i32, github_id: Option<&str>) -> Result<()> {
        let value = github_id
            .map(|s| DatabaseValue::Text(s.to_string()))
            .unwrap_or(DatabaseValue::Null);
        self.update_user_field(id, "github_id", value).await
    }

    async fn create_session(&self, user_id: i32, token: &str, expires_at: i64) -> Result<()> {
        let sql = Sql::CreateSession {
            user_id,
            token,
            expires_at,
        };
        self.execute(sql).await
    }

    async fn get_user_by_session_token(&self, token: &str) -> Result<Option<User>> {
        let sql = Sql::GetUserBySessionToken {
            token,
            now: utils::now_utc_ms(),
        };
        self.query_first(sql).await
    }

    async fn delete_session(&self, token: &str) -> Result<()> {
        let sql = Sql::DeleteSession { token };
        self.execute(sql).await
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
        let sql = Sql::UpdateUserItem {
            user_id,
            title,
            status: status as i32,
            score: score
                .map(|s| DatabaseValue::Int(s as i64))
                .unwrap_or(DatabaseValue::Null),
            updated_at,
            begin_at: begin_at
                .map(DatabaseValue::Int)
                .unwrap_or(DatabaseValue::Null),
        };

        self.execute(sql).await
    }

    async fn get_user_items_all(&self, user_id: i32) -> Result<Vec<UserItem>> {
        let sql = Sql::GetUserItemsAll { user_id };
        self.query_all(sql).await
    }

    async fn get_user_items_by_range(
        &self,
        user_id: i32,
        start_ts: i64,
        end_ts: i64,
    ) -> Result<Vec<UserItem>> {
        let sql = Sql::GetUserItemsByRange {
            user_id,
            start_ts,
            end_ts,
        };
        self.query_all(sql).await
    }
}
