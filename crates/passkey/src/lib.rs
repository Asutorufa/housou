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
//! ```rust,ignore
//! use passkey_server::{PasskeyConfig, PasskeyStore, start_registration};
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = PasskeyConfig {
//!         rp_id: "example.com".to_string(),
//!         rp_name: "My App".to_string(),
//!         origin: "https://example.com".to_string(),
//!     };
//!
//!     // Your implementation of PasskeyStore
//!     let store = MyDatabase::new();
//!     let user_id = 123;
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
