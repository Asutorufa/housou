use crate::model::UserStatus;
use crate::utils;
use async_trait::async_trait;
use d1_orm::{Bindable, D1Database, Migrator, Model, Repository};
use passkey_server::types::{PasskeyState, StoredPasskey};
use passkey_server::{PasskeyError, PasskeyStore};

use serde_derive::{Deserialize, Serialize};
use worker::wasm_bindgen::JsValue;
use worker::*;

#[derive(Debug, Serialize, Deserialize, Clone, Model)]
#[d1(table_name = "users")]
pub struct User {
    #[d1(primary_key, auto_increment)]
    pub id: i32,
    #[d1(unique)]
    pub email: String,
    #[d1(unique)]
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

#[derive(Debug, Serialize, Deserialize, Clone, Model)]
#[d1(table_name = "sessions")]
pub struct Session {
    #[d1(primary_key, auto_increment)]
    pub id: i32,
    pub user_id: i32,
    pub token: String,
    pub expires_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Model)]
#[d1(table_name = "user_items_v2")]
pub struct UserItem {
    #[d1(index)]
    pub user_id: i32,
    pub title: String, // Changed from item_id
    pub status: UserStatus,
    pub score: Option<i32>,
    pub updated_at: i64,
    #[d1(index)]
    pub begin_at: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserItemSummary {
    pub status: UserStatus,
    pub score: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Model)]
#[d1(table_name = "users")]
struct UserV1 {
    #[d1(primary_key, auto_increment)]
    pub id: i32,
    #[d1(unique)]
    pub email: String,
    #[d1(unique)]
    pub username: String,
    pub password_hash: Option<String>,
    pub github_id: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Model)]
#[d1(table_name = "users")]
struct UserV2 {
    #[d1(primary_key, auto_increment)]
    pub id: i32,
    #[d1(unique)]
    pub email: String,
    #[d1(unique)]
    pub username: String,
    pub password_hash: Option<String>,
    pub github_id: Option<String>,
    pub created_at: i64,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Model)]
#[d1(table_name = "users")]
struct UserV4 {
    #[d1(primary_key, auto_increment)]
    pub id: i32,
    #[d1(unique)]
    pub email: String,
    #[d1(unique)]
    pub username: String,
    pub password_hash: Option<String>,
    pub github_id: Option<String>,
    pub created_at: i64,
    pub avatar_url: Option<String>,
    pub telegram_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Model)]
#[d1(table_name = "sessions")]
struct SessionV1 {
    #[d1(primary_key, auto_increment)]
    pub id: i32,
    pub user_id: i32,
    #[d1(unique)]
    pub token: String,
    pub expires_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Model)]
#[d1(
    table_name = "user_items_v2",
    constraint = "PRIMARY KEY (user_id, title)",
    constraint = "FOREIGN KEY(user_id) REFERENCES users(id)"
)]
struct UserItemV1 {
    #[d1(index)]
    pub user_id: i32,
    pub title: String,
    pub status: i32,
    pub score: Option<i32>,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Model)]
#[d1(
    table_name = "user_items_v2",
    constraint = "PRIMARY KEY (user_id, title)",
    constraint = "FOREIGN KEY(user_id) REFERENCES users(id)"
)]
struct UserItemV5 {
    #[d1(index)]
    pub user_id: i32,
    pub title: String,
    pub status: i32,
    pub score: Option<i32>,
    pub updated_at: i64,
    #[d1(index)]
    pub begin_at: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Model)]
#[d1(
    table_name = "passkeys",
    constraint = "FOREIGN KEY(user_id) REFERENCES users(id)"
)]
struct PasskeyV3 {
    #[d1(index)]
    pub user_id: i32,
    #[d1(primary_key)]
    pub cred_id: String,
    pub passkey_json: String,
    pub name: String,
    pub created_at: i64,
    pub last_used_at: i64,
    pub counter: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Model)]
#[d1(table_name = "passkey_states")]
struct PasskeyStateV3 {
    #[d1(primary_key)]
    pub id: String,
    pub state_json: String,
    pub expires_at: i64,
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

    async fn get_user_by_field(&self, field: &str, value: JsValue) -> Result<Option<User>> {
        Self::validate_select_field(field)?;
        // Use ORM
        self.users()
            .find_one(self.users().select().where_eq(field, value))
            .await
            .map_err(|e| Error::RustError(e.to_string()))
    }

    fn migration_v1_sql() -> Vec<String> {
        let mut stmts = vec![
            UserV1::schema().to_sql(),
            SessionV1::schema().to_sql(),
            UserItemV1::schema().to_sql(),
        ];
        stmts.extend(UserItemV1::indexes().into_iter().map(|idx| idx.to_sql()));
        stmts
    }

    fn migration_v3_sql() -> Vec<String> {
        let mut stmts = vec![
            PasskeyV3::schema().to_sql(),
            PasskeyStateV3::schema().to_sql(),
        ];
        stmts.extend(PasskeyV3::indexes().into_iter().map(|idx| idx.to_sql()));
        stmts
    }

    fn migration_v2_sql() -> Vec<String> {
        d1_orm::additive_migration_sql(
            &UserV1::schema(),
            &UserV2::schema(),
            &UserV1::indexes(),
            &UserV2::indexes(),
        )
    }

