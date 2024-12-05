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
