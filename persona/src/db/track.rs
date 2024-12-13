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
use sqlx::types::chrono::Utc;

use crate::db::account::Account;
use crate::error::Error;

#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct Track {
    pub id: String,
    pub author_id: String,
    pub content: String,
    pub parent_id: Option<String>,
    pub created_at: i64,
    // #[sqlx(default)]
    // pub signature: Option<String>
}

impl Track {
    pub async fn create(
        db: &SqlitePool,
        author: &Account,
        content: &str,
        parent_id: Option<String>,
    ) -> Result<Self, Error> {
        let id = format!("{}/{}", author.id, uuid7::uuid7());
        let ts = Utc::now().timestamp_millis();

        Ok(sqlx::query_as!(
            Track,
            "INSERT INTO tracks (id, author_id, content, parent_id, created_at) VALUES ($1, $2, $3, $4, $5) RETURNING *;",
            id,
            author.id,
            content,
            parent_id,
            ts
        ).fetch_one(db).await?)
    }
}
