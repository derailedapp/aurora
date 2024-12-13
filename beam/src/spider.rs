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

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, types::chrono::Utc};

use crate::algo;

/// Functionality of Beam which spiders throughout different Persona instances
/// collecting as much data as it can from actors.
///
/// TODOs:
/// - Handle 429s

#[derive(thiserror::Error, Debug)]
pub enum SpiderError {
    #[error("Crawl is unsupported or disabled on this Persona")]
    CrawlUnsupported,

    #[error("Crawling gave back an unknown response status code")]
    StatusCodeUnknown,

    #[error("Invalid Data Structure sent by crawl")]
    DataStructureUnknown(#[from] serde_json::Error),

    #[error("Failed to process returned body")]
    ReqwestError(#[from] reqwest::Error),

    #[error("An ID given is not registered on Rail Depot")]
    IdNotRegistered,

    #[error("Invalid public key")]
    InvalidPublicKey(#[from] vodozemac::KeyError),

    #[error("Invalid signature given")]
    InvalidSignatureVodo(#[from] vodozemac::SignatureError),

    #[error("Invalid signature given")]
    InvalidSignature,

    #[error("Database error")]
    DBError(#[from] sqlx::Error),
}

#[derive(Deserialize, Serialize)]
pub struct CrawlUserResult {
    pub actor: persona::actor::Actor,
    pub tracks: Vec<persona::track::Track>,
    pub following: Vec<String>,
}

pub async fn req_crawl_user(
    c: &reqwest::Client,
    server: &str,
    user_id: &str,
) -> Result<(CrawlUserResult, String, Option<String>), SpiderError> {
    let resp = c
        .get("http://".to_string() + server + "/users/" + user_id + "/crawl")
        .send()
        .await
        .map_err(|err| {
            if let Some(status) = err.status() {
                if status == StatusCode::NOT_FOUND {
                    return SpiderError::CrawlUnsupported;
                }
            }
            SpiderError::StatusCodeUnknown
        })?;

    let headers = resp.headers().clone();
    let text = resp.text().await?;

    Ok((
        serde_json::from_str::<CrawlUserResult>(&text)?,
        text,
        headers
            .get("X-Actor-Signature")
            .map(|v| v.to_str().unwrap().to_string()),
    ))
}

pub async fn get_user_server(c: &reqwest::Client, user_id: &str) -> Result<String, SpiderError> {
    let resp = c
        .get(
            "http://".to_string()
                + env::var("RAIL_DEPOT")
                    .expect("Rail Depot not present")
                    .as_str()
                + "/"
                + user_id,
        )
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(SpiderError::CrawlUnsupported);
    }

    let json = resp.json::<raildepot::Identifier>().await?;

    Ok(json.server)
}

pub async fn verify_user_ids(
    c: &reqwest::Client,
    user_ids: Vec<String>,
) -> Result<(), SpiderError> {
    let resp = c
        .get(
            "http://".to_string()
                + env::var("RAIL_DEPOT")
                    .expect("Rail Depot not present")
                    .as_str()
                + "/exists",
        )
        .json(&raildepot::VerifyIdentifiers {
            identifiers: user_ids,
        })
        .send()
        .await?;

    if resp.status() != StatusCode::NO_CONTENT {
        Err(SpiderError::IdNotRegistered)
    } else {
        Ok(())
    }
}

pub async fn crawl_user(
    db: PgPool,
    c: Option<reqwest::Client>,
    server: Option<&str>,
    user_id: &str,
) -> Result<(), SpiderError> {
    let client = c.unwrap_or_default();
    let server = if let Some(s) = server {
        s.to_string()
    } else {
        get_user_server(&client, user_id).await?
    };
    let identifier = persona::depot::get_identifier(&client, user_id)
        .await
        .map_err(|_| SpiderError::IdNotRegistered)?;

    let (crawl_data, txt, sig) = req_crawl_user(&client, &server, user_id).await?;

    let s = sig.unwrap_or("".to_string());

    for pubkey in identifier.public_keys.iter() {
        let key = vodozemac::Ed25519PublicKey::from_base64(pubkey)?;
        let sig = vodozemac::Ed25519Signature::from_base64(&s)?;

        if key.verify(txt.as_bytes(), &sig).is_err() {
            return Err(SpiderError::InvalidSignature);
        }
    }

    let user_ids = crawl_data.following;

    verify_user_ids(&client, user_ids.clone()).await?;

    let mut tx = db.begin().await?;
    let capture_ts = Utc::now().timestamp_millis();

    let actor = crawl_data.actor;
    sqlx::query!("INSERT INTO actors (id, display_name, avatar, banner, bio, status) VALUES ($1, $2, $3, $4, $5, $6);", actor.id, actor.display_name, actor.avatar, actor.banner, actor.bio, actor.status).execute(&mut *tx).await?;
    for user_id in user_ids {
        sqlx::query!(
            "INSERT INTO followings (user_id, other_user_id) VALUES ($1, $2);",
            actor.id,
            user_id
        )
        .execute(&mut *tx)
        .await?;
    }
    for track in crawl_data.tracks {
        let canonical_track_id = actor.id.clone() + "/" + track.id.as_str();

        // TODO: verify parent id?
        sqlx::query!("INSERT INTO tracks (id, author_id, content, ts, original_ts, parent_id) VALUES ($1, $2, $3, $4, $5, $6)", canonical_track_id, actor.id, track.content, capture_ts, track.created_at, track.parent_id).execute(&mut *tx).await?;

        let topics = algo::topics_from_content(&track.content);

        for topic in topics {
            sqlx::query!(
                "INSERT INTO track_topics (id, topic) VALUES ($1, $2);",
                canonical_track_id,
                topic
            )
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;

    Ok(())
}
