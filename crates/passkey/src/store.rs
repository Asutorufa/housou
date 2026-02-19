use crate::error::Result;
use crate::types::{PasskeyState, StoredPasskey};
use async_trait::async_trait;

/// A trait for managing the persistence of passkeys and authentication state.
///
/// Implement this trait to connect the `passkey` library to your chosen database.
/// All methods are `async` and use `async_trait(?Send)` for compatibility with WASM.
#[cfg_attr(not(feature = "send"), async_trait(?Send))]
#[cfg_attr(feature = "send", async_trait)]
pub trait PasskeyStore {
    /// Save a new passkey credential to the database.
    ///
    /// - `user_id`: The ID of the owner.
    /// - `cred_id`: The unique identifier for this credential (Base64url).
    /// - `public_key`: The public key material (Base64url-encoded COSE key).
    /// - `name`: A human-readable name for the passkey.
    /// - `counter`: The initial signature counter.
    /// - `created_at`: Creation timestamp in milliseconds.
    async fn create_passkey(
        &self,
        user_id: String,
        cred_id: &str,
        public_key: &str,
        name: &str,
        counter: i64,
        created_at: i64,
    ) -> Result<()>;

    /// Retrieve a passkey by its credential ID.
    async fn get_passkey(&self, cred_id: &str) -> Result<Option<StoredPasskey>>;

    /// List all passkeys associated with a specific user.
    async fn list_passkeys(&self, user_id: String) -> Result<Vec<StoredPasskey>>;

    /// Delete a passkey. Implementations should verify that `user_id` owns the `cred_id`.
    async fn delete_passkey(&self, user_id: String, cred_id: &str) -> Result<()>;

    /// Update the signature counter and last-used timestamp after a successful login.
    async fn update_passkey_counter(
        &self,
        cred_id: &str,
        new_counter: i64,
        last_used_at: i64,
    ) -> Result<()>;

    /// Rename an existing passkey.
    async fn update_passkey_name(&self, cred_id: &str, new_name: &str) -> Result<()>;

    /// Save ephemeral registration or login state (challenges).
    ///
    /// The `state_json` contains internal session data and should be retrievable by `id`.
    /// `expires_at` is the expiration timestamp in milliseconds.
    async fn save_state(&self, id: &str, state_json: &str, expires_at: i64) -> Result<()>;

    /// Retrieve ephemeral state by its ID. It should return `None` if expired.
    async fn get_state(&self, id: &str) -> Result<Option<PasskeyState>>;

    /// Delete ephemeral state (e.g., after the session is finished or fails).
    async fn delete_state(&self, id: &str) -> Result<()>;
}
