//! Lightweight ORM and migration helpers for Cloudflare D1.
//!
//! `d1-orm` provides:
//! - query builders (`Select`, `Insert`, `Update`, `Delete`)
//! - model/repository abstractions (`Model`, `Repository`)
//! - schema and additive migration helpers
//! - a derive macro re-export (`#[derive(Model)]`)
//!
//! This crate is designed for Worker + D1 environments.

extern crate self as d1_orm;

mod batch;
mod raw_query;
mod sets;
/// Error and result types used by this crate.
pub mod error;
/// Migration model, migrator runtime, and migration macros.
pub mod migrate;
mod migrate_tests;
/// Query builder types for CRUD SQL generation.
pub mod query;
/// Schema builder and additive migration SQL helpers.
pub mod schema;
mod schema_tests;
/// Model trait and typed repository abstraction.
pub mod traits;

/// Derive macro for `d1_orm::Model`.
pub use d1_orm_derive::Model;
/// Re-exported crate error type and result alias.
pub use error::{Error, Result};
/// Re-exported migration types.
pub use migrate::{
    model_diff_sql, model_setup_sql, model_step_probes, model_step_sql, Migration, Migrator,
    SchemaProbe,
};
/// Re-exported query builder types and trait.
pub use query::{Bindable, Delete, Insert, Order, Select, Update};
/// Re-exported schema builder types and helper.
pub use schema::{
    additive_migration_sql, AlterTable, Column, ColumnType, Constraint, Index, Table,
};
/// Re-exported model trait and repository type.
pub use traits::{Model, Repository};

/// Re-exported `wasm_bindgen::JsValue` used for statement bindings.
pub use worker::wasm_bindgen::JsValue;
/// Re-exported D1 database handle from `worker`.
pub use worker::D1Database;
/// Re-exported D1 command result type from `worker`.
pub use worker::D1Result;

/// Serialize a Rust value into `JsValue` for D1 parameter binding.
pub fn to_js_value<T: serde::Serialize>(value: &T) -> Result<JsValue> {
    let serializer = serde_wasm_bindgen::Serializer::new().serialize_missing_as_null(true);
    serde::Serialize::serialize(value, &serializer)
        .map_err(|e| Error::Database(format!("failed to serialize value into JsValue: {}", e)))
}
