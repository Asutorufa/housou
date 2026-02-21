extern crate self as d1_orm;

pub mod error;
pub mod migrate;
mod migrate_tests;
mod model_tests;
pub mod query;
pub mod schema;
mod schema_tests;
pub mod traits;

pub use d1_orm_derive::Model;
pub use error::{Error, Result};
pub use migrate::{Migration, Migrator, SchemaProbe};
pub use query::{Bindable, Delete, Insert, Order, Select, Update};
pub use schema::{
    additive_migration_sql, AlterTable, Column, ColumnType, Constraint, Index, Table,
};
pub use traits::{Model, Repository};

// Re-export useful worker types
pub use worker::wasm_bindgen::JsValue;
pub use worker::D1Database;
pub use worker::D1Result;

pub fn to_js_value<T: serde::Serialize>(value: &T) -> Result<JsValue> {
    serde_wasm_bindgen::to_value(value)
        .map_err(|e| Error::Database(format!("failed to serialize value into JsValue: {}", e)))
}

#[macro_export]
macro_rules! impl_model {
    ($name:ident, $table:expr) => {
        impl $crate::traits::Model for $name {
            const TABLE: &'static str = $table;
        }
    };
    ($name:ident, $table:expr, $pk:expr) => {
        impl $crate::traits::Model for $name {
            const TABLE: &'static str = $table;
            fn primary_key() -> &'static str {
                $pk
            }
        }
    };
}
