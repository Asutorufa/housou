use crate::types::DatabaseValue;
use crate::{DatabaseExecutor, Error, MigrationInfo, MigrationMeta, Query};
use std::borrow::Cow;

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

async fn check_table_exists<D>(db: &D, table: &str) -> Result<bool, Error>
where
    D: DatabaseExecutor,
{
    let q = CheckTableQuery { table };
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

/// Executes a series of migration steps.
///
/// This function iterates through the provided migration steps, checks if the
/// corresponding database object (table, index, or column) already exists, and
/// executes the step if it does not.
///
/// It uses `println!` to log progress. For custom logging, use [`migrate_with_logger`].
pub async fn migrate<D, S, I>(db: &D, steps: I) -> Result<(), Error>
where
    D: DatabaseExecutor,
    S: Query + MigrationMeta,
    I: IntoIterator<Item = S>,
{
    migrate_with_logger(db, steps, |msg| println!("{}", msg)).await
}

/// Executes a series of migration steps with a custom logger.
///
/// This function iterates through the provided migration steps, checks if the
/// corresponding database object (table, index, or column) already exists, and
/// executes the step if it does not.
///
/// The `logger` callback is invoked with a message for each executed migration step.
pub async fn migrate_with_logger<D, S, I, F>(db: &D, steps: I, logger: F) -> Result<(), Error>
where
    D: DatabaseExecutor,
    S: Query + MigrationMeta,
    I: IntoIterator<Item = S>,
    F: Fn(&str),
{
    for step in steps {
        let info = step
            .migration_info()
            .ok_or_else(|| Error::Other("Migration step missing metadata".to_string()))?;

        let exists = match info {
            MigrationInfo::Table(name) => check_table_exists(db, name).await?,
            MigrationInfo::Index(name) => check_index_exists(db, name).await?,
            MigrationInfo::Column { table, column } => check_column_exists(db, table, column).await?,
        };

        if !exists {
            logger(&format!("Applying migration: {:?}", info));
            db.execute(step).await?;
        }
    }
    Ok(())
}
