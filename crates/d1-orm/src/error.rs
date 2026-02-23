#[derive(Debug)]
pub enum Error {
    Build(String),
    Param(String),
    #[cfg(feature = "d1")]
    D1(worker::Error),
    #[cfg(feature = "sqlite")]
    Sqlite(rusqlite::Error),
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Build(msg) => write!(f, "Build Error: {}", msg),
            Error::Param(msg) => write!(f, "Parameter Error: {}", msg),
            #[cfg(feature = "d1")]
            Error::D1(e) => write!(f, "D1 Error: {}", e),
            #[cfg(feature = "sqlite")]
            Error::Sqlite(e) => write!(f, "Sqlite Error: {}", e),
            Error::Other(msg) => write!(f, "Other Error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(feature = "d1")]
impl From<worker::Error> for Error {
    fn from(e: worker::Error) -> Self {
        Error::D1(e)
    }
}

#[cfg(feature = "sqlite")]
impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Error::Sqlite(e)
    }
}
