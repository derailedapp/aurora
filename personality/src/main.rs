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

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::PATCH])
        .allow_headers(Any)
        .allow_origin(Any);

    let app = axum::Router::new()
        .route("/public-keys", get(get_public_keys))
        .merge(routes::users::router())
        .merge(routes::tracks::router())
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
