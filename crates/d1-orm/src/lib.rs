pub mod error;
pub mod query;
pub mod traits;

pub use error::{Error, Result};
pub use query::{Bindable, Delete, Insert, Order, Select, Update};
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
