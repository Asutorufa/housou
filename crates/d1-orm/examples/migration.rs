use d1_orm::sqlite::SqliteExecutor;
use d1_orm::{define_sql, migrate, Migration};

// Define some migration SQL statements with migration metadata attached
define_sql!(
    MyMigrations

    @table("users")
    CreateUsersTable => "CREATE TABLE users (id INTEGER PRIMARY KEY, username TEXT NOT NULL)",

    @index("idx_users_username")
    CreateUsersUsernameIndex => "CREATE UNIQUE INDEX idx_users_username ON users(username)",

    @column("users", "email")
    AddUsersEmailColumn => "ALTER TABLE users ADD COLUMN email TEXT",

    // Add a dummy parameterized query so that the generated 'a lifetime is used
    DummyQuery { param: &'a str } => "SELECT ?",
);

#[tokio::main]
async fn main() -> Result<(), d1_orm::Error> {
    // 1. Initialize SQLite Backend
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    let executor = SqliteExecutor::new(conn);

    // 2. Define migrations
    let migrations = vec![
        Migration::new(
            1,
            "Initial setup",
            vec![
                MyMigrations::CreateUsersTable,
                MyMigrations::CreateUsersUsernameIndex,
            ],
        ),
        Migration::new(
            2,
            "Add email column",
            vec![MyMigrations::AddUsersEmailColumn],
        ),
    ];

    println!("Starting migrations...");

    // Execute migrations using the generic helper
    // Uses default table "_d1_migrations" and println! logger
    // Explicit type annotation |msg: &str| helps the compiler with lifetime inference
    migrate(
        &executor,
        migrations,
        None,
        Some(|msg: &str| println!("{}", msg)),
    )
    .await?;

    println!("Migrations completed successfully!");

    Ok(())
}
