use d1_orm::sqlite::SqliteExecutor;
use d1_orm::{build_update_sql, define_model, define_sql, DatabaseExecutor};

// 1. Define your model and update struct
define_model!(
    /// A user in the system
    User,
    UserField,
    UserUpdate {
        id: i32 [pk],
        username: String,
        email: String,
    }
);

// 2. Define your SQL queries
define_sql!(
    MySql

    // Simple parameterized query
    GetUser { id: i32 } => "SELECT * FROM users WHERE id = ?",

    // Insert query with multiple parameters
    CreateUser { username: &'a str, email: &'a str } =>
        "INSERT INTO users (username, email) VALUES (?, ?)",

    // Dynamic update query using the generated UserUpdate enum
    // Note: 'updates' must come before 'id' because the generated SQL
    // is structured as 'UPDATE users SET ... WHERE id = ?'
    UpdateUser { updates: Vec<UserUpdate> [skip_primary_key], id: i32 } =>
        build_update_sql("users", "id", &updates),
);

#[tokio::main]
async fn main() -> Result<(), d1_orm::Error> {
    // 3. Initialize the database backend (SQLite in this case)
    // `d1-orm` also supports Cloudflare D1 via the `worker` crate!
    let conn = rusqlite::Connection::open_in_memory().unwrap();

    // Create table for the example
    conn.execute(
        "CREATE TABLE users (id INTEGER PRIMARY KEY, username TEXT NOT NULL, email TEXT NOT NULL)",
        [],
    )
    .unwrap();

    let executor = SqliteExecutor::new(conn);

    // 4. Execute queries using the type-safe builder

    // Create a new user
    executor
        .execute(MySql::CreateUser {
            username: "alice",
            email: "alice@example.com",
        })
        .await?;

    // Query the user
    let user: Option<User> = executor.query_first(MySql::GetUser { id: 1 }).await?;
    println!("Found user: {:?}", user);

    // Dynamic Updates!
    // This allows you to construct updates programmatically
    // without writing boilerplate for every permutation of fields
    executor
        .execute(MySql::UpdateUser {
            updates: vec![UserUpdate::email("alice.new@example.com".to_string())],
            id: 1,
        })
        .await?;

    // Verify it updated
    let updated: Option<User> = executor.query_first(MySql::GetUser { id: 1 }).await?;
    println!("Updated user: {:?}", updated);

    Ok(())
}
