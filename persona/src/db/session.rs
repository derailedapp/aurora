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

use serde::Serialize;
use sqlx::{SqlitePool, prelude::FromRow};
use uuid7::uuid7;

use crate::error::Error;

use super::account::Account;

#[derive(FromRow, Serialize, Clone)]
pub struct Session {
    pub id: String,
    pub account_id: String,
}

impl Session {
    pub async fn from_account(account: &Account, db: &SqlitePool) -> Result<Self, Error> {
        let id = uuid7().to_string();
        Ok(sqlx::query_as!(
            Session,
            "INSERT INTO sessions (id, account_id) VALUES ($1, $2) RETURNING *;",
            id,
            account.id
        )
        .fetch_one(db)
        .await?)
    }
}
