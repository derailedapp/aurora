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
