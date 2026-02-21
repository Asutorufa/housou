use worker::wasm_bindgen::JsValue;

#[derive(Clone, Debug)]
pub enum Order {
    Asc,
    Desc,
}

impl std::fmt::Display for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Order::Asc => write!(f, "ASC"),
            Order::Desc => write!(f, "DESC"),
        }
    }
}

pub trait Bindable {
    fn to_sql(&self) -> (String, Vec<JsValue>);
}

pub struct Select {
    table: String,
    columns: Vec<String>,
    wheres: Vec<String>,
    bindings: Vec<JsValue>,
    order_by: Option<(String, Order)>,
    limit: Option<u64>,
    offset: Option<u64>,
    joins: Vec<String>,
}

impl Select {
    pub fn new(table: &str) -> Self {
        Self {
            table: table.to_string(),
            columns: vec!["*".to_string()],
            wheres: Vec::new(),
            bindings: Vec::new(),
            order_by: None,
            limit: None,
            offset: None,
            joins: Vec::new(),
        }
    }

    pub fn columns(mut self, cols: &[&str]) -> Self {
        self.columns = cols.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn where_raw(mut self, condition: &str, bind: JsValue) -> Self {
        self.wheres.push(condition.to_string());
        self.bindings.push(bind);
        self
    }

    pub fn where_eq(self, column: &str, value: impl Into<JsValue>) -> Self {
        self.where_raw(&format!("{} = ?", column), value.into())
    }

    pub fn where_gt(self, column: &str, value: impl Into<JsValue>) -> Self {
        self.where_raw(&format!("{} > ?", column), value.into())
    }

    pub fn where_gte(self, column: &str, value: impl Into<JsValue>) -> Self {
        self.where_raw(&format!("{} >= ?", column), value.into())
    }

    pub fn where_lt(self, column: &str, value: impl Into<JsValue>) -> Self {
        self.where_raw(&format!("{} < ?", column), value.into())
    }

    pub fn where_lte(self, column: &str, value: impl Into<JsValue>) -> Self {
        self.where_raw(&format!("{} <= ?", column), value.into())
    }

    pub fn where_null(mut self, column: &str) -> Self {
        self.wheres.push(format!("{} IS NULL", column));
        self
    }

    pub fn where_not_null(mut self, column: &str) -> Self {
        self.wheres.push(format!("{} IS NOT NULL", column));
        self
    }

    pub fn limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: u64) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn order_by(mut self, column: &str, order: Order) -> Self {
        self.order_by = Some((column.to_string(), order));
        self
    }

    pub fn join(mut self, join: &str) -> Self {
        self.joins.push(join.to_string());
        self
    }
}

impl Bindable for Select {
    fn to_sql(&self) -> (String, Vec<JsValue>) {
        let cols = self.columns.join(", ");
        let mut sql = format!("SELECT {} FROM {}", cols, self.table);

        for join in &self.joins {
            sql.push_str(" ");
            sql.push_str(join);
        }

        if !self.wheres.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.wheres.join(" AND "));
        }

        if let Some((col, order)) = &self.order_by {
            sql.push_str(&format!(" ORDER BY {} {}", col, order));
        }

        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        (sql, self.bindings.clone())
    }
}

pub struct Insert {
    table: String,
    columns: Vec<String>,
    values: Vec<JsValue>,
    returning: Option<String>,
    on_conflict: Option<String>,
}

impl Insert {
    pub fn new(table: &str) -> Self {
        Self {
            table: table.to_string(),
            columns: Vec::new(),
            values: Vec::new(),
            returning: None,
            on_conflict: None,
        }
    }

    pub fn set(mut self, column: &str, value: impl Into<JsValue>) -> Self {
        self.columns.push(column.to_string());
        self.values.push(value.into());
        self
    }

    pub fn returning(mut self, columns: &str) -> Self {
        self.returning = Some(columns.to_string());
        self
    }

    pub fn on_conflict(mut self, clause: &str) -> Self {
        self.on_conflict = Some(clause.to_string());
        self
    }
}

impl Bindable for Insert {
    fn to_sql(&self) -> (String, Vec<JsValue>) {
        let cols = self.columns.join(", ");
        let placeholders: Vec<&str> = self.columns.iter().map(|_| "?").collect();
        let placeholders_str = placeholders.join(", ");

        let mut sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            self.table, cols, placeholders_str
        );

