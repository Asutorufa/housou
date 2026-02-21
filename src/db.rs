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
    #[d1(primary_key, auto_increment, select_by)]
    pub id: i32,
    #[d1(unique, select_by)]
    pub email: String,
    #[d1(unique, select_by)]
    pub username: String,
    #[d1(since = 2)]
    pub avatar_url: Option<String>,
    #[serde(skip_serializing)]
    pub password_hash: Option<String>,
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    #[d1(select_by)]
    pub github_id: Option<String>,
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    #[d1(since = 4, unique_index, select_by)]
    pub telegram_id: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Model)]
#[d1(table_name = "sessions")]
pub struct Session {
    #[d1(primary_key, auto_increment)]
    pub id: i32,
    pub user_id: i32,
    #[d1(select_by)]
    pub token: String,
    pub expires_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Model)]
#[d1(
    table_name = "user_items_v2",
    constraint = "PRIMARY KEY (user_id, title)",
    constraint = "FOREIGN KEY(user_id) REFERENCES users(id)"
)]
pub struct UserItem {
    #[d1(index, select_by)]
    pub user_id: i32,
    pub title: String, // Changed from item_id
    #[d1(integer)]
    pub status: UserStatus,
    pub score: Option<i32>,
    pub updated_at: i64,
    #[d1(index, since = 5)]
    pub begin_at: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserItemSummary {
    pub status: UserStatus,
    pub score: Option<i32>,
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

    fn user_items(&self) -> Repository<'_, UserItem> {
        Repository::new(&self.db)
    }
}

#[async_trait(?Send)]
impl Database for AppDatabase {
    async fn migrate(&self) -> Result<()> {
        let migrations =
            d1_orm::d1_auto_migrations!(User, Session, UserItem, PasskeyRow, PasskeyStateRow);

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
            .create_returning(&self.db)
            .await
            .map_err(|e| Error::RustError(e.to_string()))?;
        result.ok_or_else(|| Error::RustError("Failed to create user".to_string()))
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        User::get_by_email(&self.db, email)
            .await
            .map_err(|e| Error::RustError(e.to_string()))
    }

    async fn get_user_by_id(&self, id: i32) -> Result<Option<User>> {
        User::get_by_id(&self.db, id)
            .await
            .map_err(|e| Error::RustError(e.to_string()))
    }

    async fn get_user_by_github_id(&self, github_id: &str) -> Result<Option<User>> {
        User::get_by_github_id(&self.db, github_id)
            .await
            .map_err(|e| Error::RustError(e.to_string()))
    }

    async fn get_user_by_telegram_id(&self, telegram_id: &str) -> Result<Option<User>> {
        User::get_by_telegram_id(&self.db, telegram_id)
            .await
            .map_err(|e| Error::RustError(e.to_string()))
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        User::get_by_username(&self.db, username)
            .await
            .map_err(|e| Error::RustError(e.to_string()))
    }

    async fn update_user_profile(
        &self,
        id: i32,
        new_username: &str,
        new_email: Option<&str>,
        new_avatar_url: Option<&str>,
    ) -> Result<()> {
        let mut set = d1_orm::d1_sets! {
            "username" => JsValue::from_str(new_username)
        };
        if let Some(email) = new_email {
            set.push(("email", JsValue::from_str(email)));
        }
        set.push((
            "avatar_url",
            new_avatar_url.map(JsValue::from_str).unwrap_or(JsValue::NULL),
        ));
        User::update_by_id(&self.db, id, &set)
            .await
            .map(|_| ())
            .map_err(|e| Error::RustError(e.to_string()))
    }

    async fn update_user_password(&self, id: i32, password_hash: &str) -> Result<()> {
        let sets = d1_orm::d1_sets! {
            "password_hash" => JsValue::from_str(password_hash)
        };
        User::update_by_id(&self.db, id, &sets)
            .await
            .map(|_| ())
            .map_err(|e| Error::RustError(e.to_string()))
    }

    async fn update_user_telegram_id(&self, id: i32, telegram_id: Option<&str>) -> Result<()> {
        let sets = d1_orm::d1_sets! {
            "telegram_id" => telegram_id.map(JsValue::from_str).unwrap_or(JsValue::NULL)
        };
        User::update_by_id(
            &self.db,
            id,
            &sets,
        )
        .await
        .map(|_| ())
        .map_err(|e| Error::RustError(e.to_string()))
    }

    async fn update_user_github_id(&self, id: i32, github_id: Option<&str>) -> Result<()> {
        let sets = d1_orm::d1_sets! {
            "github_id" => github_id.map(JsValue::from_str).unwrap_or(JsValue::NULL)
        };
        User::update_by_id(
            &self.db,
            id,
            &sets,
        )
        .await
        .map(|_| ())
        .map_err(|e| Error::RustError(e.to_string()))
    }

