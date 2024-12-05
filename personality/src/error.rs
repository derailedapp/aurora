use reqwest::header::ToStrError;

#[derive(thiserror::Error, Debug, axum_thiserror::ErrorStatus)]
pub enum Error {
    #[error("Internal Server Error")]
    #[status(500)]
    DBError(#[from] sqlx::Error),

    #[error("Internal Server Error")]
    #[status(500)]
    Migrate(#[from] sqlx::migrate::MigrateError),

    #[error("Internal Server Error")]
    #[status(500)]
    StrError(#[from] ToStrError),

    #[error("Internal Server Error")]
    #[status(500)]
    FailedPasswordHash,

    #[error("Invalid Token")]
    #[status(401)]
    InvalidToken(#[from] jsonwebtoken::errors::Error),

    #[error("Invalid Token")]
    #[status(401)]
    BadToken,

    #[error("Expired session")]
    #[status(401)]
    ExpiredSession,
}
