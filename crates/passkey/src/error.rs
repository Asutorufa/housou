use std::fmt;

#[derive(Debug)]
pub enum PasskeyError {
    DatabaseError(String),
    InvalidChallenge,
    RegistrationSessionExpired,
    LoginSessionExpired,
    OriginMismatch { expected: String, got: String },
    InvalidOperationType,
    RpIdHashMismatch,
    UserPresentFlagNotSet,
    InvalidSignature(String),
    PasskeyNotFound,
    UserHandleMismatch,
    SignatureCounterRegression,
    SerializationError(serde_json::Error),
    Base64Error(base64::DecodeError),
    InternalError(String),
}

impl fmt::Display for PasskeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DatabaseError(e) => write!(f, "Database error: {}", e),
            Self::InvalidChallenge => write!(f, "Invalid challenge"),
            Self::RegistrationSessionExpired => {
                write!(f, "Registration session expired or invalid")
            }
            Self::LoginSessionExpired => write!(f, "Login session expired or invalid"),
            Self::OriginMismatch { expected, got } => {
                write!(f, "Origin mismatch: expected {}, got {}", expected, got)
            }
            Self::InvalidOperationType => write!(f, "Invalid operation type"),
            Self::RpIdHashMismatch => write!(f, "RP ID Hash mismatch"),
            Self::UserPresentFlagNotSet => write!(f, "User Present flag not set"),
            Self::InvalidSignature(e) => write!(f, "Invalid signature: {}", e),
            Self::PasskeyNotFound => write!(f, "Passkey not found"),
            Self::UserHandleMismatch => write!(f, "User Handle mismatch"),
            Self::SignatureCounterRegression => write!(f, "Signature counter regression"),
            Self::SerializationError(e) => write!(f, "Serialization error: {}", e),
            Self::Base64Error(e) => write!(f, "Base64 decode error: {}", e),
            Self::InternalError(e) => write!(f, "Internal error: {}", e),
        }
    }
}

impl std::error::Error for PasskeyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::SerializationError(e) => Some(e),
            Self::Base64Error(e) => Some(e),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for PasskeyError {
    fn from(err: serde_json::Error) -> Self {
        PasskeyError::SerializationError(err)
    }
}

impl From<base64::DecodeError> for PasskeyError {
    fn from(err: base64::DecodeError) -> Self {
        PasskeyError::Base64Error(err)
    }
}

pub type Result<T> = std::result::Result<T, PasskeyError>;