    fn migration_v4_sql() -> Vec<String> {
        let mut sql = d1_orm::additive_migration_sql(
            &UserV2::schema(),
            &UserV4::schema(),
            &UserV2::indexes(),
            &UserV4::indexes(),
        );
        sql.push(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_users_telegram_id ON users(telegram_id)"
                .to_string(),
        );
        sql
    }

    fn migration_v5_sql() -> Vec<String> {
        d1_orm::additive_migration_sql(
            &UserItemV1::schema(),
            &UserItemV5::schema(),
            &UserItemV1::indexes(),
            &UserItemV5::indexes(),
        )
    }
}

#[async_trait(?Send)]
impl Database for AppDatabase {
    async fn migrate(&self) -> Result<()> {
        let migrations = d1_orm::d1_migrations![
            d1_orm::d1_migration!(
                1,
                sqls = Self::migration_v1_sql(),
                infer = [
                    d1_orm::d1_probe!(table "users"),
                    d1_orm::d1_probe!(table "sessions"),
                    d1_orm::d1_probe!(table "user_items_v2")
                ]
            ),
            d1_orm::d1_migration!(
                2,
                sqls = Self::migration_v2_sql(),
                infer = [d1_orm::d1_probe!(column "users", "avatar_url")]
            ),
            d1_orm::d1_migration!(
                3,
                sqls = Self::migration_v3_sql(),
                infer = [
                    d1_orm::d1_probe!(table "passkeys"),
                    d1_orm::d1_probe!(table "passkey_states")
                ]
            ),
            d1_orm::d1_migration!(
                4,
                sqls = Self::migration_v4_sql(),
                infer = [
                    d1_orm::d1_probe!(column "users", "telegram_id"),
                    d1_orm::d1_probe!(index "idx_users_telegram_id")
                ]
            ),
            d1_orm::d1_migration!(
                5,
                sqls = Self::migration_v5_sql(),
                infer = [
                    d1_orm::d1_probe!(column "user_items_v2", "begin_at"),
                    d1_orm::d1_probe!(index "idx_user_items_v2_begin_at")
                ]
            ),
        ];

        Migrator::new(&self.db)
            .run(&migrations, utils::now_utc_ms())
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
        let user = User {
            id: 0,
            email: email.to_string(),
            username: username.to_string(),
            avatar_url: avatar_url.map(str::to_string),
            password_hash: password_hash.map(str::to_string),
            github_id: github_id.map(str::to_string),
            telegram_id: telegram_id.map(str::to_string),
            created_at,
        };

        let result = user
            .insert_returning(&self.db)
            .await
            .map_err(|e| Error::RustError(e.to_string()))?;
        result.ok_or_else(|| Error::RustError("Failed to create user".to_string()))
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        self.get_user_by_field("email", JsValue::from_str(email))
            .await
    }

    async fn get_user_by_id(&self, id: i32) -> Result<Option<User>> {
        User::find_by_pk(&self.db, id)
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
        if let Some(mut user) = self.get_user_by_id(id).await? {
            user.username = new_username.to_string();
            user.avatar_url = new_avatar_url.map(str::to_string);
            if let Some(email) = new_email {
                user.email = email.to_string();
            }

            user.update(&self.db)
                .await
                .map(|_| ())
                .map_err(|e| Error::RustError(e.to_string()))?;
        }

        Ok(())
    }

    async fn update_user_password(&self, id: i32, password_hash: &str) -> Result<()> {
        if let Some(mut user) = self.get_user_by_id(id).await? {
            user.password_hash = Some(password_hash.to_string());
            user.update(&self.db)
                .await
                .map(|_| ())
                .map_err(|e| Error::RustError(e.to_string()))?;
        }
        Ok(())
    }

    async fn update_user_telegram_id(&self, id: i32, telegram_id: Option<&str>) -> Result<()> {
        if let Some(mut user) = self.get_user_by_id(id).await? {
            user.telegram_id = telegram_id.map(str::to_string);
            user.update(&self.db)
                .await
                .map(|_| ())
                .map_err(|e| Error::RustError(e.to_string()))?;
        }
        Ok(())
    }

    async fn update_user_github_id(&self, id: i32, github_id: Option<&str>) -> Result<()> {
        if let Some(mut user) = self.get_user_by_id(id).await? {
            user.github_id = github_id.map(str::to_string);
            user.update(&self.db)
                .await
                .map(|_| ())
                .map_err(|e| Error::RustError(e.to_string()))?;
        }
        Ok(())
    }

    async fn create_session(&self, user_id: i32, token: &str, expires_at: i64) -> Result<()> {
        let session = Session {
            id: 0,
            user_id,
            token: token.to_string(),
            expires_at,
        };

        session
            .insert(&self.db)
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
#[derive(Debug, Serialize, Deserialize, Model)]
#[d1(table_name = "passkeys")]
pub struct PasskeyRow {
    #[d1(index)]
    pub user_id: i32,
    #[d1(primary_key)]
    pub cred_id: String,
    pub passkey_json: String,
    pub name: String,
    pub created_at: i64,
    pub last_used_at: i64,
    pub counter: i64,
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

#[derive(Debug, Serialize, Deserialize, Clone, Model)]
#[d1(table_name = "passkey_states")]
pub struct PasskeyStateRow {
    #[d1(primary_key)]
    pub id: String,
    pub state_json: String,
    pub expires_at: i64,
}

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
}
