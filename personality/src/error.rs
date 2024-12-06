/*
    Copyright 2024 V.J. De Chico

    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at

        http://www.apache.org/licenses/LICENSE-2.0

    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.
*/

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
    ReqwestError(#[from] reqwest::Error),

    #[error("Internal Server Error")]
    #[status(500)]
    VodoError(#[from] vodozemac::KeyError),

    #[error("Internal Server Error")]
    #[status(500)]
    FailedPasswordHash,

    #[error("Internal Server Error")]
    #[status(500)]
    DatabaseNotExists,

    #[error("Invalid Token")]
    #[status(401)]
    InvalidToken(#[from] jsonwebtoken::errors::Error),

    #[error("Invalid Token")]
    #[status(401)]
    BadToken,

    #[error("Expired session")]
    #[status(401)]
    ExpiredSession,

    #[error("Invalid email or password")]
    #[status(401)]
    Argon2Error
}
