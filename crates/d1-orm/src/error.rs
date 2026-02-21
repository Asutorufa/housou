use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Worker error: {0}")]
    Worker(#[from] worker::Error),
    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Record not found")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, Error>;
