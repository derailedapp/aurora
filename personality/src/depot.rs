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

use raildepot::{CreateId, DeleteIdentifier, Identifier};
use sqlx::types::chrono::Utc;
use vodozemac::{Ed25519PublicKey, Ed25519SecretKey};

use crate::{error::Error, state::State};

pub async fn create_identifier(state: &State, public_key: Ed25519PublicKey) -> String {
    let mut public_keys = Vec::new();
    public_keys.push(public_key.to_base64());

    let body = serde_json::to_string(&CreateId {
        public_keys,
        server: state.server.clone(),
        ts: Utc::now().timestamp_millis()
    }).unwrap();

    let depot = std::env::var("DEPOT_URL").expect("Depot URL not present");

    let req = state
        .client
        .post(depot + "/")
        .body(body.clone())
        .header("Content-Type", "application/json")
        .header("X-Depot-Signature", state.key.sign(body.as_bytes()).to_base64())
        .send()
        .await
        .unwrap();
    let json = req.json::<Identifier>().await.unwrap();
    json.id
}

pub async fn delete_identifier(state: &State, identifier: &str, key: Ed25519SecretKey) -> Result<(), Error> {
    let body = serde_json::to_string(&DeleteIdentifier {
        ts: Utc::now().timestamp_millis()
    }).unwrap();

    let depot = std::env::var("DEPOT_URL").expect("Depot URL not present");

    state
        .client
        .delete(depot + "/" + identifier)
        .body(body.clone())
        .header("Content-Type", "application/json")
        .header("X-Depot-Signature", key.sign(body.as_bytes()).to_base64())
        .send()
        .await?;
    Ok(())
}
