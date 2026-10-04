use crate::types::DatabaseValue;
use crate::{DatabaseExecutor, Error, MigrationInfo, MigrationMeta, Query};
use std::borrow::Cow;

const DEFAULT_MIGRATION_TABLE: &str = "_d1_migrations";

/// Represents a single database migration version.
#[derive(Clone)]
pub struct Migration<Q> {
    version: u32,
    description: &'static str,
    steps: Vec<Q>,
}

impl<Q> Migration<Q> {
    /// Creates a new migration with the given version, description, and steps.
    pub const fn new(version: u32, description: &'static str, steps: Vec<Q>) -> Self {
        Self {
            version,
            description,
            steps,
        }
    }
}

// --- Internal Queries ---

struct CheckTableQuery<'a> {
    table: &'a str,
}

impl<'a> Query for CheckTableQuery<'a> {
    fn build(&self) -> Result<(Cow<'static, str>, Vec<DatabaseValue>), Error> {
        Ok((
            Cow::Borrowed("SELECT 1 FROM sqlite_master WHERE type='table' AND name = ?"),
            vec![self.table.into()],
        ))
    }
}

struct CheckIndexQuery<'a> {
    index: &'a str,
}

impl<'a> Query for CheckIndexQuery<'a> {
    fn build(&self) -> Result<(Cow<'static, str>, Vec<DatabaseValue>), Error> {
        Ok((
            Cow::Borrowed("SELECT 1 FROM sqlite_master WHERE type='index' AND name = ?"),
            vec![self.index.into()],
        ))
    }
}

struct CheckColumnQuery<'a> {
    table: &'a str,
    column: &'a str,
}

impl<'a> Query for CheckColumnQuery<'a> {
    fn build(&self) -> Result<(Cow<'static, str>, Vec<DatabaseValue>), Error> {
        Ok((
            Cow::Borrowed("SELECT 1 FROM pragma_table_info(?) WHERE name = ?"),
            vec![self.table.into(), self.column.into()],
        ))
    }
}

struct CreateMigrationTableQuery {
    table_name: String,
}

impl Query for CreateMigrationTableQuery {
    fn build(&self) -> Result<(Cow<'static, str>, Vec<DatabaseValue>), Error> {
        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {} (version INTEGER PRIMARY KEY, applied_at INTEGER NOT NULL)",
            self.table_name
        );
        Ok((Cow::Owned(sql), vec![]))
    }
}

struct GetCurrentVersionQuery {
    table_name: String,
}

impl Query for GetCurrentVersionQuery {
    fn build(&self) -> Result<(Cow<'static, str>, Vec<DatabaseValue>), Error> {
        let sql = format!("SELECT MAX(version) as ver FROM {}", self.table_name);
        Ok((Cow::Owned(sql), vec![]))
    }
}

struct InsertVersionQuery {
    table_name: String,
    version: u32,
}

impl Query for InsertVersionQuery {
    fn build(&self) -> Result<(Cow<'static, str>, Vec<DatabaseValue>), Error> {
        let sql = format!(
            "INSERT INTO {} (version, applied_at) VALUES (?, strftime('%s', 'now'))",
            self.table_name
        );
        Ok((Cow::Owned(sql), vec![self.version.into()]))
    }
}

#[derive(serde::Deserialize)]
struct VersionResult {
    ver: Option<u32>,
}

// --- Helpers ---

async fn check_table_exists<D>(db: &D, table: &str) -> Result<bool, Error>
where
    D: DatabaseExecutor,
{
    let q = CheckTableQuery { table };
    // We try to deserialize into a generic JSON Value because check queries return diverse shapes
    // (scalar 1 or object { "1": 1 }) depending on the backend implementation detail.
    // For D1/SqliteExecutor, query_first usually returns an Option<T>.
    // If the query returns "SELECT 1 ...", D1 might return { "1": 1 } or just 1.
    // Safest bet is to check if *any* row is returned.
    let res: Option<serde_json::Value> = db.query_first(q).await?;
    Ok(res.is_some())
}

async fn check_index_exists<D>(db: &D, index: &str) -> Result<bool, Error>
where
    D: DatabaseExecutor,
{
    let q = CheckIndexQuery { index };
    let res: Option<serde_json::Value> = db.query_first(q).await?;
    Ok(res.is_some())
}

async fn check_column_exists<D>(db: &D, table: &str, column: &str) -> Result<bool, Error>
where
    D: DatabaseExecutor,
{
    let q = CheckColumnQuery { table, column };
    let res: Option<serde_json::Value> = db.query_first(q).await?;
    Ok(res.is_some())
}

/// Executes database migrations.
///
/// # Arguments
///
/// * `db` - The database executor.
/// * `migrations` - A list of migrations to apply.
/// * `migration_table` - Optional custom name for the migration tracking table. Defaults to `_d1_migrations`.
/// * `logger` - Optional callback for logging migration progress.
pub async fn migrate<D, Q, I, F>(
    db: &D,
    migrations: I,
    migration_table: Option<&str>,
    logger: Option<F>,
) -> Result<(), Error>
where
    D: DatabaseExecutor,
    Q: Query + MigrationMeta + Clone, // Clone needed for iterating steps
    I: IntoIterator<Item = Migration<Q>>,
    F: Fn(&str),
{
    let table_name = migration_table
        .unwrap_or(DEFAULT_MIGRATION_TABLE)
        .to_string();

    // 1. Ensure migration table exists
    db.execute(CreateMigrationTableQuery {
        table_name: table_name.clone(),
    })
    .await?;

    // 2. Get current version
    let version_result: Option<VersionResult> = db
        .query_first(GetCurrentVersionQuery {
            table_name: table_name.clone(),
        })
        .await?;
    let current_ver = version_result.and_then(|r| r.ver).unwrap_or(0);

    for migration in migrations {
        if migration.version <= current_ver {
            continue;
        }

        if let Some(log) = &logger {
            log(&format!(
                "Applying migration v{}: {}",
                migration.version, migration.description
            ));
        }

        for step in migration.steps {
            let info = step.migration_info();
            let should_execute = match info {
                Some(MigrationInfo::Table(name)) => !check_table_exists(db, name).await?,
                Some(MigrationInfo::Index(name)) => !check_index_exists(db, name).await?,
                Some(MigrationInfo::Column { table, column }) => {
                    !check_column_exists(db, table, column).await?
                }
                None => true, // Always execute if no metadata provided
            };

            if should_execute {
                if let Some(log) = &logger {
                    if let Some(info) = info {
                        log(&format!("  -> Executing step for {:?}", info));
                    } else {
                        log("  -> Executing raw step");
                    }
                }
                db.execute(step.clone()).await?;
            } else if let Some(log) = &logger {
                log(&format!("  -> Skipping step (already exists): {:?}", info));
            }
        }

        // 3. Update version
        db.execute(InsertVersionQuery {
            table_name: table_name.clone(),
            version: migration.version,
        })
        .await?;
    }

    Ok(())
}
