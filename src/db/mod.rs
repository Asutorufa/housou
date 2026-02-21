use crate::db::sql::DatabaseValue;
use async_trait::async_trait;
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
    async fn get_user_by_session_token(&self, token: &str) -> Result<Option<User>>;
    async fn delete_session(&self, token: &str) -> Result<()>;

    async fn update_user_item(
        &self,
        user_id: i32,
        title: &str,
        status: crate::model::UserStatus,
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

impl<E: DatabaseExecutor> AppDatabase<E> {
    pub fn new(db: E) -> Self {
        Self { db }
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

    pub(crate) async fn get_user_by_field(
        &self,
        field: &str,
        value: DatabaseValue,
    ) -> Result<Option<User>> {
        Self::validate_select_field(field)?;
        let sql = Sql::GetUserByField { field, value };
        self.query_first::<User>(sql).await
    }

    pub(crate) async fn update_user_field(
        &self,
        id: i32,
        field: &str,
        value: DatabaseValue,
    ) -> Result<()> {
        Self::validate_update_field(field)?;
        let sql = Sql::UpdateUserField { id, field, value };
        self.execute(sql).await
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_select_field() {
        // Allowed fields
        assert!(AppDatabase::<worker::D1Database>::validate_select_field("id").is_ok());
        assert!(AppDatabase::<worker::D1Database>::validate_select_field("email").is_ok());
        assert!(AppDatabase::<worker::D1Database>::validate_select_field("username").is_ok());
        assert!(AppDatabase::<worker::D1Database>::validate_select_field("github_id").is_ok());
        assert!(AppDatabase::<worker::D1Database>::validate_select_field("telegram_id").is_ok());

        // Disallowed fields
        assert!(AppDatabase::<worker::D1Database>::validate_select_field("password_hash").is_err());
        assert!(AppDatabase::<worker::D1Database>::validate_select_field("created_at").is_err());
        assert!(AppDatabase::<worker::D1Database>::validate_select_field("avatar_url").is_err());
        assert!(
            AppDatabase::<worker::D1Database>::validate_select_field("1; DROP TABLE users")
                .is_err()
        );
    }

    #[test]
    fn test_validate_update_field() {
        // Allowed fields
        assert!(AppDatabase::<worker::D1Database>::validate_update_field("password_hash").is_ok());
        assert!(AppDatabase::<worker::D1Database>::validate_update_field("telegram_id").is_ok());
        assert!(AppDatabase::<worker::D1Database>::validate_update_field("github_id").is_ok());

        // Disallowed fields
        assert!(AppDatabase::<worker::D1Database>::validate_update_field("email").is_err());
        assert!(AppDatabase::<worker::D1Database>::validate_update_field("username").is_err());
        assert!(AppDatabase::<worker::D1Database>::validate_update_field("id").is_err());
        assert!(AppDatabase::<worker::D1Database>::validate_update_field("avatar_url").is_err());
        assert!(
            AppDatabase::<worker::D1Database>::validate_update_field("1; DROP TABLE users")
                .is_err()
        );
    }
}
