use std::time::Duration;

use axum::{
    Json,
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, Method, StatusCode},
    response::IntoResponse,
    routing::post,
};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use vodozemac::{Ed25519PublicKey, Ed25519Signature, KeyError};

/// The full amount of data represented by the identifier.
#[derive(Serialize)]
struct Identifier {
    /// A unique ID used to identify a user, lodge, or guild
    id: String,
    /// A set of unique (to this context) IDs used for verifying actions by this identifier
    public_keys: Vec<String>,
    /// A domain handle which has a TXT record `_depot` which contains `id`
    handle: Option<String>,
}

#[derive(Deserialize)]
struct CreateId {
    /// list of ed25519 public keys.
    public_keys: Vec<String>,
}

async fn create_id(
    State(db): State<SqlitePool>,
    Json(model): Json<CreateId>,
) -> Result<(StatusCode, Json<Identifier>), (StatusCode, String)> {
    if model.public_keys.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Must include public keys".to_string(),
        ));
    }

    let id = nanoid!();

    let mut tx = db.begin().await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
        )
    })?;
    sqlx::query!("INSERT INTO identifiers (id) VALUES ($1);", id)
        .execute(&mut *tx)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
            )
        })?;
    for key in model.public_keys.iter() {
        Ed25519PublicKey::from_base64(key).map_err(|err| match err {
            KeyError::Base64Error(_) => (StatusCode::BAD_REQUEST, "Invalid public key".to_string()),
            _ => (
                StatusCode::BAD_REQUEST,
                "Invalid public key or internal server error".to_string(),
            ),
        })?;
        sqlx::query!(
            "INSERT INTO public_keys (id, key) VALUES ($1, $2);",
            id,
            key
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
            )
        })?;
    }

    tx.commit().await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
        )
    })?;

    Ok((
        StatusCode::CREATED,
        Json(Identifier {
            id,
            public_keys: model.public_keys,
            handle: None,
        }),
    ))
}

async fn get_public_keys(
    State(db): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<Json<Vec<String>>, (StatusCode, String)> {
    let keys_recs = sqlx::query!("SELECT key FROM public_keys WHERE id = $1", id)
        .fetch_all(&db)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
            )
        })?;
    let keys: Vec<String> = keys_recs.into_iter().map(|r| r.key).collect();

    Ok(Json(keys))
}

#[derive(Deserialize)]
struct PushPublicKeys {
    /// list of ed25519 public keys.
    public_keys: Vec<String>,
    /// A timestamp with maximum jitter of one minute
    ts: i64,
}

async fn push_public_keys(
    headers: HeaderMap,
    State(db): State<SqlitePool>,
    Path(id): Path<String>,
    body: Bytes,
) -> Result<(StatusCode, impl IntoResponse), (StatusCode, String)> {
    // NOTE: really annoying but we have to do JSON validation ourself here
    // because `body` and `Json(model)` can't coexist in Axum land.
    let model: PushPublicKeys = serde_json::from_slice(&body)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid JSON object".to_string()))?;

    let dt = chrono::DateTime::from_timestamp_millis(model.ts).unwrap_or_default();

    if (dt.timestamp_millis() - model.ts).lt(&0) | (dt.timestamp_millis() - model.ts).gt(&60_500) {
        return Err((StatusCode::BAD_REQUEST, "Invalid timestamp".to_string()));
    }

    let sig = headers.get("X-Depot-Signature");

    if let Some(sig) = sig {
        let public_keys = sqlx::query!("SELECT key FROM public_keys WHERE id = $1", id)
            .fetch_all(&db)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error".to_string(),
                )
            })?;
        let keys: Vec<String> = public_keys.into_iter().map(|r| r.key).collect();

        let mut verified = false;

        for key in keys {
            let k = Ed25519PublicKey::from_base64(&key).unwrap();

            if let Ok(()) = k.verify(
                &body,
                &Ed25519Signature::from_base64(sig.to_str().map_err(|_| {
                    (
                        StatusCode::UNAUTHORIZED,
                        "Invalid header contents".to_string(),
                    )
                })?)
                .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid signature".to_string()))?,
            ) {
                verified = true;
                break;
            }
        }

        if !verified {
            return Err((
                StatusCode::UNAUTHORIZED,
                "No public keys match signature given".to_string(),
            ));
        }

        let mut tx = db.begin().await.map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
            )
        })?;

        for key in model.public_keys.iter() {
            Ed25519PublicKey::from_base64(key).map_err(|err| match err {
                KeyError::Base64Error(_) => {
                    (StatusCode::BAD_REQUEST, "Invalid public key".to_string())
                }
                _ => (
                    StatusCode::BAD_REQUEST,
                    "Invalid public key or internal server error".to_string(),
                ),
            })?;
            sqlx::query!(
                "INSERT INTO public_keys (id, key) VALUES ($1, $2);",
                id,
                key
            )
            .execute(&mut *tx)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error".to_string(),
                )
            })?;
        }

        tx.commit().await.map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
            )
        })?;
    } else {
        return Err((StatusCode::UNAUTHORIZED, "No signature".to_string()));
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
        .route("/:id/keys", post(push_public_keys).get(get_public_keys))
        .layer(cors)
        .with_state(db);

    // keep consistency with port numbers
    let listener = TcpListener::bind("0.0.0.0:24650").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
