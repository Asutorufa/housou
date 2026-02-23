# d1-orm

A lightweight ORM and SQL builder for Rust, designed to run anywhere, perfectly suited for Cloudflare D1 and SQLite.

## Features

- **Backend Agnostic**: Bring your own database driver or use the built-in integrations for Cloudflare D1 (via `worker` crate) and SQLite (via `rusqlite`). 
- **Type-Safe SQL Builder**: `define_sql!` macro for type-safe parameter binding and zero-overhead declarative SQL queries.
- **Model Definition**: `define_model!` macro for defining structs, mapping database models, and generating update structs automatically.
- **WASM Compatible**: Works flawlessly within WASM targets like Cloudflare Workers (non-Send environments).
- **Migration System**: Built-in migration utilities for easy schema management.
- **Async Trait**: Implements an ecosystem agnostic `DatabaseExecutor` trait for async database operations.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
# For Cloudflare Workers (D1)
d1-orm = { version = "0.1", features = ["d1"] } 

# For native environments (SQLite)
d1-orm = { version = "0.1", features = ["sqlite"] } 
```

## Quick Start
See [`examples/basic.rs`](https://github.com/Asutorufa/housou/blob/main/crates/d1-orm/examples/basic.rs) for the complete runnable example.

### 1. Define Model
```rust
use d1_orm::define_model;

define_model!(
    /// User model representing the `users` table
    User,
    UserField,
    UserUpdate {
        id: i32 [pk],
        username: String,
        email: String,
    }
);
```

### 2. Define Queries
```rust
use d1_orm::{define_sql, build_update_sql};

define_sql!(
    MySql
    
    // Select queries map nicely
    GetUser { id: i32 } => "SELECT * FROM users WHERE id = ?",
    
    // Insert parameters
    CreateUser { username: &'a str, email: &'a str } => 
        "INSERT INTO users (username, email) VALUES (?, ?)",
        
    // Dynamic updates generated automagically
    // Note: 'updates' must come before 'id' because the generated SQL
    // is structured as 'UPDATE users SET ... WHERE id = ?'
    UpdateUser { updates: Vec<UserUpdate> [skip_primary_key], id: i32 } => 
        build_update_sql("users", "id", &updates),
);
```

### 3. Execute
```rust,no_run
use d1_orm::{DatabaseExecutor, sqlite::SqliteExecutor};
use d1_orm::{define_sql, build_update_sql, define_model};

# define_model!(User, UserField, UserUpdate { id: i32 [pk], username: String, email: String, });
# define_sql!(MySql GetUser { id: i32 } => "SELECT 1", CreateUser { username: &'a str, email: &'a str } => "INSERT", UpdateUser { updates: Vec<UserUpdate> [skip_primary_key], id: i32 } => build_update_sql("users", "id", &updates),);

#[tokio::main]
async fn main() -> Result<(), d1_orm::Error> {
    // 1. Initialize SQLite Backend
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, username TEXT NOT NULL, email TEXT NOT NULL)", []).unwrap();
    
    let executor = SqliteExecutor::new(conn);

    // 2. Strongly Typed Interactions 
    executor.execute(MySql::CreateUser {
        username: "alice",
        email: "alice@example.com",
    }).await?;

    // query_first and query_all methods provided by DatabaseExecutor
    let alice: Option<User> = executor.query_first(MySql::GetUser { id: 1 }).await?;
    println!("Alice: {:?}", alice);
    
    // 3. Perform programmatic updates effortlessly!
    executor.execute(MySql::UpdateUser {
        updates: vec![UserUpdate::email("alice.new@example.com".to_string())],
        id: 1,
    }).await?;

    Ok(())
}
```

## Credits

Part of the [Housou](https://github.com/Asutorufa/housou) project.
