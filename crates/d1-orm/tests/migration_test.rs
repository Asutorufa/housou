#[cfg(feature = "sqlite")]
#[tokio::test]
async fn test_migration_skipping() {
    use d1_orm::sqlite::SqliteExecutor;
    use d1_orm::{define_sql, migrate_with_logger};
    use rusqlite::Connection;
    use std::sync::{Arc, Mutex};

    define_sql!(
        TestMigrations
        @table("users")
        CreateUsersTable => "CREATE TABLE users (id INTEGER PRIMARY KEY)",

        Dummy { _p: &'a str } => "SELECT 1",
    );

    let conn = Connection::open_in_memory().unwrap();
    let executor = SqliteExecutor::new(conn);

    let steps = vec![TestMigrations::CreateUsersTable];

    let logs = Arc::new(Mutex::new(Vec::new()));
    let logs_clone = logs.clone();

    // First run: should apply
    migrate_with_logger(&executor, steps.clone(), |msg| {
        logs_clone.lock().unwrap().push(msg.to_string());
    })
    .await
    .unwrap();

    {
        let logs = logs.lock().unwrap();
        assert_eq!(logs.len(), 1);
        assert!(logs[0].contains("Applying migration"));
    }

    // Second run: should skip
    let logs_clone = logs.clone();
    migrate_with_logger(&executor, steps, |msg| {
        logs_clone.lock().unwrap().push(msg.to_string());
    })
    .await
    .unwrap();

    {
        let logs = logs.lock().unwrap();
        // Should still be 1 log from the first run. Second run adds nothing.
        assert_eq!(logs.len(), 1);
    }
}
