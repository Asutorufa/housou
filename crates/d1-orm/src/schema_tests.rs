#[cfg(test)]
mod tests {
    use crate::schema::{Column, ColumnType, Index, Table};

    #[test]
    fn test_table_sql_generation() {
        let table = Table::new("users")
            .column(
                Column::new("id", ColumnType::Integer)
                    .primary_key()
                    .auto_increment(),
            )
            .column(Column::new("email", ColumnType::Text).unique().not_null());

        let sql = table.to_sql();
        assert_eq!(
            sql,
            "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY AUTOINCREMENT, email TEXT UNIQUE NOT NULL)"
        );
    }

    #[test]
    fn test_index_sql_generation() {
        let index = Index::new("idx_users_email", "users")
            .column("email")
            .unique();

        let sql = index.to_sql();
        assert_eq!(
            sql,
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email ON users (email)"
        );
    }

    #[test]
    fn test_alter_table_single_sql() {
        let sql = crate::schema::AlterTable::new("users")
            .add_column(Column::new("avatar_url", ColumnType::Text))
            .to_single_sql();
        assert_eq!(
            sql,
            Some("ALTER TABLE users ADD COLUMN avatar_url TEXT".to_string())
        );
    }

    #[test]
    fn test_table_macro_generation() {
        let table = crate::d1_table!(
            "users",
            columns = [
                crate::d1_column!("id", ColumnType::Integer, [primary_key, auto_increment]),
                crate::d1_column!("email", ColumnType::Text, [unique, not_null])
            ],
            constraints = ["FOREIGN KEY(id) REFERENCES accounts(id)"]
        );

        let sql = table.to_sql();
        assert_eq!(
            sql,
            "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY AUTOINCREMENT, email TEXT UNIQUE NOT NULL, FOREIGN KEY(id) REFERENCES accounts(id))"
        );
    }

    #[test]
    fn test_additive_migration_sql() {
        let from = Table::new("users").column(Column::new("id", ColumnType::Integer));
        let to = Table::new("users")
            .column(Column::new("id", ColumnType::Integer))
            .column(Column::new("avatar_url", ColumnType::Text));

        let from_indexes = vec![];
        let to_indexes = vec![Index::new("idx_users_avatar_url", "users").column("avatar_url")];

        let sql = crate::schema::additive_migration_sql(&from, &to, &from_indexes, &to_indexes);
        assert_eq!(
            sql,
            vec![
                "ALTER TABLE users ADD COLUMN avatar_url TEXT".to_string(),
                "CREATE INDEX IF NOT EXISTS idx_users_avatar_url ON users (avatar_url)".to_string(),
            ]
        );
    }
}
