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
