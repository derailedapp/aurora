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
use sqlx::SqlitePool;
use sqlx::prelude::FromRow;

use crate::db::account::Account;
use crate::error::Error;

#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct Actor {
    pub id: String,
    pub display_name: Option<String>,
    pub avatar: Option<String>,
    pub banner: Option<String>,
    pub bio: Option<String>,
    pub status: Option<String>,
}

impl Actor {
    pub async fn from_account(account: &Account, db: &SqlitePool) -> Result<Self, Error> {
        Ok(sqlx::query_as!(
            Actor,
            "INSERT INTO actors (id) VALUES ($1) RETURNING *",
            account.id,
        )
        .fetch_one(db)
        .await?)
    }
}
