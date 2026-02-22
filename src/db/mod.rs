use crate::db::sql::FieldUpdate;
use crate::utils;
use async_trait::async_trait;
use serde::Deserialize;
use worker::*;

pub mod d1;
pub mod models;
pub mod passkey;
pub mod sql;
#[cfg(test)]
pub mod sqlite;

pub use models::*;
pub use sql::Sql;

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
    async fn get_user(&self, filter: UserUpdate) -> Result<Option<User>>;
    async fn update_user(&self, id: i32, updates: Vec<UserUpdate>) -> Result<()>;

    async fn create_session(&self, user_id: i32, token: &str, expires_at: i64) -> Result<()>;
    async fn get_user_by_session_token(&self, filter: SessionUpdate) -> Result<Option<User>>;
    async fn delete_session(&self, token: &str) -> Result<()>;

    async fn update_user_item(
        &self,
        user_id: i32,
        title: &str,
        updates: Vec<UserItemUpdate>,
    ) -> Result<()>;
    async fn get_user_items_all(&self, user_id: i32) -> Result<Vec<UserItem>>;
    async fn get_user_items_by_range(
        &self,
        user_id: i32,
        start_ts: i64,
        end_ts: i64,
    ) -> Result<Vec<UserItem>>;
}

#[async_trait(?Send)]
pub trait DatabaseExecutor {
    async fn query_all<T>(&self, sql: Sql<'_>) -> Result<Vec<T>>
    where
        T: serde::de::DeserializeOwned;

    async fn query_first<T>(&self, sql: Sql<'_>) -> Result<Option<T>>
    where
        T: serde::de::DeserializeOwned;

    async fn execute(&self, sql: Sql<'_>) -> Result<()>;

    async fn execute_batch(&self, sqls: Vec<Sql<'_>>) -> Result<()>;
}

pub struct AppDatabase<E: DatabaseExecutor> {
    pub(crate) db: E,
}

#[derive(Debug, Deserialize)]
struct TableColumnInfo {
    name: String,
}

#[derive(Clone, Copy)]
enum MigrationStep {
    Sql(&'static str),
    AddColumnIfMissing {
        table: &'static str,
        column: &'static str,
        sql: &'static str,
    },
}

struct Migration {
    version: i32,
    steps: &'static [MigrationStep],
}

const MIGRATION_V1_STEPS: &[MigrationStep] = &[
    MigrationStep::Sql(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            email TEXT UNIQUE,
            username TEXT,
            password_hash TEXT,
            github_id TEXT UNIQUE,
            created_at INTEGER
        );",
    ),
    MigrationStep::Sql(
        "CREATE TABLE IF NOT EXISTS sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER,
            token TEXT UNIQUE,
            expires_at INTEGER,
            FOREIGN KEY(user_id) REFERENCES users(id)
        );",
    ),
    MigrationStep::Sql("CREATE UNIQUE INDEX IF NOT EXISTS idx_users_username ON users(username);"),
    MigrationStep::Sql(
        "CREATE TABLE IF NOT EXISTS user_items_v2 (
            user_id INTEGER,
            title TEXT,
            status INTEGER,
            score INTEGER,
            updated_at INTEGER,
            PRIMARY KEY (user_id, title),
            FOREIGN KEY(user_id) REFERENCES users(id)
        );",
    ),
    MigrationStep::Sql(
        "CREATE INDEX IF NOT EXISTS idx_user_items_v2_user_id ON user_items_v2(user_id);",
    ),
];

const MIGRATION_V2_STEPS: &[MigrationStep] = &[MigrationStep::AddColumnIfMissing {
    table: "users",
    column: "avatar_url",
    sql: "ALTER TABLE users ADD COLUMN avatar_url TEXT;",
}];

const MIGRATION_V3_STEPS: &[MigrationStep] = &[
    MigrationStep::Sql(
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
    ),
    MigrationStep::Sql("CREATE INDEX IF NOT EXISTS idx_passkeys_user_id ON passkeys(user_id);"),
    MigrationStep::Sql(
        "CREATE TABLE IF NOT EXISTS passkey_states (
            id TEXT PRIMARY KEY,
            state_json TEXT NOT NULL,
            expires_at INTEGER NOT NULL
        );",
    ),
];

const MIGRATION_V4_STEPS: &[MigrationStep] = &[
    MigrationStep::AddColumnIfMissing {
        table: "users",
        column: "telegram_id",
        sql: "ALTER TABLE users ADD COLUMN telegram_id TEXT;",
    },
    MigrationStep::Sql(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_users_telegram_id ON users(telegram_id);",
    ),
];

const MIGRATION_V5_STEPS: &[MigrationStep] = &[
    MigrationStep::AddColumnIfMissing {
        table: "user_items_v2",
        column: "begin_at",
        sql: "ALTER TABLE user_items_v2 ADD COLUMN begin_at INTEGER;",
    },
    MigrationStep::Sql(
        "CREATE INDEX IF NOT EXISTS idx_user_items_v2_begin_at ON user_items_v2(begin_at);",
    ),
];

const MIGRATION_V6_STEPS: &[MigrationStep] = &[MigrationStep::Sql(
    "CREATE INDEX IF NOT EXISTS idx_user_items_v2_user_id_begin_at ON user_items_v2(user_id, begin_at);",
)];

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        steps: MIGRATION_V1_STEPS,
    },
    Migration {
        version: 2,
        steps: MIGRATION_V2_STEPS,
    },
    Migration {
        version: 3,
        steps: MIGRATION_V3_STEPS,
    },
    Migration {
        version: 4,
        steps: MIGRATION_V4_STEPS,
    },
    Migration {
        version: 5,
        steps: MIGRATION_V5_STEPS,
    },
    Migration {
        version: 6,
        steps: MIGRATION_V6_STEPS,
    },
];

