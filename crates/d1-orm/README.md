# d1-orm

A lightweight ORM and SQL builder for Rust, designed for Cloudflare D1 and SQLite.

## Features

- **Backend Agnostic**: Supports Cloudflare D1 (via `worker` crate) and SQLite (via `rusqlite`).
- **Type-Safe SQL Builder**: `define_sql!` macro for type-safe SQL queries with parameter binding.
- **Model Definition**: `define_model!` macro for defining database models and update structs.
- **Migration Support**: Built-in migration helpers.
- **Async Trait**: `DatabaseExecutor` trait for async database operations.

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
d1-orm = { version = "0.1", features = ["d1"] } # For Cloudflare Workers
# or
d1-orm = { version = "0.1", features = ["sqlite"] } # For SQLite
```

## Example

```rust
use d1_orm::{define_model, define_sql};

define_model!(User, UserField, UserUpdate {
    id: i32 [pk],
    username: String,
    email: String,
});

define_sql!(
    MySql
    GetUser { id: i32 } => "SELECT * FROM users WHERE id = ?",
    CreateUser { username: &'a str, email: &'a str } => "INSERT INTO users (username, email) VALUES (?, ?)",
);
```
