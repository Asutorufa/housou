use crate::{D1Database, Error, Result};
use serde::Deserialize;
use std::collections::HashSet;
use worker::wasm_bindgen::JsValue;

/// Build `CREATE TABLE` + `CREATE INDEX` SQL for a model at a given version.
pub fn model_setup_sql<M: crate::Model>(version: i32) -> Vec<String> {
    let mut sql = Vec::new();
    if let Some(table) = M::schema_at(version) {
        sql.push(table.to_sql());
        sql.extend(M::indexes_at(version).into_iter().map(|idx| idx.to_sql()));
    }
    sql
}

/// Build additive migration SQL for a model between two versions.
pub fn model_diff_sql<M: crate::Model>(from: i32, to: i32) -> Vec<String> {
    match (M::schema_at(from), M::schema_at(to)) {
        (None, None) => Vec::new(),
        (None, Some(_)) => model_setup_sql::<M>(to),
        (Some(_), None) => Vec::new(),
        (Some(from_table), Some(to_table)) => crate::additive_migration_sql(
            &from_table,
            &to_table,
            &M::indexes_at(from),
            &M::indexes_at(to),
        ),
    }
}

/// Build SQL for a single version step (`from -> to`) for one model.
pub fn model_step_sql<M: crate::Model>(from: i32, to: i32) -> Vec<String> {
    if from <= 0 {
        model_setup_sql::<M>(to)
    } else {
        model_diff_sql::<M>(from, to)
    }
}

/// Schema probe used to infer whether a migration was already applied.
#[derive(Debug, Clone)]
pub enum SchemaProbe {
    /// Check table existence.
    Table(String),
    /// Check column existence in table.
    Column {
        /// Table name.
        table: String,
        /// Column name.
        column: String,
    },
    /// Check index existence.
    Index(String),
}

/// Build inference probes for a single version step (`from -> to`) for one model.
pub fn model_step_probes<M: crate::Model>(from: i32, to: i32) -> Vec<SchemaProbe> {
    match (M::schema_at(from), M::schema_at(to)) {
        (None, Some(table)) => {
            let mut probes = vec![SchemaProbe::Table(table.name)];
            probes.extend(
                M::indexes_at(to)
                    .into_iter()
                    .map(|idx| SchemaProbe::Index(idx.name)),
            );
            probes
        }
        (Some(from_table), Some(to_table)) => {
            let mut probes = Vec::new();
            let from_columns: HashSet<&str> = from_table
                .columns
                .iter()
                .map(|c| c.name.as_str())
                .collect();
            for column in &to_table.columns {
                if !from_columns.contains(column.name.as_str()) {
                    probes.push(SchemaProbe::Column {
                        table: to_table.name.clone(),
                        column: column.name.clone(),
                    });
                }
            }

            let from_index_names: HashSet<String> =
                M::indexes_at(from).into_iter().map(|i| i.name).collect();
            for index in M::indexes_at(to) {
                if !from_index_names.contains(&index.name) {
                    probes.push(SchemaProbe::Index(index.name));
                }
            }
            probes
        }
        _ => Vec::new(),
    }
}

/// Declarative migration definition.
#[derive(Debug, Clone)]
pub struct Migration {
    /// Monotonic migration version.
    pub version: i32,
    /// SQL statements to execute for this migration.
    pub sql: Vec<String>,
    /// Optional probes that can infer this migration as already applied.
    pub infer_when: Vec<SchemaProbe>,
}

impl Migration {
    /// Create a migration with a version.
    pub fn new(version: i32) -> Self {
        Self {
            version,
            sql: Vec::new(),
            infer_when: Vec::new(),
        }
    }

    /// Append one SQL statement.
    pub fn with_sql(mut self, sql: impl Into<String>) -> Self {
        self.sql.push(sql.into());
        self
    }

    /// Append multiple SQL statements.
    pub fn with_sqls<I, S>(mut self, sqls: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.sql.extend(sqls.into_iter().map(Into::into));
        self
    }

    /// Add one inference probe.
    pub fn infer_if(mut self, probe: SchemaProbe) -> Self {
        self.infer_when.push(probe);
        self
    }
}

/// Build a [`SchemaProbe`] from concise syntax.
#[macro_export]
macro_rules! d1_probe {
    (table $table:expr) => {
        $crate::SchemaProbe::Table(($table).to_string())
    };
    (index $index:expr) => {
        $crate::SchemaProbe::Index(($index).to_string())
    };
    (column $table:expr, $column:expr) => {
        $crate::SchemaProbe::Column {
            table: ($table).to_string(),
            column: ($column).to_string(),
        }
    };
}

