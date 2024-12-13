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

use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, prelude::FromRow};

use crate::{depot::create_identifier, error::Error};

use super::tent::get_user_db;

#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct Account {
    pub id: String,
    pub email: String,
    #[serde(skip)]
    pub password: String,
    pub flags: i64,
    pub theme: String,
    pub pickle: String,
    pub ed_key: String,
}

impl Account {
    pub async fn create_default(
        state: &crate::state::State,
        email: String,
        password: String,
    ) -> Result<(Self, SqlitePool), Error> {
        let key = vodozemac::Ed25519SecretKey::new();
        let pub_key = key.public_key();
        let ed_key = key.to_base64();

        let id = create_identifier(state, pub_key).await;
        let pool = get_user_db(&id).await?;

        let pick = vodozemac::olm::Account::new()
            .pickle()
            .encrypt(id.as_bytes().try_into().unwrap());

        Ok((sqlx::query_as!(
            Account,
            "INSERT INTO accounts (id, email, password, flags, theme, pickle, ed_key) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *",
            id,
            email,
            password,
            0i64,
            "dark",
            pick,
            ed_key
        ).fetch_one(&pool).await?, pool))
    }
}