impl<E: DatabaseExecutor> AppDatabase<E> {
    pub fn new(db: E) -> Self {
        Self { db }
    }

    // Generic helpers delegation
    pub(crate) async fn query_all<T>(&self, sql: Sql<'_>) -> Result<Vec<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        self.db.query_all(sql).await
    }

    pub(crate) async fn query_first<T>(&self, sql: Sql<'_>) -> Result<Option<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        self.db.query_first(sql).await
    }

    pub(crate) async fn execute(&self, sql: Sql<'_>) -> Result<()> {
        self.db.execute(sql).await
    }

    pub(crate) async fn execute_batch(&self, sqls: Vec<Sql<'_>>) -> Result<()> {
        self.db.execute_batch(sqls).await
    }

    async fn has_column(&self, table: &str, column: &str) -> Result<bool> {
        let query = format!("PRAGMA table_info({table})");
        let rows: Vec<TableColumnInfo> = self.query_all(Sql::Raw { sql: &query }).await?;
        Ok(rows.iter().any(|c| c.name == column))
    }

    fn has_effective_updates<T: FieldUpdate>(updates: &[T], skipped_fields: &[&str]) -> bool {
        updates.iter().any(|u| !skipped_fields.contains(&u.field()))
    }

    async fn apply_migration(&self, migration: &Migration) -> Result<()> {
        crate::log!("Applying migration version {}", migration.version);

        let mut batch_queries = Vec::with_capacity(migration.steps.len() + 1);
        for step in migration.steps {
            match step {
                MigrationStep::Sql(sql) => batch_queries.push(Sql::Raw { sql }),
                MigrationStep::AddColumnIfMissing { table, column, sql } => {
                    if !self.has_column(table, column).await? {
                        batch_queries.push(Sql::Raw { sql });
                    }
                }
            }
        }

        let now = utils::now_utc_ms();
        batch_queries.push(Sql::InsertMigration {
            version: migration.version,
            applied_at: now,
        });

        self.db.execute_batch(batch_queries).await
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

        for migration in MIGRATIONS {
            if migration.version > current_version {
                self.apply_migration(migration).await?;
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

    async fn get_user(&self, filter: UserUpdate) -> Result<Option<User>> {
        let sql = Sql::GetUserByField { filter };
        self.query_first::<User>(sql).await
    }

    async fn update_user(&self, id: i32, updates: Vec<UserUpdate>) -> Result<()> {
        if !Self::has_effective_updates(&updates, &["id"]) {
            return Ok(());
        }
        let sql = Sql::UpdateUser { id, updates };
        self.execute(sql).await
    }

    async fn create_session(&self, user_id: i32, token: &str, expires_at: i64) -> Result<()> {
        let sql = Sql::CreateSession {
            user_id,
            token,
            expires_at,
        };
        self.execute(sql).await
    }

    async fn get_user_by_session_token(&self, filter: SessionUpdate) -> Result<Option<User>> {
        let sql = Sql::GetUserBySessionToken {
            filter,
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
        updates: Vec<crate::db::models::UserItemUpdate>,
    ) -> Result<()> {
        if !Self::has_effective_updates(&updates, &["user_id", "title"]) {
            return Ok(());
        }
        let sql = Sql::UpdateUserItem {
            user_id,
            title,
            updates,
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