        if let Some(conflict) = &self.on_conflict {
            sql.push_str(" ON CONFLICT ");
            sql.push_str(conflict);
        }

        if let Some(ret) = &self.returning {
            sql.push_str(" RETURNING ");
            sql.push_str(ret);
        }

        (sql, self.values.clone())
    }
}

pub struct Update {
    table: String,
    set_clauses: Vec<String>,
    set_bindings: Vec<JsValue>,
    where_clauses: Vec<String>,
    where_bindings: Vec<JsValue>,
}

impl Update {
    pub fn new(table: &str) -> Self {
        Self {
            table: table.to_string(),
            set_clauses: Vec::new(),
            set_bindings: Vec::new(),
            where_clauses: Vec::new(),
            where_bindings: Vec::new(),
        }
    }

    pub fn set(mut self, column: &str, value: impl Into<JsValue>) -> Self {
        self.set_clauses.push(format!("{} = ?", column));
        self.set_bindings.push(value.into());
        self
    }

    pub fn where_eq(mut self, column: &str, value: impl Into<JsValue>) -> Self {
        self.where_clauses.push(format!("{} = ?", column));
        self.where_bindings.push(value.into());
        self
    }

    pub fn where_null(mut self, column: &str) -> Self {
        self.where_clauses.push(format!("{} IS NULL", column));
        self
    }
}

impl Bindable for Update {
    fn to_sql(&self) -> (String, Vec<JsValue>) {
        let sets = self.set_clauses.join(", ");
        let mut sql = format!("UPDATE {} SET {}", self.table, sets);

        if !self.where_clauses.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.where_clauses.join(" AND "));
        }

        let mut bindings = self.set_bindings.clone();
        bindings.extend(self.where_bindings.clone());

        (sql, bindings)
    }
}

pub struct Delete {
    table: String,
    wheres: Vec<String>,
    bindings: Vec<JsValue>,
}

impl Delete {
    pub fn new(table: &str) -> Self {
        Self {
            table: table.to_string(),
            wheres: Vec::new(),
            bindings: Vec::new(),
        }
    }

    pub fn where_eq(mut self, column: &str, value: impl Into<JsValue>) -> Self {
        self.wheres.push(format!("{} = ?", column));
        self.bindings.push(value.into());
        self
    }

    pub fn where_lt(mut self, column: &str, value: impl Into<JsValue>) -> Self {
        self.wheres.push(format!("{} < ?", column));
        self.bindings.push(value.into());
        self
    }

    pub fn where_gt(mut self, column: &str, value: impl Into<JsValue>) -> Self {
        self.wheres.push(format!("{} > ?", column));
        self.bindings.push(value.into());
        self
    }
}

impl Bindable for Delete {
    fn to_sql(&self) -> (String, Vec<JsValue>) {
        let mut sql = format!("DELETE FROM {}", self.table);
        if !self.wheres.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.wheres.join(" AND "));
        }
        (sql, self.bindings.clone())
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_select() {
        let (sql, bindings) = Select::new("users")
            .where_eq("id", JsValue::from_f64(1.0))
            .limit(5)
            .to_sql();
        assert_eq!(sql, "SELECT * FROM users WHERE id = ? LIMIT 5");
        assert_eq!(bindings.len(), 1);
    }

    #[wasm_bindgen_test]
    fn test_insert() {
        let (sql, bindings) = Insert::new("users")
            .set("name", JsValue::from_str("Alice"))
            .returning("*")
            .to_sql();
        assert_eq!(sql, "INSERT INTO users (name) VALUES (?) RETURNING *");
        assert_eq!(bindings.len(), 1);
    }

    #[wasm_bindgen_test]
    fn test_update() {
        let (sql, bindings) = Update::new("users")
            .set("name", JsValue::from_str("Bob"))
            .where_eq("id", JsValue::from_f64(1.0))
            .to_sql();
        assert_eq!(sql, "UPDATE users SET name = ? WHERE id = ?");
        assert_eq!(bindings.len(), 2);
    }

    #[wasm_bindgen_test]
    fn test_delete() {
        let (sql, bindings) = Delete::new("users")
            .where_eq("id", JsValue::from_f64(1.0))
            .to_sql();
        assert_eq!(sql, "DELETE FROM users WHERE id = ?");
        assert_eq!(bindings.len(), 1);
    }
}
