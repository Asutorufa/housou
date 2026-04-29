use crate::utils;
use async_trait::async_trait;
pub use d1_orm::{DatabaseExecutor, FieldUpdate, Migration};
use worker::*;

pub mod models;
pub mod passkey;
pub mod sql;

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

    async fn get_comments(
        &self,
        title: &str,
        viewer_id: Option<i32>,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<CommentWithUser>>;
    async fn get_comments_count(&self, title: &str) -> Result<i32>;
    async fn create_comment(
        &self,
        user_id: i32,
        title: &str,
        content: &str,
        score: Option<i32>,
    ) -> Result<Comment>;
    async fn update_comment(
        &self,
        user_id: i32,
        title: &str,
        content: &str,
        score: Option<i32>,
    ) -> Result<Comment>;
    async fn delete_comment(&self, id: i32, user_id: i32) -> Result<()>;
}

pub struct AppDatabase<E: DatabaseExecutor> {
    pub(crate) db: E,
}

fn get_migrations() -> Vec<Migration<Sql<'static>>> {
    vec![
        Migration::new(
            1,
            "Initial schema",
            vec![
                Sql::CreateUsersTable,
                Sql::CreateSessionsTable,
                Sql::CreateUsersUsernameIndex,
                Sql::CreateUserItemsV2Table,
                Sql::CreateUserItemsV2UserIdIndex,
            ],
        ),
        Migration::new(2, "Add avatar_url", vec![Sql::AddUsersAvatarUrlColumn]),
        Migration::new(
            3,
            "Add passkeys",
            vec![
                Sql::CreatePasskeysTable,
                Sql::CreatePasskeysUserIdIndex,
                Sql::CreatePasskeyStatesTable,
            ],
        ),
        Migration::new(
            4,
            "Add telegram_id",
            vec![
                Sql::AddUsersTelegramIdColumn,
                Sql::CreateUsersTelegramIdIndex,
            ],
        ),
        Migration::new(
            5,
            "Add begin_at",
            vec![
                Sql::AddUserItemsV2BeginAtColumn,
                Sql::CreateUserItemsV2BeginAtIndex,
            ],
        ),
        Migration::new(
            6,
            "Add composite index",
            vec![Sql::CreateUserItemsV2UserIdBeginAtIndex],
        ),
        Migration::new(
            7,
            "Add comments",
            vec![Sql::CreateCommentsTable, Sql::CreateCommentsTitleIndex],
        ),
        Migration::new(
            8,
            "Add score to comments",
            vec![Sql::AddCommentsScoreColumn],
        ),
        Migration::new(
            9,
            "Add updated_at to comments",
            vec![
                Sql::AddCommentsUpdatedAtColumn,
                Sql::BackfillCommentsUpdatedAt,
            ],
        ),
    ]
}

impl<E: DatabaseExecutor> AppDatabase<E> {
    pub fn new(db: E) -> Self {
        Self { db }
    }

    // Generic helpers delegation
    pub(crate) async fn query_all<T>(&self, sql: Sql<'_>) -> Result<Vec<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        self.db
            .query_all(sql)
            .await
            .map_err(|e| Error::RustError(e.to_string()))
    }

    pub(crate) async fn query_first<T>(&self, sql: Sql<'_>) -> Result<Option<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        self.db
            .query_first(sql)
            .await
            .map_err(|e| Error::RustError(e.to_string()))
    }

    pub(crate) async fn execute(&self, sql: Sql<'_>) -> Result<()> {
        self.db
            .execute(sql)
            .await
            .map_err(|e| Error::RustError(e.to_string()))
    }

    pub(crate) async fn execute_batch(&self, sqls: Vec<Sql<'_>>) -> Result<()> {
        self.db
            .execute_batch(sqls)
            .await
            .map_err(|e| Error::RustError(e.to_string()))
    }

    fn has_effective_updates<T: FieldUpdate>(updates: &[T], skipped_fields: &[&str]) -> bool {
        updates.iter().any(|u| !skipped_fields.contains(&u.field()))
    }
}

