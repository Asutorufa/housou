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
}
