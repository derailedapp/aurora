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

use vodozemac::{KeyError, SignatureError};

#[allow(clippy::enum_variant_names)]
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

    #[error("Maximum public keys set")]
    #[status(400)]
    MaximumPublicKeys,
}
