use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::error::{Error, Result};
use worker::D1Database;
use worker::wasm_bindgen::JsValue;
use crate::query::{Select, Insert, Update, Delete, Bindable};

#[async_trait]
pub trait Model: Sized + Serialize + DeserializeOwned + Send + Sync {
    const TABLE: &'static str;
    fn primary_key() -> &'static str { "id" }
}

pub struct Repository<'a, T> {
    pub db: &'a D1Database,
    _marker: std::marker::PhantomData<T>,
}

impl<'a, T: Model> Repository<'a, T> {
    pub fn new(db: &'a D1Database) -> Self {
        Self { db, _marker: std::marker::PhantomData }
    }

    pub fn select(&self) -> Select {
        Select::new(T::TABLE)
    }

    pub fn insert(&self) -> Insert {
        Insert::new(T::TABLE)
    }

    pub fn update(&self) -> Update {
        Update::new(T::TABLE)
    }

    pub fn delete(&self) -> Delete {
        Delete::new(T::TABLE)
    }

    pub async fn delete_by_id(&self, id: impl Into<JsValue> + Send) -> Result<()> {
        let pk = T::primary_key();
        let (sql, bindings) = Delete::new(T::TABLE)
            .where_eq(pk, id)
            .to_sql();

        let stmt = self.db.prepare(&sql).bind(&bindings)?;
        stmt.run().await.map_err(Error::from).map(|_| ())
    }

    pub async fn find_by_id(&self, id: impl Into<JsValue> + Send) -> Result<Option<T>> {
        let pk = T::primary_key();
        let (sql, bindings) = Select::new(T::TABLE)
            .where_eq(pk, id)
            .limit(1)
            .to_sql();

        let stmt = self.db.prepare(&sql).bind(&bindings)?;
        stmt.first(None).await.map_err(Into::into)
    }

    pub async fn find_one(&self, query: Select) -> Result<Option<T>> {
        let (sql, bindings) = query.limit(1).to_sql();
        let stmt = self.db.prepare(&sql).bind(&bindings)?;
        stmt.first(None).await.map_err(Into::into)
    }

    pub async fn find_all(&self, query: Select) -> Result<Vec<T>> {
        let (sql, bindings) = query.to_sql();
        let stmt = self.db.prepare(&sql).bind(&bindings)?;
        let result = stmt.all().await?;
        result.results().map_err(Into::into)
    }

    /// Execute a raw query or builder output that returns generic D1Result
    pub async fn execute(&self, bindable: impl Bindable) -> Result<worker::D1Result> {
        let (sql, bindings) = bindable.to_sql();
        let stmt = self.db.prepare(&sql).bind(&bindings)?;
        stmt.run().await.map_err(Error::from)
    }

    /// Execute an insert and return the inserted row (if supported by RETURNING)
    pub async fn insert_one(&self, insert: Insert) -> Result<Option<T>> {
        let (sql, bindings) = insert.to_sql();
        let stmt = self.db.prepare(&sql).bind(&bindings)?;
        stmt.first(None).await.map_err(Into::into)
    }
}
