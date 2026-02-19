use async_trait::async_trait;
use crate::types::{StoredPasskey, PasskeyState};
use crate::error::Result;

#[async_trait(?Send)]
pub trait PasskeyStore {
    // Credential CRUD
    async fn create_passkey(
        &self,
        user_id: i32,
        cred_id: &str,
        public_key: &str,
        name: &str,
        counter: i64,
        created_at: i64,
    ) -> Result<()>;
    async fn get_passkey(&self, cred_id: &str) -> Result<Option<StoredPasskey>>;
    async fn list_passkeys(&self, user_id: i32) -> Result<Vec<StoredPasskey>>;
    async fn delete_passkey(&self, user_id: i32, cred_id: &str) -> Result<()>;
    async fn update_passkey_counter(
        &self,
        cred_id: &str,
        new_counter: i64,
        last_used_at: i64,
    ) -> Result<()>;
    async fn update_passkey_name(&self, cred_id: &str, new_name: &str) -> Result<()>;

    // Ephemeral state (challenge ↔ session)
    async fn save_state(&self, id: &str, state_json: &str, expires_at: i64) -> Result<()>;
    async fn get_state(&self, id: &str) -> Result<Option<PasskeyState>>;
    async fn delete_state(&self, id: &str) -> Result<()>;
}