/// Build a [`Migration`] with SQL and optional inference probes.
#[macro_export]
macro_rules! d1_migration {
    (@apply $migration:expr) => {
        $migration
    };
    (@apply $migration:expr, $probe:expr $(, $rest:expr)*) => {
        $crate::d1_migration!(@apply $migration.infer_if($probe) $(, $rest)*)
    };
    ($version:expr, sqls = $sqls:expr $(, infer = [$($probe:expr),* $(,)?])? $(,)?) => {{
        $crate::d1_migration!(
            @apply
            $crate::Migration::new($version).with_sqls($sqls)
            $(, $($probe),*)?
        )
    }};
    ($version:expr, sql = [$($sql:expr),+ $(,)?] $(, infer = [$($probe:expr),* $(,)?])? $(,)?) => {{
        $crate::d1_migration!(
            @apply
            $crate::Migration::new($version).with_sqls(vec![$($sql),+])
            $(, $($probe),*)?
        )
    }};
    ($version:expr, sql = $sql:expr $(, infer = [$($probe:expr),* $(,)?])? $(,)?) => {{
        $crate::d1_migration!(
            @apply
            $crate::Migration::new($version).with_sql($sql)
            $(, $($probe),*)?
        )
    }};
}

/// Build a migration vector.
#[macro_export]
macro_rules! d1_migrations {
    ($($migration:expr),* $(,)?) => {
        vec![$($migration),*]
    };
}

/// Build setup SQL for multiple models at one version.
#[macro_export]
macro_rules! d1_model_setup_sqls {
    ($version:expr; $($model:ty),+ $(,)?) => {{
        let mut sql = ::std::vec::Vec::<::std::string::String>::new();
        $(
            sql.extend($crate::model_setup_sql::<$model>($version));
        )+
        sql
    }};
}

/// Build additive SQL for multiple models between two versions.
#[macro_export]
macro_rules! d1_model_diff_sqls {
    ($from:expr, $to:expr; $($model:ty),+ $(,)?) => {{
        let mut sql = ::std::vec::Vec::<::std::string::String>::new();
        $(
            sql.extend($crate::model_diff_sql::<$model>($from, $to));
        )+
        sql
    }};
}

/// Automatically build migration list from model schemas.
///
/// Forms:
/// - `d1_auto_migrations!(ModelA, ModelB, ...)` (auto max version)
/// - `d1_auto_migrations!(max_version; ModelA, ModelB, ...)`
#[macro_export]
macro_rules! d1_auto_migrations {
    ($($model:ty),+ $(,)?) => {{
        let mut __max_version: i32 = 0;
        $(
            __max_version = ::std::cmp::max(__max_version, <$model as $crate::Model>::latest_version());
        )+
        $crate::d1_auto_migrations!(@with_max __max_version; $($model),+)
    }};
    ($max_version:expr; $($model:ty),+ $(,)?) => {{
        $crate::d1_auto_migrations!(@with_max $max_version; $($model),+)
    }};
    (@with_max $max_version:expr; $($model:ty),+ $(,)?) => {{
        let mut __migrations = ::std::vec::Vec::<$crate::Migration>::new();
        let __max: i32 = $max_version;
        let mut __version: i32 = 1;
        while __version <= __max {
            let mut __sql = ::std::vec::Vec::<::std::string::String>::new();
            let mut __infer = ::std::vec::Vec::<$crate::SchemaProbe>::new();
            $(
                __sql.extend($crate::model_step_sql::<$model>(__version - 1, __version));
                __infer.extend($crate::model_step_probes::<$model>(__version - 1, __version));
            )+
            if !__sql.is_empty() {
                __migrations.push($crate::Migration {
                    version: __version,
                    sql: __sql,
                    infer_when: __infer,
                });
            }
            __version += 1;
        }
        __migrations
    }};
}

/// Runs schema migrations and persists applied versions in `schema_migrations`.
pub struct Migrator<'a> {
    db: &'a D1Database,
}

#[derive(Debug, Deserialize)]
struct AppliedMigration {
    version: i32,
}

#[derive(Debug, Deserialize)]
struct ExistsRow {
    found: i32,
}

impl<'a> Migrator<'a> {
    /// Create a migrator from a D1 database handle.
    pub fn new(db: &'a D1Database) -> Self {
        Self { db }
    }

