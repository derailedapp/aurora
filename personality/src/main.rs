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

#![feature(duration_constructors)]

use axum::routing::get;
use reqwest::Method;
use state::State;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use vodozemac::Ed25519Keypair;

mod db;
mod depot;
mod error;
mod routes;
mod state;
mod token;

pub async fn get_public_keys(axum::extract::State(state): axum::extract::State<State>) -> String {
    state.key.public_key().to_base64()
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

    let app = axum::Router::new()
        .route("/public-keys", get(get_public_keys))
        .merge(routes::users::router())
        .layer(cors)
        .with_state(State {
            server: std::env::var("PRSN_SERVER").expect("Server domain is needed to be set with `PRSN_SERVER` for identification. If running in dev mode, use `localhost` and make sure the variable `DEPT_DEV` is present on your Rail Depot instance."),
            client: reqwest::Client::new(),
            key: Ed25519Keypair::new(),
            jwt_secret: std::env::var("PRSN_SECRET_KEY").expect("Secret key is needed to be set with `PRSN_SECRET_KEY` for secure JWT authentication.")
        });

    // keep consistency with port numbers
    let listener = TcpListener::bind("0.0.0.0:24640").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
