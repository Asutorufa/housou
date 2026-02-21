#[cfg(test)]
mod tests {
    use crate::schema::{Column, ColumnType, Index, Table};
    use crate::{Migration, SchemaProbe};
    use serde::{Deserialize, Serialize};

    #[test]
    fn test_migration_macro_with_sqls_and_infer() {
        let migration = crate::d1_migration!(
            10,
            sqls = vec!["CREATE TABLE foo (id INTEGER)".to_string()],
            infer = [crate::d1_probe!(table "foo")]
        );

        assert_eq!(migration.version, 10);
        assert_eq!(migration.sql.len(), 1);
        assert!(matches!(migration.infer_when[0], SchemaProbe::Table(ref t) if t == "foo"));
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

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct MockModel;

    impl crate::Model for MockModel {
        const TABLE: &'static str = "mock";

        fn schema_at(version: i32) -> Option<Table> {
            match version {
                1 => Some(Table::new("mock").column(Column::new("id", ColumnType::Integer))),
                2 => Some(
                    Table::new("mock")
                        .column(Column::new("id", ColumnType::Integer))
                        .column(Column::new("name", ColumnType::Text)),
                ),
                _ => None,
            }
        }

        fn indexes_at(version: i32) -> Vec<Index> {
            if version >= 2 {
                vec![Index::new("idx_mock_name", "mock").column("name")]
            } else {
                Vec::new()
            }
        }

        fn latest_version() -> i32 {
            2
        }
    }

    #[test]
    fn test_model_setup_and_diff_sql() {
        let setup = crate::model_setup_sql::<MockModel>(1);
        assert_eq!(setup.len(), 1);
        assert_eq!(setup[0], "CREATE TABLE IF NOT EXISTS mock (id INTEGER)");

        let diff = crate::model_diff_sql::<MockModel>(1, 2);
        assert_eq!(
            diff,
            vec![
                "ALTER TABLE mock ADD COLUMN name TEXT".to_string(),
                "CREATE INDEX IF NOT EXISTS idx_mock_name ON mock (name)".to_string(),
            ]
        );
    }

    #[test]
    fn test_auto_migrations_macro() {
        let migrations = crate::d1_auto_migrations!(MockModel);
        assert_eq!(migrations.len(), 2);
        assert_eq!(migrations[0].version, 1);
        assert_eq!(migrations[1].version, 2);
    }
}
