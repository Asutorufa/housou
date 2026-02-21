#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    use crate::Bindable;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, crate::Model)]
    #[d1(table_name = "test_users")]
    struct TestUser {
        #[d1(primary_key, auto_increment)]
        id: i32,
        name: String,
        #[d1(unique)]
        email: String,
    }

    #[test]
    fn test_derived_insert_query() {
        let user = TestUser {
            id: 42,
            name: "alice".to_string(),
            email: "alice@example.com".to_string(),
        };

        let insert = user.insert_query().expect("insert query should build");
        let (sql, bindings) = insert.to_sql();

        assert_eq!(sql, "INSERT INTO test_users (name, email) VALUES (?, ?)");
        assert_eq!(bindings.len(), 2);
    }

    #[test]
    fn test_derived_update_query() {
        let user = TestUser {
            id: 42,
            name: "alice".to_string(),
            email: "alice@example.com".to_string(),
        };

        let update = user.update_query().expect("update query should build");
        let (sql, bindings) = update.to_sql();

        assert_eq!(
            sql,
            "UPDATE test_users SET name = ?, email = ? WHERE id = ?"
        );
        assert_eq!(bindings.len(), 3);
    }
}
