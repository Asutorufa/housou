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
}
