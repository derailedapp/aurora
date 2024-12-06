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
use chrono::Utc;
use error::Error;
use nanoid::nanoid;
use raildepot::{CreateId, DeleteIdentifier, Identifier, PushPublicKeys};
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

    let dt = Utc::now();

    if (dt.timestamp_millis() - model.ts).lt(&0) | (dt.timestamp_millis() - model.ts).gt(&60_500) {
        return Err(Error::InvalidTimestamp);
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
    sqlx::query!(
        "INSERT INTO identifiers (id, tombstone) VALUES ($1, $2);",
        id,
        false
    )
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
            tombstone: false,
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
) -> Result<(StatusCode, Json<Identifier>), Error> {
    let id_rec = sqlx::query!("SELECT * FROM identifiers WHERE id = $1;", id)
        .fetch_one(&db)
        .await?;
    let keys_recs = sqlx::query!("SELECT key FROM public_keys WHERE id = $1", id)
        .fetch_all(&db)
        .await?;
    let public_keys: Vec<String> = keys_recs.into_iter().map(|r| r.key).collect();

    let status = if id_rec.tombstone {
        StatusCode::GONE
    } else {
        StatusCode::OK
    };

    Ok((
        status,
        Json(Identifier {
            id: id_rec.id,
            public_keys,
            handle: id_rec.handle,
            server: id_rec.server,
            tombstone: id_rec.tombstone,
        }),
    ))
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

    let dt = Utc::now();

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

async fn kill_identifier(
    headers: HeaderMap,
    State((db, _)): State<(SqlitePool, reqwest::Client)>,
    Path(id): Path<String>,
    body: Bytes,
) -> Result<(StatusCode, impl IntoResponse), Error> {
    // NOTE: really annoying but we have to do JSON validation ourself here
    // because `body` and `Json(model)` can't coexist in Axum land.
    let model: DeleteIdentifier = serde_json::from_slice(&body)?;

    let dt = Utc::now();

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

        sqlx::query!(
            "UPDATE identifiers SET handle = $1, server = $2, tombstone = $3 WHERE id = $4;",
            Option::<String>::None,
            Option::<String>::None,
            true,
            id
        )
        .execute(&mut *tx)
        .await?;
        sqlx::query!("DELETE FROM public_keys WHERE id = $1", id)
            .execute(&mut *tx)
            .await?;

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
        .route("/:id", get(get_identifier).delete(kill_identifier))
        .route("/:id/keys", post(push_public_keys).get(get_public_keys))
        .layer(cors)
        .with_state((db, reqwest::Client::new()));

    // keep consistency with port numbers
    let listener = TcpListener::bind("0.0.0.0:24650").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
