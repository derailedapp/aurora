use std::time::Duration;

use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use axum::{Json, extract::State, routing::post};
use jsonwebtoken::EncodingKey;
use serde::{Deserialize, Serialize};
use serde_valid::Validate;
use sqlx::types::chrono;

use crate::{
    db::{account::Account, actor::Actor, session::Session},
    error::Error,
    token::Claims,
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

pub fn router() -> axum::Router<crate::state::State> {
    axum::Router::new().route("/users", post(create_user))
}
