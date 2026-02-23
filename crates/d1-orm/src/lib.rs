//! # d1-orm
//!
//! A lightweight ORM and SQL builder for Rust, perfectly suited for Cloudflare D1 and SQLite.
//!
//! ## Features
//!
//! - **Backend Agnostic**: Bring your own database driver or use the built-in integrations for Cloudflare D1 (via `worker` crate) and SQLite (via `rusqlite`).
//! - **Type-Safe SQL Builder**: `define_sql!` macro for type-safe parameter binding and zero-overhead declarative SQL queries.
//! - **Model Definition**: `define_model!` macro for defining structs, mapping database models, and generating update structs automatically.
//! - **WASM Compatible**: Works flawlessly within WASM targets like Cloudflare Workers (non-Send environments).
//! - **Async Trait**: Implements an ecosystem agnostic `DatabaseExecutor` trait for async database operations.
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
//!
//! ## Example usage
//!
//! For a complete, runnable example using an in-memory SQL store, see [`examples/basic.rs`](https://github.com/Asutorufa/housou/blob/main/crates/d1-orm/examples/basic.rs).
//!
//! ```rust,no_run
//! use d1_orm::sqlite::SqliteExecutor;
//! use d1_orm::{build_update_sql, define_model, define_sql, DatabaseExecutor};
//!
//! // 1. Define your model and update struct
//! define_model!(
//!     /// A user in the system
//!     User,
//!     UserField,
//!     UserUpdate {
//!         id: i32 [pk],
//!         username: String,
//!         email: String,
//!     }
//! );
//!
//! // 2. Define your SQL queries
//! define_sql!(
//!     MySql
//!     
//!     // Simple parameterized query
//!     GetUser { id: i32 } => "SELECT * FROM users WHERE id = ?",
//!     
//!     // Insert query with multiple parameters
//!     CreateUser { username: &'a str, email: &'a str } =>
//!         "INSERT INTO users (username, email) VALUES (?, ?)",
//!         
//!     // Dynamic update query using the generated UserUpdate enum
//!     UpdateUser { updates: Vec<UserUpdate> [skip_primary_key], id: i32 } =>
//!         build_update_sql("users", "id", &updates),
//! );
//!
//! #[tokio::main]
//! async fn main() -> Result<(), d1_orm::Error> {
//!     // 3. Initialize the database backend (SQLite in this case)
//!     let conn = rusqlite::Connection::open_in_memory().unwrap();
//!     
//!     conn.execute(
//!         "CREATE TABLE users (id INTEGER PRIMARY KEY, username TEXT NOT NULL, email TEXT NOT NULL)",
//!         [],
//!     ).unwrap();
//!     
//!     let executor = SqliteExecutor::new(conn);
//!
//!     // 4. Execute queries using the type-safe builder
//!     executor.execute(MySql::CreateUser {
//!         username: "alice",
//!         email: "alice@example.com",
//!     }).await?;
//!
//!     // 5. Query results into strongly-typed models
//!     let user: Option<User> = executor.query_first(MySql::GetUser { id: 1 }).await?;
//!     println!("Found user: {:?}", user);
//!     
//!     // 6. Dynamic Updates programmatically crafted
//!     executor.execute(MySql::UpdateUser {
//!         updates: vec![UserUpdate::email("alice.new@example.com".to_string())],
//!         id: 1,
//!     }).await?;
//!
//!     Ok(())
//! }
//! ```

pub mod builder;
pub mod error;
pub mod macros;
pub mod migration;
pub mod traits;
pub mod types;

#[cfg(feature = "d1")]
pub mod d1;
#[cfg(feature = "sqlite")]
pub mod sqlite;

pub use builder::{build_update_sql, build_upsert_sql, UpsertConfig};
pub use error::Error;
pub use migration::{migrate, migrate_with_logger};
pub use traits::{
    DatabaseExecutor, FieldMeta, FieldUpdate, IntoResultCow, MigrationInfo, MigrationMeta, Query,
    QueryExt, SqlBackend, ToParams,
};
pub use types::DatabaseValue;
