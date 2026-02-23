use d1_orm::sqlite::SqliteExecutor;
use d1_orm::{define_sql, DatabaseExecutor, MigrationInfo, MigrationMeta};

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

    let steps = vec![
        MyMigrations::CreateUsersTable,
        MyMigrations::CreateUsersUsernameIndex,
        MyMigrations::AddUsersEmailColumn,
    ];

    println!("Starting migrations...");

    for step in steps {
        if let Some(info) = step.migration_info() {
            match info {
                MigrationInfo::Table(name) => {
                    println!("Ensuring table '{}' exists...", name);
                    // In a real app, you would check if the table exists first:
                    // SELECT name FROM sqlite_master WHERE type='table' AND name = ?
                }
                MigrationInfo::Index(name) => {
                    println!("Ensuring index '{}' exists...", name);
                    // Check if index exists:
                    // SELECT name FROM sqlite_master WHERE type='index' AND name = ?
                }
                MigrationInfo::Column { table, column } => {
                    println!(
                        "Ensuring column '{}' exists in table '{}'...",
                        column, table
                    );
                    // Check if column exists:
                    // SELECT * FROM pragma_table_info(?)
                }
            }

            // Execute the migration step
            executor.execute(step).await?;
        }
    }

    println!("Migrations completed successfully!");

    Ok(())
}
