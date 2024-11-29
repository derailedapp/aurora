// Copyright (C) 2024 V.J. De Chico
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use axum::{http::StatusCode, Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorMessage {
    pub message: String,
    pub code: i32,
}

#[derive(Debug)]
pub enum OVTError {
    InternalServerError,
    InvalidEmailOrPassword,
    InvalidToken,
    ExpiredSession,
    GuildNotFound,
    InvalidPermissions,
}

impl OVTError {
    pub fn to_resp(&self) -> (StatusCode, Json<ErrorMessage>) {
        match self {
            Self::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorMessage {
                    message: "Internal Server Error".to_string(),
                    code: 0,
                }),
            ),
            Self::InvalidEmailOrPassword => (
                StatusCode::BAD_REQUEST,
                Json(ErrorMessage {
                    message: "Invalid email or password".to_string(),
                    code: 1,
                }),
            ),
            Self::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                Json(ErrorMessage {
                    message: "Invalid session token".to_string(),
                    code: 2,
                }),
            ),
            Self::ExpiredSession => (
                StatusCode::UNAUTHORIZED,
                Json(ErrorMessage {
                    message: "Session has expired or does not exist anymore".to_string(),
                    code: 3,
                }),
            ),
            Self::GuildNotFound => (
                StatusCode::NOT_FOUND,
                Json(ErrorMessage {
                    message: "Guild not found".to_string(),
                    code: 4,
                }),
            ),
            Self::InvalidPermissions => (
                StatusCode::FORBIDDEN,
                Json(ErrorMessage {
                    message: "Invalid permissions".to_string(),
                    code: 5,
                }),
            ),
        }
    }
}
