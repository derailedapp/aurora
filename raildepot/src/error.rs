use vodozemac::{KeyError, SignatureError};

#[derive(thiserror::Error, Debug, axum_thiserror::ErrorStatus)]
pub enum Error {
    #[error("Invalid Signature")]
    #[status(401)]
    InvalidKey(#[from] KeyError),

    #[error("Invalid Signature")]
    #[status(401)]
    InvalidSignature(#[from] SignatureError),

    #[error("Invalid Signature")]
    #[status(401)]
    BadSignature,

    #[error("No signature present")]
    #[status(401)]
    NoSignature,

    #[error("Public key field is empty")]
    #[status(400)]
    PublicKeysEmpty,

    #[error("Localhost is not a valid domain")]
    #[status(400)]
    LocalhostInvalid,

    #[error("Invalid JSON object")]
    #[status(400)]
    InvalidJSON(#[from] serde_json::Error),

    #[error("Internal Server Error")]
    #[status(500)]
    SQLiteError(#[from] sqlx::Error),

    #[error("Invalid Timestamp")]
    #[status(400)]
    InvalidTimestamp,
}