#[async_trait(?Send)]
impl<E: DatabaseExecutor> Database for AppDatabase<E> {
    async fn migrate(&self) -> Result<()> {
        let migrations = get_migrations();
        d1_orm::migrate(
            &self.db,
            migrations,
            Some("schema_migrations"),
            Some(|msg: &str| {
                crate::log!("{}", msg);
            }),
        )
        .await
        .map_err(|e| Error::RustError(e.to_string()))
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

    async fn get_comments(
        &self,
        title: &str,
        viewer_id: Option<i32>,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<CommentWithUser>> {
        if let Some(viewer_id) = viewer_id {
            let sql = Sql::GetCommentsWithUser {
                title,
                viewer_id,
                limit,
                offset,
            };
            self.query_all(sql).await
        } else {
            let sql = Sql::GetCommentsWithUserGuest {
                title,
                limit,
                offset,
            };
            self.query_all(sql).await
        }
    }

    async fn get_comments_count(&self, title: &str) -> Result<i32> {
        let sql = Sql::GetCommentsCount { title };
        let res: Option<SchemaVersion> = self.query_first(sql).await?;
        Ok(res.and_then(|r| r.version).unwrap_or(0))
    }

    async fn create_comment(
        &self,
        user_id: i32,
        title: &str,
        content: &str,
        score: Option<i32>,
    ) -> Result<Comment> {
        let created_at = utils::now_utc_ms();
        let sql = Sql::CreateComment {
            user_id,
            title,
            content,
            score,
            created_at,
            updated_at: created_at,
        };
        self.query_first(sql)
            .await?
            .ok_or_else(|| Error::RustError("Failed to create comment".to_string()))
    }

    async fn update_comment(
        &self,
        user_id: i32,
        title: &str,
        content: &str,
        score: Option<i32>,
    ) -> Result<Comment> {
        let updated_at = utils::now_utc_ms();
        let sql = Sql::UpdateComment {
            content,
            score,
            updated_at,
            user_id,
            title,
        };
        self.query_first(sql)
            .await?
            .ok_or_else(|| Error::RustError("Failed to update comment".to_string()))
    }

    async fn delete_comment(&self, id: i32, user_id: i32) -> Result<()> {
        let sql = Sql::DeleteComment { id, user_id };
        self.execute(sql).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use d1_orm::MigrationInfo;
    use d1_orm::sqlite::SqliteExecutor;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct TableColumnInfo {
        name: String,
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_migrations_and_basic_workflow() -> Result<()> {
        let executor =
            SqliteExecutor::new_in_memory().map_err(|e| Error::RustError(e.to_string()))?;
        let db = AppDatabase::new(executor);

        db.migrate().await?;

        let user = db
            .create_user("test@example.com", "testuser", None, None, None, None)
            .await?;
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");

        let fetched = db.get_user(UserUpdate::id(user.id)).await?.unwrap();
        assert_eq!(fetched.id, user.id);

        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_sqlite_workflow_extended() -> Result<()> {
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
                "extended@example.com",
                "extendeduser",
                Some("hash"),
                None,
                None,
                None,
            )
            .await?;

        assert_eq!(user.email, "extended@example.com");
        assert_eq!(user.username, "extendeduser");

        // Get user by field
        let user_by_email = db
            .get_user(UserUpdate::email("extended@example.com".to_string()))
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
                UserItemUpdate::status(crate::model::UserStatus::Completed),
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

        // Comments
        let comment = db
            .create_comment(user.id, "Anime Title", "Great show!", Some(8))
            .await?;
        assert_eq!(comment.content, "Great show!");
        assert_eq!(comment.user_id, user.id);
        assert_eq!(comment.score, Some(8));
        assert_eq!(comment.created_at, comment.updated_at);

        let comments = db.get_comments("Anime Title", Some(user.id), 10, 0).await?;
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].content, "Great show!");
        assert_eq!(comments[0].username, "extendeduser");
        assert_eq!(comments[0].updated_at, comment.updated_at);

        let updated_comment = db
            .update_comment(user.id, "Anime Title", "Masterpiece!", Some(10))
            .await?;
        assert_eq!(updated_comment.content, "Masterpiece!");
        assert_eq!(updated_comment.score, Some(10));
        assert_eq!(updated_comment.created_at, comment.created_at);
        assert!(updated_comment.updated_at >= comment.updated_at);

        let count = db.get_comments_count("Anime Title").await?;
        assert_eq!(count, 1);

        db.delete_comment(comment.id, user.id).await?;
        let count_after = db.get_comments_count("Anime Title").await?;
        assert_eq!(count_after, 0);

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
    async fn test_migration_idempotent_when_records_missing() -> Result<()> {
        let executor =
            SqliteExecutor::new_in_memory().map_err(|e| Error::RustError(e.to_string()))?;
        let db = AppDatabase::new(executor);

        // Run migrations normally
        db.migrate()
            .await
            .map_err(|e| Error::RustError(e.to_string()))?;

        // Drop the schema_migrations table, but keep the created tables
        db.execute(Sql::AdHoc {
            info: MigrationInfo::Table("schema_migrations"),
            sql: std::borrow::Cow::Borrowed("DROP TABLE schema_migrations;"),
        })
        .await
        .map_err(|e| Error::RustError(e.to_string()))?;

        // Run migrations again
        // It should recreate schema_migrations and perform idempotency checks
        // skipping tables/indexes/columns that already exist.
        db.migrate()
            .await
            .map_err(|e| Error::RustError(e.to_string()))?;

        // Verify that tables still exist and are accessible
        let tables: Vec<TableColumnInfo> = db
            .query_all(Sql::CheckTableExists { name: "users" })
            .await
            .unwrap();
        assert!(!tables.is_empty());
        assert_eq!(tables[0].name, "users");

        Ok(())
    }
}
