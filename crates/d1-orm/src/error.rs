use thiserror::Error;

/// Error type used by `d1-orm`.
#[derive(Error, Debug)]
pub enum Error {
    /// Underlying Cloudflare Worker/D1 SDK error.
    #[error("Worker error: {0}")]
    Worker(#[from] worker::Error),
    /// JSON serialization/deserialization error.
    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
    /// Generic database-level error message.
    #[error("Database error: {0}")]
    Database(String),
    /// Requested record was not found.
    #[error("Record not found")]
    NotFound,
}

/// Standard result alias for `d1-orm`.
pub type Result<T> = std::result::Result<T, Error>;
