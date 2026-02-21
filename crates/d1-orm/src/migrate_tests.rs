#[cfg(test)]
mod tests {
    use crate::{Migration, SchemaProbe};

    #[test]
    fn test_migration_macro_with_sqls_and_infer() {
        let migration = crate::d1_migration!(
            10,
            sqls = vec!["CREATE TABLE foo (id INTEGER)".to_string()],
            infer = [crate::d1_probe!(table "foo")]
        );

        assert_eq!(migration.version, 10);
        assert_eq!(migration.sql.len(), 1);
        assert!(matches!(migration.infer_when[0], SchemaProbe::Table("foo")));
    }

    #[test]
    fn test_migration_macro_with_sql_array() {
        let migration = crate::d1_migration!(
            11,
            sql = [
                "ALTER TABLE foo ADD COLUMN bar TEXT",
                "CREATE INDEX idx_foo_bar ON foo(bar)"
            ]
        );

        assert_eq!(migration.version, 11);
        assert_eq!(migration.sql.len(), 2);
    }

    #[test]
    fn test_migrations_macro() {
        let migrations: Vec<Migration> = crate::d1_migrations![
            crate::d1_migration!(1, sql = "SELECT 1"),
            crate::d1_migration!(2, sql = "SELECT 2"),
        ];
        assert_eq!(migrations.len(), 2);
    }
}
