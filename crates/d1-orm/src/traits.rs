use crate::error::{Error, Result};
use crate::query::{Bindable, Delete, Insert, Select, Update};
use crate::schema::{Index, Table};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use worker::wasm_bindgen::JsValue;
use worker::D1Database;

/// Trait implemented by persisted model types.
#[async_trait]
pub trait Model: Sized + Serialize + DeserializeOwned + Send + Sync {
    /// Backing table name.
    const TABLE: &'static str;

    /// Primary key column name. Defaults to `"id"`.
    fn primary_key() -> &'static str {
        "id"
    }

    /// Schema for a specific version, if the model exists in that version.
    fn schema_at(_version: i32) -> Option<Table> {
        None
    }

    /// Indexes for a specific version.
    fn indexes_at(_version: i32) -> Vec<Index> {
        Vec::new()
    }

    /// Highest schema version referenced by this model.
    fn latest_version() -> i32 {
        1
    }
}

/// Typed repository for model-centric operations against D1.
pub struct Repository<'a, T> {
    /// Database handle used by the repository.
    pub db: &'a D1Database,
    _marker: std::marker::PhantomData<T>,
}

impl<'a, T: Model> Repository<'a, T> {
    /// Create a new repository for `T`.
    pub fn new(db: &'a D1Database) -> Self {
        Self {
            db,
            _marker: std::marker::PhantomData,
        }
    }

    /// Start a `SELECT` builder scoped to `T::TABLE`.
    pub fn select(&self) -> Select {
        Select::new(T::TABLE)
    }

    /// Start an `INSERT` builder scoped to `T::TABLE`.
    pub fn insert(&self) -> Insert {
        Insert::new(T::TABLE)
    }

    /// Start an `UPDATE` builder scoped to `T::TABLE`.
    pub fn update(&self) -> Update {
        Update::new(T::TABLE)
    }

    /// Start a `DELETE` builder scoped to `T::TABLE`.
    pub fn delete(&self) -> Delete {
        Delete::new(T::TABLE)
    }

    /// Delete one row by the model primary key value.
    pub async fn delete_by_id(&self, id: impl Into<JsValue> + Send) -> Result<()> {
        let pk = T::primary_key();
        let (sql, bindings) = Delete::new(T::TABLE).where_eq(pk, id).to_sql();

        let stmt = self.db.prepare(&sql).bind(&bindings)?;
        stmt.run().await.map_err(Error::from).map(|_| ())
    }

    /// Find one row by primary key.
    pub async fn find_by_id(&self, id: impl Into<JsValue> + Send) -> Result<Option<T>> {
        let pk = T::primary_key();
        let (sql, bindings) = Select::new(T::TABLE).where_eq(pk, id).limit(1).to_sql();

        let stmt = self.db.prepare(&sql).bind(&bindings)?;
        stmt.first(None).await.map_err(Into::into)
    }

    /// Execute a `Select` and return the first row.
    pub async fn find_one(&self, query: Select) -> Result<Option<T>> {
        let (sql, bindings) = query.limit(1).to_sql();
        let stmt = self.db.prepare(&sql).bind(&bindings)?;
        stmt.first(None).await.map_err(Into::into)
    }

    /// Execute a `Select` and return all matching rows.
    pub async fn find_all(&self, query: Select) -> Result<Vec<T>> {
        let (sql, bindings) = query.to_sql();
        let stmt = self.db.prepare(&sql).bind(&bindings)?;
        let result = stmt.all().await?;
        result.results().map_err(Into::into)
    }

    /// Execute a builder and return the raw `D1Result`.
    pub async fn execute(&self, bindable: impl Bindable) -> Result<worker::D1Result> {
        let (sql, bindings) = bindable.to_sql();
        let stmt = self.db.prepare(&sql).bind(&bindings)?;
        stmt.run().await.map_err(Error::from)
    }

    /// Execute an insert and return one row (requires `RETURNING` support).
    pub async fn insert_one(&self, insert: Insert) -> Result<Option<T>> {
        let (sql, bindings) = insert.to_sql();
        let stmt = self.db.prepare(&sql).bind(&bindings)?;
        stmt.first(None).await.map_err(Into::into)
    }
}
