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

use raildepot::{CreateId, DeleteIdentifier, Identifier};
use sqlx::types::chrono::Utc;
use vodozemac::{Ed25519PublicKey, Ed25519SecretKey};

use crate::{error::Error, routes::tracks::TrackExistence, state::State};

pub async fn create_identifier(state: &State, public_key: Ed25519PublicKey) -> String {
    let public_keys = vec![public_key.to_base64()];

    let body = serde_json::to_string(&CreateId {
        public_keys,
        server: state.server.clone(),
        ts: Utc::now().timestamp_millis(),
    })
    .unwrap();

    let depot = std::env::var("DEPOT_URL").expect("Depot URL not present");

    let req = state
        .client
        .post(depot)
        .body(body.clone())
        .header("Content-Type", "application/json")
        .header(
            "X-Depot-Signature",
            state.key.sign(body.as_bytes()).to_base64(),
        )
        .send()
        .await
        .unwrap();
    let json = req.json::<Identifier>().await.unwrap();
    json.id
}

pub async fn delete_identifier(
    state: &State,
    identifier: &str,
    key: Ed25519SecretKey,
) -> Result<(), Error> {
    let body = serde_json::to_string(&DeleteIdentifier {
        ts: Utc::now().timestamp_millis(),
    })
    .unwrap();

    let depot = std::env::var("DEPOT_URL").expect("Depot URL not present");

    state
        .client
        .delete(format!("{}/{}", depot, identifier))
        .body(body.clone())
        .header("Content-Type", "application/json")
        .header("X-Depot-Signature", key.sign(body.as_bytes()).to_base64())
        .send()
        .await?;
    Ok(())
}

pub async fn get_identifier(
    client: &reqwest::Client,
    identifier: &str,
) -> Result<Identifier, Error> {
    let depot = std::env::var("DEPOT_URL").expect("Depot URL not present");

    Ok(client
        .get(format!("{}/{}", depot, identifier))
        .send()
        .await?
        .json::<Identifier>()
        .await?)
}

pub async fn verify_track_existence(
    client: &reqwest::Client,
    identifier: &Identifier,
    track_id: &str,
) -> Result<(), Error> {
    let req = client
        .get(format!(
            "http://{}/users/{}/tracks/{}/exists",
            identifier.server, identifier.id, track_id
        ))
        .send()
        .await?
        .json::<TrackExistence>()
        .await?;

    if req.exists {
        Ok(())
    } else {
        Err(Error::InvalidParentId)
    }
}
