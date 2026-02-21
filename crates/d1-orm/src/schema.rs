use std::collections::HashSet;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum ColumnType {
    Integer,
    Real,
    Text,
    Blob,
    Boolean, // Mapped to Integer
}

impl fmt::Display for ColumnType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColumnType::Integer | ColumnType::Boolean => write!(f, "INTEGER"),
            ColumnType::Real => write!(f, "REAL"),
            ColumnType::Text => write!(f, "TEXT"),
            ColumnType::Blob => write!(f, "BLOB"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    PrimaryKey,
    AutoIncrement,
    NotNull,
    Unique,
    Default(String),
}

#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub col_type: ColumnType,
    pub constraints: Vec<Constraint>,
}

impl Column {
    pub fn new(name: &str, col_type: ColumnType) -> Self {
        Self {
            name: name.to_string(),
            col_type,
            constraints: Vec::new(),
        }
    }

    pub fn primary_key(mut self) -> Self {
        self.constraints.push(Constraint::PrimaryKey);
        self
    }

    pub fn auto_increment(mut self) -> Self {
        self.constraints.push(Constraint::AutoIncrement);
        self
    }

    pub fn not_null(mut self) -> Self {
        self.constraints.push(Constraint::NotNull);
        self
    }

    pub fn unique(mut self) -> Self {
        self.constraints.push(Constraint::Unique);
        self
    }

    pub fn to_sql(&self) -> String {
        let mut sql = format!("{} {}", self.name, self.col_type);
        for constraint in &self.constraints {
            match constraint {
                Constraint::PrimaryKey => sql.push_str(" PRIMARY KEY"),
                Constraint::AutoIncrement => sql.push_str(" AUTOINCREMENT"),
                Constraint::NotNull => sql.push_str(" NOT NULL"),
                Constraint::Unique => sql.push_str(" UNIQUE"),
                Constraint::Default(s) => sql.push_str(&format!(" DEFAULT {}", s)),
            }
        }
        sql
    }
}

#[derive(Debug, Clone)]
pub struct Index {
    pub name: String,
    pub table: String,
    pub columns: Vec<String>,
    pub unique: bool,
}

impl Index {
    pub fn new(name: &str, table: &str) -> Self {
        Self {
            name: name.to_string(),
            table: table.to_string(),
            columns: Vec::new(),
            unique: false,
        }
    }

    pub fn column(mut self, col: &str) -> Self {
        self.columns.push(col.to_string());
        self
    }

    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    pub fn to_sql(&self) -> String {
        let unique = if self.unique { "UNIQUE " } else { "" };
        let cols = self.columns.join(", ");
        format!(
            "CREATE {}INDEX IF NOT EXISTS {} ON {} ({})",
            unique, self.name, self.table, cols
        )
    }
}

pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
    pub constraints: Vec<String>, // Table-level constraints
}

impl Table {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            columns: Vec::new(),
            constraints: Vec::new(),
        }
    }

    pub fn column(mut self, col: Column) -> Self {
        self.columns.push(col);
        self
    }

    pub fn constraint(mut self, constraint: impl Into<String>) -> Self {
        self.constraints.push(constraint.into());
        self
    }

    pub fn to_sql(&self) -> String {
        let mut defs: Vec<String> = self.columns.iter().map(|c| c.to_sql()).collect();
        defs.extend(self.constraints.clone());
        format!(
            "CREATE TABLE IF NOT EXISTS {} ({})",
            self.name,
            defs.join(", ")
        )
    }
}

#[macro_export]
macro_rules! d1_column {
    ($name:expr, $col_type:expr) => {
        $crate::Column::new($name, $col_type)
    };
    ($name:expr, $col_type:expr, [$($constraint:ident),+ $(,)?]) => {
        $crate::d1_column!(@apply $crate::Column::new($name, $col_type), $($constraint),+)
    };
    (@apply $col:expr, $constraint:ident) => {
        ($col).$constraint()
    };
    (@apply $col:expr, $head:ident, $($tail:ident),+) => {
        $crate::d1_column!(@apply ($col).$head(), $($tail),+)
    };
}

#[macro_export]
macro_rules! d1_table {
    (
        $name:expr,
        columns = [$($column:expr),+ $(,)?]
        $(, constraints = [$($constraint:expr),* $(,)?])?
        $(,)?
    ) => {{
        let table = $crate::Table::new($name);
        let table = $crate::d1_table!(@with_columns table, $($column),+);
        $(
            let table = $crate::d1_table!(@with_constraints table, $($constraint),*);
        )?
        table
    }};
    (@with_columns $table:expr, $column:expr) => {
        ($table).column($column)
    };
    (@with_columns $table:expr, $column:expr, $($rest:expr),+) => {
        $crate::d1_table!(@with_columns ($table).column($column), $($rest),+)
    };
    (@with_constraints $table:expr) => {
        $table
    };
    (@with_constraints $table:expr, $constraint:expr) => {
        ($table).constraint($constraint)
    };
    (@with_constraints $table:expr, $head:expr, $($tail:expr),+) => {
        $crate::d1_table!(@with_constraints ($table).constraint($head), $($tail),+)
    };
}

// Alter Table Helpers for Migrations
pub struct AlterTable {
    table: String,
    actions: Vec<String>,
}

impl AlterTable {
    pub fn new(table: &str) -> Self {
        Self {
            table: table.to_string(),
            actions: Vec::new(),
        }
    }

    pub fn add_column(mut self, col: Column) -> Self {
        self.actions.push(format!("ADD COLUMN {}", col.to_sql()));
        self
    }

    pub fn to_sql_stmts(&self) -> Vec<String> {
        self.actions
            .iter()
            .map(|action| format!("ALTER TABLE {} {}", self.table, action))
            .collect()
    }

    pub fn to_single_sql(&self) -> Option<String> {
        if self.actions.len() == 1 {
            Some(format!("ALTER TABLE {} {}", self.table, self.actions[0]))
        } else {
            None
        }
    }
}

pub fn additive_migration_sql(
    from_table: &Table,
    to_table: &Table,
    from_indexes: &[Index],
    to_indexes: &[Index],
) -> Vec<String> {
    let mut sql = Vec::new();

    let from_columns: HashSet<&str> = from_table.columns.iter().map(|c| c.name.as_str()).collect();
    for column in &to_table.columns {
        if !from_columns.contains(column.name.as_str()) {
            if let Some(stmt) = AlterTable::new(&to_table.name)
                .add_column(column.clone())
                .to_single_sql()
            {
                sql.push(stmt);
            }
        }
    }

    let from_index_names: HashSet<&str> = from_indexes.iter().map(|i| i.name.as_str()).collect();
    for index in to_indexes {
        if !from_index_names.contains(index.name.as_str()) {
            sql.push(index.to_sql());
        }
    }

    sql
}
