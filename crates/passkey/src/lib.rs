//! # Passkey
//!
//! A generic Rust implementation of the WebAuthn (Passkey) protocol, designed to be
//! storage-agnostic and easy to integrate into any backend.
//!
//! ## Features
//!
//! - **Generic Storage**: Implement the [`PasskeyStore`] trait to use your own database.
//! - **WASM Friendly**: Works well in WASM environments like Cloudflare Workers.
//! - **Simple Protocol Handlers**: Provides high-level functions for the standard WebAuthn registration and login flows.
//!
//! ## Example usage
//!
//! For a complete, runnable example using Axum and an in-memory store, see [`examples/basic.rs`](https://github.com/Asutorufa/housou/blob/main/crates/passkey/examples/basic.rs).
//!
//! ```rust,no_run
//! use passkey_server::{PasskeyConfig, PasskeyStore, start_registration, types::{StoredPasskey, PasskeyState}, error::Result};
//! use async_trait::async_trait;
//!
//! // Example usage requires implementing the PasskeyStore trait
//! struct MyDatabase;
//!
//! #[async_trait(?Send)]
//! impl PasskeyStore for MyDatabase {
//!     async fn create_passkey(&self, _: String, _: &str, _: &str, _: &str, _: i64, _: i64) -> Result<()> { Ok(()) }
//!     async fn get_passkey(&self, _: &str) -> Result<Option<StoredPasskey>> { Ok(None) }
//!     async fn list_passkeys(&self, _: String) -> Result<Vec<StoredPasskey>> { Ok(vec![]) }
//!     async fn delete_passkey(&self, _: String, _: &str) -> Result<()> { Ok(()) }
//!     async fn update_passkey_counter(&self, _: &str, _: i64, _: i64) -> Result<()> { Ok(()) }
//!     async fn update_passkey_name(&self, _: &str, _: &str) -> Result<()> { Ok(()) }
//!     async fn save_state(&self, _: &str, _: &str, _: i64) -> Result<()> { Ok(()) }
//!     async fn get_state(&self, _: &str) -> Result<Option<PasskeyState>> { Ok(None) }
//!     async fn delete_state(&self, _: &str) -> Result<()> { Ok(()) }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = PasskeyConfig {
//!         rp_id: "example.com".to_string(),
//!         rp_name: "My App".to_string(),
//!         origin: "https://example.com".to_string(),
//!         state_ttl: 300,
//!     };
//!
//!     let store = MyDatabase;
//!     let user_id = "123";
//!     let now_ms = 1708358400000;
//!
//!     // 1. Start registration
//!     let options = start_registration(
//!         &store,
//!         user_id,
//!         "alice",
//!         "Alice Doe",
//!         &config,
//!         now_ms
//!     ).await.unwrap();
//!
//!     println!("Send these options to the frontend: {:?}", options);
//! }
//! ```

pub mod error;
pub mod protocol;
pub mod store;
pub mod types;

pub use error::{PasskeyError, Result};
pub use protocol::{finish_login, finish_registration, start_login, start_registration};
pub use store::PasskeyStore;
pub use types::PasskeyConfig;

#[cfg(test)]
mod tests;
