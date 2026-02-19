use thiserror::Error;

#[derive(Error, Debug)]
pub enum PasskeyError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Invalid challenge")]
    InvalidChallenge,

    #[error("Registration session expired or invalid")]
    RegistrationSessionExpired,

    #[error("Login session expired or invalid")]
    LoginSessionExpired,

    #[error("Origin mismatch: expected {expected}, got {got}")]
    OriginMismatch { expected: String, got: String },

    #[error("Invalid operation type")]
    InvalidOperationType,

    #[error("RP ID Hash mismatch")]
    RpIdHashMismatch,

    #[error("User Present flag not set")]
    UserPresentFlagNotSet,

    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("Passkey not found")]
    PasskeyNotFound,

    #[error("User Handle mismatch")]
    UserHandleMismatch,

    #[error("Signature counter regression")]
    SignatureCounterRegression,

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Base64 decode error: {0}")]
    Base64Error(#[from] base64::DecodeError),

    #[error("Internal error: {0}")]
    InternalError(String),
}

pub type Result<T> = std::result::Result<T, PasskeyError>;
