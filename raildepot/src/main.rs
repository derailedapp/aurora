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

use std::time::Duration;
mod error;

use axum::{
    Json,
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use error::Error;
use nanoid::nanoid;
use raildepot::{CreateId, Identifier, PushPublicKeys};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use vodozemac::{Ed25519PublicKey, Ed25519Signature};

async fn verify_server_key(
    client: &reqwest::Client,
    body: Bytes,
    signature: String,
    server: &str,
) -> Result<(), Error> {
    let resp = client
        .get("http://".to_string() + server + "/public-keys")
        .send()
        .await;
    if let Ok(resp) = resp {
        let d = resp.text().await;
        if let Ok(d) = d {
            vodozemac::Ed25519PublicKey::from_base64(&d)?
                .verify(&body, &Ed25519Signature::from_base64(&signature)?)?;

            return Ok(());
        }
    }
    Err(Error::NoSignature)
}

async fn create_id(
    headers: HeaderMap,
    State((db, client)): State<(SqlitePool, reqwest::Client)>,
    body: Bytes,
) -> Result<(StatusCode, Json<Identifier>), Error> {
    let model: CreateId = serde_json::from_slice(&body)?;

    if model.public_keys.is_empty() {
        return Err(Error::PublicKeysEmpty);
    }

    if std::env::var("DEPOT_DEV").is_err() && model.server.starts_with("localhost") {
        return Err(Error::LocalhostInvalid);
    }

    let sig = headers.get("X-Depot-Signature");

    if sig.is_none() {
        return Err(Error::NoSignature);
    }

    verify_server_key(
        &client,
        body,
        sig.unwrap().to_str().unwrap().to_string(),
        &model.server,
    )
    .await?;

    let id = nanoid!();

    let mut tx = db.begin().await?;
    sqlx::query!("INSERT INTO identifiers (id) VALUES ($1);", id)
        .execute(&mut *tx)
        .await?;
    for key in model.public_keys.iter() {
        Ed25519PublicKey::from_base64(key)?;
        sqlx::query!(
            "INSERT INTO public_keys (id, key) VALUES ($1, $2);",
            id,
            key
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(Identifier {
            id,
            public_keys: model.public_keys,
            handle: None,
            server: model.server,
        }),
    ))
}

async fn get_public_keys(
    State((db, _)): State<(SqlitePool, reqwest::Client)>,
    Path(id): Path<String>,
) -> Result<Json<Vec<String>>, Error> {
    let keys_recs = sqlx::query!("SELECT key FROM public_keys WHERE id = $1", id)
        .fetch_all(&db)
        .await?;
    let keys: Vec<String> = keys_recs.into_iter().map(|r| r.key).collect();

    Ok(Json(keys))
}

async fn get_identifier(
    State((db, _)): State<(SqlitePool, reqwest::Client)>,
    Path(id): Path<String>,
) -> Result<Json<Identifier>, Error> {
    let id_rec = sqlx::query!("SELECT * FROM identifiers WHERE id = $1;", id)
        .fetch_one(&db)
        .await?;
    let keys_recs = sqlx::query!("SELECT key FROM public_keys WHERE id = $1", id)
        .fetch_all(&db)
        .await?;
    let public_keys: Vec<String> = keys_recs.into_iter().map(|r| r.key).collect();

    Ok(Json(Identifier {
        id: id_rec.id,
        public_keys,
        handle: id_rec.handle,
        server: id_rec.server,
    }))
}

async fn push_public_keys(
    headers: HeaderMap,
    State((db, _)): State<(SqlitePool, reqwest::Client)>,
    Path(id): Path<String>,
    body: Bytes,
) -> Result<(StatusCode, impl IntoResponse), Error> {
    // NOTE: really annoying but we have to do JSON validation ourself here
    // because `body` and `Json(model)` can't coexist in Axum land.
    let model: PushPublicKeys = serde_json::from_slice(&body)?;

    let dt = chrono::DateTime::from_timestamp_millis(model.ts).unwrap_or_default();

    if (dt.timestamp_millis() - model.ts).lt(&0) | (dt.timestamp_millis() - model.ts).gt(&60_500) {
        return Err(Error::InvalidTimestamp);
    }

    let sig = headers.get("X-Depot-Signature");

    if let Some(sig) = sig {
        let public_keys = sqlx::query!("SELECT key FROM public_keys WHERE id = $1", id)
            .fetch_all(&db)
            .await?;
        let keys: Vec<String> = public_keys.into_iter().map(|r| r.key).collect();

        let mut verified = false;

        for key in keys {
            let k = Ed25519PublicKey::from_base64(&key)?;

            if let Ok(()) = k.verify(
                &body,
                &Ed25519Signature::from_base64(sig.to_str().unwrap())?,
            ) {
                verified = true;
                break;
            }
        }

        if !verified {
            return Err(Error::BadSignature);
        }

        let mut tx = db.begin().await?;

        for key in model.public_keys.iter() {
            Ed25519PublicKey::from_base64(key)?;
            sqlx::query!(
                "INSERT INTO public_keys (id, key) VALUES ($1, $2);",
                id,
                key
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
    } else {
        return Err(Error::NoSignature);
    }

    Ok((StatusCode::NO_CONTENT, "".to_string()))
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().unwrap();

    let db_connection_str = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:1234@localhost".to_string());

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::PATCH])
        .allow_headers(Any)
        .allow_origin(Any);

    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&db_connection_str)
        .await
        .expect("Couldn't connect to SQLite database");

    let app = axum::Router::new()
        .route("/", post(create_id))
        .route("/:id", get(get_identifier))
        .route("/:id/keys", post(push_public_keys).get(get_public_keys))
        .layer(cors)
        .with_state((db, reqwest::Client::new()));

    // keep consistency with port numbers
    let listener = TcpListener::bind("0.0.0.0:24650").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
