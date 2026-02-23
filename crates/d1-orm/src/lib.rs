//! # d1-orm
//!
//! A lightweight ORM and SQL builder for Cloudflare D1 and SQLite.
//!
//! ## Features
//!
//! - **Backend Agnostic**: Supports Cloudflare D1 (via `worker` crate) and SQLite (via `rusqlite`).
//! - **Type-Safe SQL Builder**: `define_sql!` macro for type-safe SQL queries with parameter binding.
//! - **Model Definition**: `define_model!` macro for defining database models and update structs.
//! - **Migration Support**: Built-in migration helpers.
//! - **Async Trait**: `DatabaseExecutor` trait for async database operations.
//!
//! ## Modules
//!
//! - `error`: Error types.
//! - `types`: Core types like `DatabaseValue`.
//! - `traits`: Core traits like `DatabaseExecutor`, `Query`.
//! - `builder`: SQL builder functions.
//! - `macros`: Helper macros.
//!
//! ## Backends
//!
//! - `d1`: Cloudflare D1 backend (requires `d1` feature).
//! - `sqlite`: SQLite backend (requires `sqlite` feature).

pub mod error;
pub mod types;
pub mod traits;
pub mod builder;
pub mod macros;

#[cfg(feature = "d1")]
pub mod d1;
#[cfg(feature = "sqlite")]
pub mod sqlite;

pub use error::Error;
pub use types::DatabaseValue;
pub use traits::{DatabaseExecutor, Query, QueryExt, SqlBackend, FieldMeta, FieldUpdate, ToParams, MigrationMeta, MigrationInfo, IntoResultCow};
pub use builder::{build_update_sql, build_upsert_sql, UpsertConfig};
