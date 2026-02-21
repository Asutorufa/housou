use std::collections::HashSet;
use std::fmt;

/// SQLite column types supported by the schema builder.
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnType {
    /// `INTEGER`
    Integer,
    /// `REAL`
    Real,
    /// `TEXT`
    Text,
    /// `BLOB`
    Blob,
    /// Boolean mapped to SQLite `INTEGER`.
    Boolean,
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

/// Column-level constraints.
#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    /// `PRIMARY KEY`
    PrimaryKey,
    /// `AUTOINCREMENT`
    AutoIncrement,
    /// `NOT NULL`
    NotNull,
    /// `UNIQUE`
    Unique,
    /// `DEFAULT <expr>`
    Default(String),
}

/// Table column definition.
#[derive(Debug, Clone)]
pub struct Column {
    /// Column name.
    pub name: String,
    /// Column data type.
    pub col_type: ColumnType,
    /// Column constraints.
    pub constraints: Vec<Constraint>,
}

impl Column {
    /// Create a new column.
    pub fn new(name: &str, col_type: ColumnType) -> Self {
        Self {
            name: name.to_string(),
            col_type,
            constraints: Vec::new(),
        }
    }

    /// Add `PRIMARY KEY`.
    pub fn primary_key(mut self) -> Self {
        self.constraints.push(Constraint::PrimaryKey);
        self
    }

    /// Add `AUTOINCREMENT`.
    pub fn auto_increment(mut self) -> Self {
        self.constraints.push(Constraint::AutoIncrement);
        self
    }

    /// Add `NOT NULL`.
    pub fn not_null(mut self) -> Self {
        self.constraints.push(Constraint::NotNull);
        self
    }

    /// Add `UNIQUE`.
    pub fn unique(mut self) -> Self {
        self.constraints.push(Constraint::Unique);
        self
    }

    /// Render this column definition into SQL.
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

/// Index definition.
#[derive(Debug, Clone)]
pub struct Index {
    /// Index name.
    pub name: String,
    /// Target table.
    pub table: String,
    /// Indexed columns.
    pub columns: Vec<String>,
    /// Whether index is unique.
    pub unique: bool,
}

impl Index {
    /// Create a new index for a table.
    pub fn new(name: &str, table: &str) -> Self {
        Self {
            name: name.to_string(),
            table: table.to_string(),
            columns: Vec::new(),
            unique: false,
        }
    }

    /// Add a column to the index.
    pub fn column(mut self, col: &str) -> Self {
        self.columns.push(col.to_string());
        self
    }

    /// Mark index as unique.
    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    /// Render `CREATE INDEX IF NOT EXISTS ...`.
    pub fn to_sql(&self) -> String {
        let unique = if self.unique { "UNIQUE " } else { "" };
        let cols = self.columns.join(", ");
        format!(
            "CREATE {}INDEX IF NOT EXISTS {} ON {} ({})",
            unique, self.name, self.table, cols
        )
    }
}

/// Table definition used to build `CREATE TABLE` SQL.
pub struct Table {
    /// Table name.
    pub name: String,
    /// Column definitions.
    pub columns: Vec<Column>,
    /// Raw table-level constraints (for example, composite unique keys).
    pub constraints: Vec<String>,
}

impl Table {
    /// Create a new table definition.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            columns: Vec::new(),
            constraints: Vec::new(),
        }
    }

    /// Add one column to the table.
    pub fn column(mut self, col: Column) -> Self {
        self.columns.push(col);
        self
    }

    /// Render `CREATE TABLE IF NOT EXISTS ...`.
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

/// Helper for generating additive `ALTER TABLE` SQL.
pub struct AlterTable {
    table: String,
    actions: Vec<String>,
}

impl AlterTable {
    /// Create an alter-table builder.
    pub fn new(table: &str) -> Self {
        Self {
            table: table.to_string(),
            actions: Vec::new(),
        }
    }

    /// Add an `ADD COLUMN` action.
    pub fn add_column(mut self, col: Column) -> Self {
        self.actions.push(format!("ADD COLUMN {}", col.to_sql()));
        self
    }

    /// Render SQL if exactly one action exists.
    ///
    /// SQLite only supports one `ADD COLUMN` per `ALTER TABLE` statement.
    pub fn to_single_sql(&self) -> Option<String> {
        if self.actions.len() == 1 {
            Some(format!("ALTER TABLE {} {}", self.table, self.actions[0]))
        } else {
            None
        }
    }
}

/// Generate additive migration SQL by comparing previous and next schema states.
///
/// This function only emits safe additive operations:
/// - missing columns become `ALTER TABLE ... ADD COLUMN ...`
/// - missing indexes become `CREATE INDEX IF NOT EXISTS ...`
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
