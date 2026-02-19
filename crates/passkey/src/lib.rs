pub mod store;
pub mod types;
pub mod error;
pub mod protocol;

pub use store::PasskeyStore;
pub use error::PasskeyError;
pub use protocol::{
    start_registration, finish_registration,
    start_login, finish_login,
};

#[cfg(test)]
mod tests;
