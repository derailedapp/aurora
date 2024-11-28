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
use sqlx::prelude::FromRow;

use crate::{DBError, FromId, FromIdResult};

#[derive(FromRow, Serialize)]
pub struct Guild {
    pub id: String,
    pub owner_id: String,
    pub name: String,
    #[sqlx(default)]
    #[serde(skip_serializing)]
    pub server_id: Option<String>,
    #[sqlx(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<i64>,
}

impl FromId<String> for Guild {
    async fn from_id(db: &sqlx::PgPool, id: String) -> FromIdResult<Self> {
        sqlx::query_as!(Guild, "SELECT * FROM guilds WHERE id = $1;", id)
            .fetch_one(db)
            .await
            .map_err(|_| DBError::RowNotFound)
    }
}