    /// Run all migrations in ascending version order.
    ///
    /// Already-applied or inferred migrations are recorded and skipped.
    pub async fn run(&self, migrations: &[Migration], applied_at: i64) -> Result<()> {
        self.ensure_migration_table().await?;
        let mut applied = self.applied_versions().await?;
        let mut ordered = migrations.to_vec();
        ordered.sort_by_key(|m| m.version);

        let mut last_version: Option<i32> = None;
        for migration in &ordered {
            if last_version == Some(migration.version) {
                return Err(Error::Database(format!(
                    "Duplicate migration version: {}",
                    migration.version
                )));
            }
            last_version = Some(migration.version);
        }

        for migration in &ordered {
            let already_applied = applied.contains(&migration.version);
            let inferred = !migration.infer_when.is_empty() && self.is_inferred(migration).await?;

            if already_applied || inferred {
                self.record_migration(migration.version, applied_at).await?;
                applied.insert(migration.version);
                continue;
            }

            self.execute_migration(migration.version, &migration.sql, applied_at)
                .await?;
            applied.insert(migration.version);
        }

        Ok(())
    }

    async fn ensure_migration_table(&self) -> Result<()> {
        self.db
            .prepare(
                "CREATE TABLE IF NOT EXISTS schema_migrations (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    version INTEGER NOT NULL UNIQUE,
                    applied_at INTEGER NOT NULL
                )",
            )
            .run()
            .await?;
        Ok(())
    }

    async fn applied_versions(&self) -> Result<HashSet<i32>> {
        let result = self
            .db
            .prepare("SELECT version FROM schema_migrations")
            .all()
            .await?;
        let rows: Vec<AppliedMigration> = result.results()?;
        Ok(rows.into_iter().map(|row| row.version).collect())
    }

    async fn is_inferred(&self, migration: &Migration) -> Result<bool> {
        for probe in &migration.infer_when {
            if !self.probe_exists(probe).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    async fn probe_exists(&self, probe: &SchemaProbe) -> Result<bool> {
        match probe {
            SchemaProbe::Table(table) => self.table_exists(table).await,
            SchemaProbe::Column { table, column } => self.column_exists(table, column).await,
            SchemaProbe::Index(index) => self.index_exists(index).await,
        }
    }

    async fn table_exists(&self, table: &str) -> Result<bool> {
        let row = self
            .db
            .prepare("SELECT 1 AS found FROM sqlite_master WHERE type = 'table' AND name = ?")
            .bind(&[JsValue::from_str(table)])?
            .first::<ExistsRow>(None)
            .await?;
        Ok(row.map(|r| r.found == 1).unwrap_or(false))
    }

    async fn index_exists(&self, index: &str) -> Result<bool> {
        let row = self
            .db
            .prepare("SELECT 1 AS found FROM sqlite_master WHERE type = 'index' AND name = ?")
            .bind(&[JsValue::from_str(index)])?
            .first::<ExistsRow>(None)
            .await?;
        Ok(row.map(|r| r.found == 1).unwrap_or(false))
    }

    async fn column_exists(&self, table: &str, column: &str) -> Result<bool> {
        let sql = format!(
            "SELECT 1 AS found FROM pragma_table_info('{}') WHERE name = ? LIMIT 1",
            table
        );
        let row = self
            .db
            .prepare(&sql)
            .bind(&[JsValue::from_str(column)])?
            .first::<ExistsRow>(None)
            .await?;
        Ok(row.map(|r| r.found == 1).unwrap_or(false))
    }

    async fn record_migration(&self, version: i32, applied_at: i64) -> Result<()> {
        self.db
            .prepare("INSERT OR IGNORE INTO schema_migrations (version, applied_at) VALUES (?, ?)")
            .bind(&[
                JsValue::from_f64(version as f64),
                JsValue::from_f64(applied_at as f64),
            ])?
            .run()
            .await?;
        Ok(())
    }

    async fn execute_migration(
        &self,
        version: i32,
        queries: &[String],
        applied_at: i64,
    ) -> Result<()> {
        let mut statements = Vec::with_capacity(queries.len() + 1);
        for query in queries {
            statements.push(self.db.prepare(query));
        }
        statements.push(
            self.db
                .prepare(
                    "INSERT OR IGNORE INTO schema_migrations (version, applied_at) VALUES (?, ?)",
                )
                .bind(&[
                    JsValue::from_f64(version as f64),
                    JsValue::from_f64(applied_at as f64),
                ])?,
        );
        self.db.batch(statements).await?;
        Ok(())
    }
}
