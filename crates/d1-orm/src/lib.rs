pub mod error;
pub mod query;
pub mod traits;

pub use error::{Error, Result};
pub use query::{Select, Insert, Update, Delete, Order, Bindable};
pub use traits::{Model, Repository};

// Re-export useful worker types
pub use worker::D1Database;
pub use worker::wasm_bindgen::JsValue;

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
            fn primary_key() -> &'static str { $pk }
        }
    };
}
