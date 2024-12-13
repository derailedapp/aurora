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
use pulsar::{Authentication, Pulsar, TokioExecutor, authentication::oauth2::OAuth2Authentication};
use reqwest::Method;
use state::State;
use std::env;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use vodozemac::Ed25519Keypair;
use ws::handle_relay;

mod db;
mod depot;
mod error;
mod produce;
mod routes;
mod state;
mod token;
mod ws;

pub async fn get_public_keys(axum::extract::State(state): axum::extract::State<State>) -> String {
    state.key.public_key().to_base64()
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().unwrap();

    // Setup Pulsar
    let addr = env::var("PULSAR_URL")
        .ok()
        .unwrap_or_else(|| "pulsar://localhost:6650".to_string());

    let mut builder = Pulsar::builder(addr, TokioExecutor);

    if let Ok(token) = env::var("PULSAR_TOKEN") {
        let authentication = Authentication {
            name: "token".to_string(),
            data: token.into_bytes(),
        };

        builder = builder.with_auth(authentication);
    } else if let Ok(oauth2_cfg) = env::var("PULSAR_OAUTH2") {
        builder = builder.with_auth_provider(OAuth2Authentication::client_credentials(
            serde_json::from_str(oauth2_cfg.as_str())
                .unwrap_or_else(|_| panic!("invalid oauth2 config [{}]", oauth2_cfg.as_str())),
        ));
    }

    let pulsar: Pulsar<_> = builder.build().await.unwrap();

    // CORS setup
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::PATCH])
        .allow_headers(Any)
        .allow_origin(Any);

    let app = axum::Router::new()
        .route("/public-keys", get(get_public_keys))
        .route("/relay", axum::routing::any(handle_relay))
        .merge(routes::users::router())
        .merge(routes::tracks::router())
        .layer(cors)
        .with_state(State {
            server: std::env::var("PRSN_SERVER").expect("Server domain is needed to be set with `PRSN_SERVER` for identification. If running in dev mode, use `localhost` and make sure the variable `DEPT_DEV` is present on your Rail Depot instance."),
            client: reqwest::Client::new(),
            key: Ed25519Keypair::new(),
            jwt_secret: std::env::var("PRSN_SECRET_KEY").expect("Secret key is needed to be set with `PRSN_SECRET_KEY` for secure JWT authentication."),
            plsr: pulsar
        });

    // keep consistency with port numbers
    let listener = TcpListener::bind("0.0.0.0:24640").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
