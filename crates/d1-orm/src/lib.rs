pub mod error;
pub mod query;
pub mod schema;
mod schema_tests;
pub mod traits;

pub use d1_orm_derive::Model;
pub use error::{Error, Result};
pub use query::{Bindable, Delete, Insert, Order, Select, Update};
pub use schema::{AlterTable, Column, ColumnType, Constraint, Index, Table};
pub use traits::{Model, Repository};

// Re-export useful worker types
pub use worker::wasm_bindgen::JsValue;
pub use worker::D1Database;

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
