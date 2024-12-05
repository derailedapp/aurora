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

use raildepot::{CreateId, Identifier};
use vodozemac::Ed25519PublicKey;

use crate::state::State;

pub async fn create_identifier(state: &State, public_key: Ed25519PublicKey) -> String {
    let mut public_keys = Vec::new();
    public_keys.push(public_key.to_base64());
    // TODO: handle error
    let req = state
        .client
        .post("")
        .json(&CreateId {
            public_keys,
            server: state.server.clone(),
        })
        .send()
        .await
        .unwrap();
    let json = req.json::<Identifier>().await.unwrap();
    json.id
}
