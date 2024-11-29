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

use std::time::Duration;

use argon2::{
    Argon2, PasswordHash, PasswordVerifier,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use aurora_db::user::User;
use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use jsonwebtoken::EncodingKey;
use serde::{Deserialize, Serialize};
use serde_valid::Validate;

use crate::{
    error::{ErrorMessage, OVTError},
    state::OVTState,
    token::Claims,
};

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUser {
    #[validate(pattern = r"^[a-b0-9_-]+$")]
    #[validate(min_length = 3)]
    #[validate(max_length = 32)]
    username: String,
    #[validate(pattern = r"/^[^@\s]*?@[^@\s]*?\.[^@\s]*$/")]
    email: String,
    #[validate(min_length = 8)]
    #[validate(max_length = 128)]
    password: String,
}

#[derive(Serialize)]
pub struct TokenReturn {
    token: String,
}

pub async fn register(
    State(state): State<OVTState>,
    Json(model): Json<CreateUser>,
) -> Result<Json<TokenReturn>, (StatusCode, Json<ErrorMessage>)> {
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(model.password.as_bytes(), &salt)
        .map_err(|_| OVTError::InternalServerError.to_resp())?
        .to_string();

    // TODO: append server identifier here
    // user_id: `@!user_id@example.com`
    // username: `@username@example.com`
    let user_id = uuid7::uuid7().to_string();
    let session_id = uuid7::uuid7().to_string();

    let mut tx = state
        .pg
        .begin()
        .await
        .map_err(|_| OVTError::InternalServerError.to_resp())?;

    sqlx::query!(
        "INSERT INTO users (id, username, email, password, flags) VALUES ($1, $2, $3, $4, 0)",
        &user_id,
        &model.username,
        &model.email,
        password_hash
    )
    .execute(&mut *tx)
    .await
    .map_err(|_| OVTError::InternalServerError.to_resp())?;
    sqlx::query!(
        "INSERT INTO user_settings (id, theme) VALUES ($1, 'dark')",
        &user_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|_| OVTError::InternalServerError.to_resp())?;
    sqlx::query!(
        "INSERT INTO sessions (id, user_id) VALUES ($1, $2)",
        &user_id,
        &session_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|_| OVTError::InternalServerError.to_resp())?;

    let time = chrono::Utc::now().timestamp_micros() as u128;

    let claims = Claims {
        sub: session_id,
        exp: (time + Duration::from_weeks(6).as_micros()) as usize,
        iat: time as usize,
    };

    let result = Ok(Json(TokenReturn {
        token: claims
            .make_token(&EncodingKey::from_secret(state.key.as_bytes()))
            .map_err(|err| err.to_resp())?,
    }));

    tx.commit()
        .await
        .map_err(|_| OVTError::InternalServerError.to_resp())?;

    result
}

#[derive(Debug, Deserialize, Validate)]
pub struct Login {
    #[validate(pattern = r"/^[^@\s]*?@[^@\s]*?\.[^@\s]*$/")]
    email: String,
    #[validate(min_length = 8)]
    #[validate(max_length = 128)]
    password: String,
}

pub async fn login(
    State(state): State<OVTState>,
    Json(model): Json<Login>,
) -> Result<Json<TokenReturn>, (StatusCode, Json<ErrorMessage>)> {
    let argon2 = Argon2::default();

    let maybe_user = sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1;", model.email)
        .fetch_optional(&state.pg)
        .await
        .map_err(|_| OVTError::InternalServerError.to_resp())?;

    if let Some(user) = maybe_user {
        if argon2
            .verify_password(
                model.password.as_bytes(),
                &PasswordHash::new(user.password.unwrap().as_str()).unwrap(),
            )
            .is_err()
        {
            return Err(OVTError::InvalidEmailOrPassword.to_resp());
        }

        let session_id = uuid7::uuid7().to_string();

        sqlx::query!(
            "INSERT INTO sessions (id, user_id) VALUES ($1, $2)",
            &user.id,
            &session_id
        )
        .execute(&state.pg)
        .await
        .map_err(|_| OVTError::InternalServerError.to_resp())?;

        let time = chrono::Utc::now().timestamp_micros() as u128;

        let claims = Claims {
            sub: session_id,
            exp: (time + Duration::from_weeks(6).as_micros()) as usize,
            iat: time as usize,
        };

        Ok(Json(TokenReturn {
            token: claims
                .make_token(&EncodingKey::from_secret(state.key.as_bytes()))
                .map_err(|err| err.to_resp())?,
        }))
    } else {
        Err(OVTError::InvalidEmailOrPassword.to_resp())
    }
}

pub fn router() -> Router<OVTState> {
    Router::<OVTState>::new()
        .route("/register", post(register))
        .route("/login", post(login))
}
