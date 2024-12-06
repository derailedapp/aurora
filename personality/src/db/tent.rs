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

use std::env;

use sqlx::migrate;
use sqlx::{Sqlite, SqlitePool, migrate::MigrateDatabase, sqlite::SqlitePoolOptions};

use crate::error::Error;

pub async fn get_user_db(id: &str) -> Result<SqlitePool, Error> {
    let uri = format!(
        "sqlite:/{}/{}",
        env::var("BASE_DB_PATH").expect("Couldn't find a path for SQLite database store"),
        id
    );
    let exists = Sqlite::database_exists(&uri).await?;

    if !exists {
        Sqlite::create_database(&uri).await?;
    }

    let pool = SqlitePoolOptions::new().connect(&uri).await?;
    migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

pub async fn clean_get_user_db(id: &str) -> Result<SqlitePool, Error> {
    let uri = "sqlite:/".to_string()
        + &env::var("BASE_DB_PATH").expect("Couldn't find a path for SQLite database store")
        + "/"
        + id;
    let exists = Sqlite::database_exists(&uri).await?;

    if !exists {
        return Err(Error::DatabaseNotExists);
    }

    let pool = SqlitePoolOptions::new().connect(&uri).await?;
    migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

pub async fn delete_user_db(id: &str) -> Result<(), Error> {
    let uri = "sqlite:/".to_string()
        + &env::var("BASE_DB_PATH").expect("Couldn't find a path for SQLite database store")
        + "/"
        + id;
    let exists = Sqlite::database_exists(&uri).await?;

    if !exists {
        return Err(Error::DatabaseNotExists);
    }

    Sqlite::drop_database(&uri).await?;

    Ok(())
}
