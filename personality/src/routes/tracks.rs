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

use axum::{
    Json,
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use serde_valid::Validate;

use crate::{
    db::{tent::clean_get_user_db, track::Track},
    depot::{get_identifier, verify_track_existence},
    error::Error,
    produce::{BeamMessage, beam_out},
    token::get_user,
};

#[derive(Serialize, Deserialize, Validate)]
pub struct GetUserTrack {
    #[validate(minimum = 1)]
    #[validate(maximum = 2048)]
    limit: i32,
}

pub async fn get_user_tracks(
    Query(params): Query<GetUserTrack>,
    Path(id): Path<String>,
) -> Result<Json<Vec<Track>>, Error> {
    let db = clean_get_user_db(&id).await?;

    Ok(Json(
        sqlx::query_as!(Track, "SELECT * FROM tracks LIMIT $1;", params.limit)
            .fetch_all(&db)
            .await?,
    ))
}

#[derive(Deserialize, Validate)]
pub struct CreateTrack {
    #[validate(max_length = 2048)]
    #[validate(min_length = 1)]
    content: String,
    /// Parent post ID.
    /// For data viability must be under 128 characters in length.
    /// For data variability must be at least 24 characters in length.
    #[validate(max_length = 128)]
    #[validate(min_length = 24)]
    #[serde(default)]
    parent_id: Option<String>,
}

pub async fn create_track(
    headers: HeaderMap,
    State(state): State<crate::state::State>,
    Json(model): Json<CreateTrack>,
) -> Result<Json<Track>, Error> {
    let (_, account, db) = get_user(&headers, &state.jwt_secret).await?;

    // verify parent id
    if let Some(ref parent_id) = model.parent_id {
        let parts: Vec<&str> = parent_id.split('/').collect();

        if parts.len() != 2 {
            return Err(Error::InvalidParentId);
        }

        let uid = *parts.first().unwrap();
        let pid = *parts.get(1).unwrap();

        // TODO: this returns 500 if the identifier is invalid. Return 400 instead.
        let user_id = get_identifier(&state.client, uid).await?;
        verify_track_existence(&state.client, &user_id, pid).await?;
    }

    let track = Track::create(&db, &account, &model.content, model.parent_id).await?;

    beam_out(BeamMessage::TrackCreate(track.clone()), &state).await?;

    Ok(Json(track))
}

pub async fn delete_track(
    headers: HeaderMap,
    State(state): State<crate::state::State>,
    Path(id): Path<String>,
) -> Result<String, Error> {
    let (_actor, _account, db) = get_user(&headers, &state.jwt_secret).await?;

    let track = sqlx::query_as!(Track, "DELETE FROM tracks WHERE id = $1 RETURNING *;", id)
        .fetch_optional(&db)
        .await?;

    if let Some(track) = track {
        beam_out(BeamMessage::TrackDelete(track), &state).await?;
    }

    Ok("".to_string())
}

#[derive(Serialize, Deserialize)]
pub struct TrackExistence {
    pub exists: bool,
}

pub fn router() -> axum::Router<crate::state::State> {
    axum::Router::new()
        .route("/users/:id/tracks", get(get_user_tracks))
        .route("/tracks", post(create_track))
        .route("/tracks/:id", delete(delete_track))
}
