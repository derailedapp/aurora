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

use aurora_db::user::User;
use axum::{
    Json,
    http::{HeaderMap, StatusCode},
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::{ErrorMessage, OVTError};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,
    pub iat: usize,
    pub sub: String,
}

impl Claims {
    pub fn make_token(&self, key: &EncodingKey) -> Result<String, OVTError> {
        encode(&Header::new(jsonwebtoken::Algorithm::HS256), self, key)
            .map_err(|_| OVTError::InternalServerError)
    }

    pub fn from_token(token: &str, key: &DecodingKey) -> Result<Self, OVTError> {
        Ok(
            decode::<Self>(token, key, &Validation::new(jsonwebtoken::Algorithm::HS256))
                .map_err(|_| OVTError::InvalidToken)?
                .claims,
        )
    }

    pub fn from_token_map(
        map: &HeaderMap,
        key: &DecodingKey,
    ) -> Result<Self, (StatusCode, Json<ErrorMessage>)> {
        if let Some(token) = map.get("authorization") {
            Self::from_token(
                token
                    .to_str()
                    .map_err(|_| OVTError::InternalServerError.to_resp())?,
                key,
            )
            .map_err(|err| err.to_resp())
        } else {
            Err(OVTError::InvalidToken.to_resp())
        }
    }
}

pub async fn get_user(
    map: &HeaderMap,
    key: &str,
    db: &PgPool,
) -> Result<User, (StatusCode, Json<ErrorMessage>)> {
    let claims = Claims::from_token_map(map, &DecodingKey::from_secret(key.as_bytes()))?;

    if let Some(user) = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE id IN (SELECT user_id FROM sessions WHERE id = $1);",
        claims.sub
    )
    .fetch_optional(db)
    .await
    .map_err(|_| OVTError::InternalServerError.to_resp())?
    {
        Ok(user)
    } else {
        Err(OVTError::ExpiredSession.to_resp())
    }
}
