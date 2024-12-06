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

use std::time::Duration;

use argon2::{
    Argon2, PasswordHash, PasswordVerifier,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use axum::{
    Json,
    extract::State,
    http::HeaderMap,
    routing::{delete, post},
};
use jsonwebtoken::EncodingKey;
use serde::{Deserialize, Serialize};
use serde_valid::Validate;
use sqlx::types::chrono;
use vodozemac::Ed25519SecretKey;

use crate::{
    db::{account::Account, actor::Actor, session::Session, tent::delete_user_db},
    depot::delete_identifier,
    error::Error,
    token::{Claims, get_user},
};

#[derive(Deserialize, Validate)]
pub struct CreateUser {
    #[validate(pattern = r"^[a-b0-9_-]+$")]
    #[validate(min_length = 3)]
    #[validate(max_length = 32)]
    email: String,
    password: String,
}

#[derive(Serialize)]
pub struct TokenResult {
    token: String,
}

pub async fn create_user(
    State(state): State<crate::state::State>,
    Json(model): Json<CreateUser>,
) -> Result<Json<TokenResult>, Error> {
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(model.password.as_bytes(), &salt)
        .map_err(|_| Error::FailedPasswordHash)?
        .to_string();

    let (account, db) = Account::create_default(&state, model.email, password_hash).await?;
    Actor::from_account(&account, &db).await?;
    let session = Session::from_account(&account, &db).await?;

    let time = chrono::Utc::now().timestamp_micros() as u128;

    let claims = Claims {
        sub: session.id,
        exp: (time + Duration::from_weeks(6).as_micros()) as usize,
        iat: time as usize,
    };

    let token = claims.make_token(&EncodingKey::from_secret(state.jwt_secret.as_bytes()))?;

    Ok(Json(TokenResult { token }))
}

#[derive(Deserialize, Validate)]
pub struct DeleteUser {
    password: String,
}

pub async fn delete_user(
    headers: HeaderMap,
    State(state): State<crate::state::State>,
    Json(model): Json<DeleteUser>,
) -> Result<String, Error> {
    let (_, account, _) = get_user(&headers, &state.jwt_secret).await?;

    let argon2 = Argon2::default();

    argon2
        .verify_password(
            model.password.as_bytes(),
            &PasswordHash::new(&account.password).map_err(|_| Error::Argon2Error)?,
        )
        .map_err(|_| Error::Argon2Error)?;

    delete_identifier(
        &state,
        &account.id,
        Ed25519SecretKey::from_base64(&account.ed_key)?,
    )
    .await?;
    delete_user_db(&account.id).await?;

    Ok("".to_string())
}

pub fn router() -> axum::Router<crate::state::State> {
    axum::Router::new()
        .route("/users", post(create_user))
        .route("/users/@me", delete(delete_user))
}
