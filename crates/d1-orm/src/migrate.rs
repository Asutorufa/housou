use crate::{D1Database, Error, Result};
use serde::Deserialize;
use std::collections::HashSet;
use worker::wasm_bindgen::JsValue;

#[derive(Debug, Clone)]
pub enum SchemaProbe {
    Table(&'static str),
    Column {
        table: &'static str,
        column: &'static str,
    },
    Index(&'static str),
}

#[derive(Debug, Clone)]
pub struct Migration {
    pub version: i32,
    pub sql: Vec<String>,
    pub infer_when: Vec<SchemaProbe>,
}

impl Migration {
    pub fn new(version: i32) -> Self {
        Self {
            version,
            sql: Vec::new(),
            infer_when: Vec::new(),
        }
    }

    pub fn with_sql(mut self, sql: impl Into<String>) -> Self {
        self.sql.push(sql.into());
        self
    }

    pub fn with_sqls<I, S>(mut self, sqls: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.sql.extend(sqls.into_iter().map(Into::into));
        self
    }

    pub fn infer_if(mut self, probe: SchemaProbe) -> Self {
        self.infer_when.push(probe);
        self
    }
}

#[macro_export]
macro_rules! d1_probe {
    (table $table:expr) => {
        $crate::SchemaProbe::Table($table)
    };
    (index $index:expr) => {
        $crate::SchemaProbe::Index($index)
    };
    (column $table:expr, $column:expr) => {
        $crate::SchemaProbe::Column {
            table: $table,
            column: $column,
        }
    };
}

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

#[macro_export]
macro_rules! d1_migrations {
    ($($migration:expr),* $(,)?) => {
        vec![$($migration),*]
    };
}

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
    pub fn new(db: &'a D1Database) -> Self {
        Self { db }
    }

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
