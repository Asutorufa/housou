use crate::db::{AppDatabase, DatabaseExecutor, Sql};
use crate::utils;
use async_trait::async_trait;
use passkey_server::types::{PasskeyState, StoredPasskey};
use passkey_server::{PasskeyError, PasskeyStore};
use serde_derive::Deserialize;
use worker::*;

/// Helper for deserializing DB rows into StoredPasskey.
/// The DB column is "passkey_json" but our struct field is "public_key".
#[derive(Debug, Deserialize)]
pub(crate) struct PasskeyRow {
    pub user_id: i32,
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

#[derive(Debug, Deserialize)]
pub(crate) struct PasskeyStateRow {
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

pub(crate) fn db_err(e: impl std::fmt::Display) -> PasskeyError {
    PasskeyError::DatabaseError(e.to_string())
}

#[cfg_attr(not(feature = "send"), async_trait(?Send))]
#[cfg_attr(feature = "send", async_trait)]
impl<E: DatabaseExecutor> PasskeyStore for AppDatabase<E> {
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
        let sql = Sql::CreatePasskey {
            user_id: user_id_int,
            cred_id,
            passkey_json: public_key,
            name,
            created_at,
            last_used_at: created_at,
            counter,
        };
        self.execute(sql).await.map_err(db_err)
    }

    async fn get_passkey(
        &self,
        cred_id: &str,
    ) -> passkey_server::error::Result<Option<StoredPasskey>> {
        let sql = Sql::GetPasskey { cred_id };
        let row: Option<PasskeyRow> = self.query_first(sql).await.map_err(db_err)?;
        Ok(row.map(Into::into))
    }

    async fn list_passkeys(
        &self,
        user_id: String,
    ) -> passkey_server::error::Result<Vec<StoredPasskey>> {
        let user_id_int = user_id
            .parse::<i32>()
            .map_err(|_| PasskeyError::InternalError("Invalid user ID".into()))?;
        let sql = Sql::ListPasskeys {
            user_id: user_id_int,
        };
        let rows: Vec<PasskeyRow> = self.query_all(sql).await.map_err(db_err)?;
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
        let sql = Sql::DeletePasskey {
            user_id: user_id_int,
            cred_id,
        };
        self.execute(sql).await.map_err(db_err)
    }

    async fn update_passkey_counter(
        &self,
        cred_id: &str,
        new_counter: i64,
        last_used_at: i64,
    ) -> passkey_server::error::Result<()> {
        let sql = Sql::UpdatePasskeyCounter {
            cred_id,
            counter: new_counter,
            last_used_at,
        };
        self.execute(sql).await.map_err(db_err)
    }

    async fn update_passkey_name(
        &self,
        cred_id: &str,
        new_name: &str,
    ) -> passkey_server::error::Result<()> {
        let sql = Sql::UpdatePasskeyName {
            cred_id,
            name: new_name,
        };
        self.execute(sql).await.map_err(db_err)
    }

    async fn save_state(
        &self,
        id: &str,
        state_json: &str,
        expires_at: i64,
    ) -> passkey_server::error::Result<()> {
        let now = utils::now_utc_ms();
        self.execute_batch(vec![
            Sql::CleanupPasskeyStates { now },
            Sql::SavePasskeyState {
                id,
                state_json,
                expires_at,
            },
        ])
        .await
        .map_err(db_err)
    }

    async fn get_state(&self, id: &str) -> passkey_server::error::Result<Option<PasskeyState>> {
        let sql = Sql::GetPasskeyState {
            id,
            now: utils::now_utc_ms(),
        };
        let row: Option<PasskeyStateRow> = self.query_first(sql).await.map_err(db_err)?;
        Ok(row.map(Into::into))
    }

    async fn delete_state(&self, id: &str) -> passkey_server::error::Result<()> {
        let sql = Sql::DeletePasskeyState { id };
        self.execute(sql).await.map_err(db_err)
    }
}