    async fn create_session(&self, user_id: i32, token: &str, expires_at: i64) -> Result<()> {
        let session = Session {
            id: 0,
            user_id,
            token: token.to_string(),
            expires_at,
        };

        session
            .create(&self.db)
            .await
            .map(|_| ())
            .map_err(|e| Error::RustError(e.to_string()))
    }

    async fn get_session(&self, token: &str) -> Result<Option<Session>> {
        Session::get_by_token(&self.db, token)
            .await
            .map_err(|e| Error::RustError(e.to_string()))
    }

    async fn get_user_by_session_token(&self, token: &str) -> Result<Option<User>> {
        let now = utils::now_utc_ms();
        let query = "SELECT users.* FROM users
                     INNER JOIN sessions ON users.id = sessions.user_id
                     WHERE sessions.token = ? AND sessions.expires_at > ?
                     LIMIT 1";
        let row = self
            .db
            .prepare(query)
            .bind(&[
                JsValue::from_str(token),
                JsValue::from_f64(now as f64),
            ])?
            .first::<User>(None)
            .await
            .map_err(|e| Error::RustError(e.to_string()))?;
        Ok(row)
    }

    async fn delete_session(&self, token: &str) -> Result<()> {
        Session::delete_by_token(&self.db, token)
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
            .where_raw("status != ?", JsValue::from_f64(UserStatus::Unregistered as i32 as f64));

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
#[d1(
    table_name = "passkeys",
    since = 3,
    constraint = "FOREIGN KEY(user_id) REFERENCES users(id)"
)]
pub struct PasskeyRow {
    #[d1(index, select_by)]
    pub user_id: i32,
    #[d1(primary_key, select_by)]
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
#[d1(table_name = "passkey_states", since = 3)]
pub struct PasskeyStateRow {
    #[d1(primary_key, select_by)]
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

        let row = PasskeyRow {
            user_id: user_id_int,
            cred_id: cred_id.to_string(),
            passkey_json: public_key.to_string(),
            name: name.to_string(),
            created_at,
            last_used_at: created_at,
            counter,
        };

        row.create(&self.db)
            .await
            .map(|_| ())
            .map_err(db_err)
    }

    async fn get_passkey(
        &self,
        cred_id: &str,
    ) -> passkey_server::error::Result<Option<StoredPasskey>> {
        let row = PasskeyRow::get_by_cred_id(&self.db, cred_id)
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

        let rows = PasskeyRow::list_by_user_id(&self.db, user_id_int)
            .await
            .map_err(db_err)?;
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

        match PasskeyRow::get_by_cred_id(&self.db, cred_id)
            .await
            .map_err(db_err)?
        {
            Some(row) if row.user_id == user_id_int => PasskeyRow::delete_by_cred_id(&self.db, cred_id)
                .await
                .map_err(db_err),
            Some(_) | None => Ok(()),
        }
    }

    async fn update_passkey_counter(
        &self,
        cred_id: &str,
        new_counter: i64,
        last_used_at: i64,
    ) -> passkey_server::error::Result<()> {
        let sets = d1_orm::d1_sets! {
            "counter" => JsValue::from_f64(new_counter as f64),
            "last_used_at" => JsValue::from_f64(last_used_at as f64)
        };
        PasskeyRow::update_by_cred_id(
            &self.db,
            cred_id,
            &sets,
        )
        .await
        .map(|_| ())
        .map_err(db_err)
    }

    async fn update_passkey_name(
        &self,
        cred_id: &str,
        new_name: &str,
    ) -> passkey_server::error::Result<()> {
        let sets = d1_orm::d1_sets! {
            "name" => JsValue::from_str(new_name)
        };
        PasskeyRow::update_by_cred_id(&self.db, cred_id, &sets)
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

        let delete = self
            .passkey_states()
            .delete()
            .where_lt("expires_at", now as f64);
        let insert = self
            .passkey_states()
            .insert()
            .set("id", id)
            .set("state_json", state_json)
            .set("expires_at", expires_at as f64)
            .on_conflict(
                "(id) DO UPDATE SET state_json=excluded.state_json, expires_at=excluded.expires_at",
            );
        d1_orm::d1_exec_batch!(&self.db, [delete, insert]).map_err(db_err)
    }

    async fn get_state(&self, id: &str) -> passkey_server::error::Result<Option<PasskeyState>> {
        let now = utils::now_utc_ms();
        let row = PasskeyStateRow::get_by_id(&self.db, id)
            .await
            .map_err(db_err)?;
        Ok(row.filter(|r| r.expires_at > now).map(Into::into))
    }

    async fn delete_state(&self, id: &str) -> passkey_server::error::Result<()> {
        PasskeyStateRow::delete_by_id(&self.db, id)
            .await
            .map_err(db_err)
    }
}

// Add helpers for repositories
impl AppDatabase {
    fn passkey_states(&self) -> Repository<'_, PasskeyStateRow> {
        Repository::new(&self.db)
    }
}
