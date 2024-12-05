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

use std::env;

use sqlx::migrate;
use sqlx::{Sqlite, SqlitePool, migrate::MigrateDatabase, sqlite::SqlitePoolOptions};

use crate::error::Error;

pub async fn get_user_db(id: &str) -> Result<SqlitePool, Error> {
    let uri = "sqlite:/".to_string()
        + &env::var("BASE_DB_PATH").expect("Couldn't find a path for SQLite database store")
        + "/"
        + id;
    let exists = Sqlite::database_exists(&uri).await?;

    if !exists {
        Sqlite::create_database(&uri).await?;
    }

    let pool = SqlitePoolOptions::new().connect(&uri).await?;
    migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}
