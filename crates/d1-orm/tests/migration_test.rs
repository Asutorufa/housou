#[cfg(feature = "sqlite")]
#[tokio::test]
async fn test_migration_versioning() {
    use d1_orm::sqlite::SqliteExecutor;
    use d1_orm::{define_sql, migrate, DatabaseExecutor, Migration};
    use rusqlite::Connection;
    use std::sync::{Arc, Mutex};

    define_sql!(
        TestMigrations
        @table("users")
        CreateUsersTable => "CREATE TABLE users (id INTEGER PRIMARY KEY)",

        @column("users", "email")
        AddEmail => "ALTER TABLE users ADD COLUMN email TEXT",

        Dummy { _p: &'a str } => "INSERT INTO users (id) SELECT ? WHERE 1=0",
    );

    let conn = Connection::open_in_memory().unwrap();
    let executor = SqliteExecutor::new(conn);

    let migrations = vec![
        Migration::new(1, "Init", vec![TestMigrations::CreateUsersTable]),
        Migration::new(2, "Add Email", vec![TestMigrations::AddEmail]),
    ];

    let logs = Arc::new(Mutex::new(Vec::new()));
    let logs_clone = logs.clone();

    // First run: Apply all
    migrate(
        &executor,
        migrations.clone(),
        Some("my_migrations"),
        Some(move |msg: &str| {
            logs_clone.lock().unwrap().push(msg.to_string());
        }),
    )
    .await
    .unwrap();

    {
        let logs = logs.lock().unwrap();
        // Should have "Applying migration v1", "Executing step", "Applying migration v2", "Executing step"
        assert!(logs.iter().any(|s| s.contains("Applying migration v1")));
        assert!(logs.iter().any(|s| s.contains("Applying migration v2")));
    }

    // Verify version table
    #[derive(serde::Deserialize)]
    struct Ver {
        ver: u32,
    }
    // Correct macro syntax: EnumName Variant => SQL
    // Added Dummy variant to use lifetime 'a
    define_sql!(CheckVer
        GetVer => "SELECT MAX(version) as ver FROM my_migrations",
        Dummy { _p: &'a str } => "SELECT ?"
    );
    let v: Option<Ver> = executor.query_first(CheckVer::GetVer).await.unwrap();
    assert_eq!(v.unwrap().ver, 2);

    // Second run: Should skip all
    logs.lock().unwrap().clear();
    let logs_clone = logs.clone();
    migrate(
        &executor,
        migrations.clone(),
        Some("my_migrations"),
        Some(move |msg: &str| {
            logs_clone.lock().unwrap().push(msg.to_string());
        }),
    )
    .await
    .unwrap();

    {
        let logs = logs.lock().unwrap();
        // Should be empty as version check skips them entirely
        assert!(logs.is_empty());
    }

    // Third run: Add new migration
    let new_migrations = vec![
        Migration::new(1, "Init", vec![TestMigrations::CreateUsersTable]),
        Migration::new(2, "Add Email", vec![TestMigrations::AddEmail]),
        Migration::new(3, "Dummy", vec![TestMigrations::Dummy { _p: "foo" }]),
    ];

    logs.lock().unwrap().clear();
    let logs_clone = logs.clone();
    migrate(
        &executor,
        new_migrations,
        Some("my_migrations"),
        Some(move |msg: &str| {
            logs_clone.lock().unwrap().push(msg.to_string());
        }),
    )
    .await
    .unwrap();

    {
        let logs = logs.lock().unwrap();
        // Should only apply v3
        assert!(!logs.iter().any(|s| s.contains("Applying migration v1")));
        assert!(!logs.iter().any(|s| s.contains("Applying migration v2")));
        assert!(logs.iter().any(|s| s.contains("Applying migration v3")));
    }
}
