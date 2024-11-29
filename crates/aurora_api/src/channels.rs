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

use aurora_db::{channel::Channel, guild::Guild, FromId};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{patch, post},
    Json, Router,
};
use serde::Deserialize;
use serde_valid::Validate;
use sqlx::PgPool;

use crate::{
    error::{ErrorMessage, OVTError},
    flags::GuildPermissions,
    guilds::verify_permissions,
    state::OVTState,
    token::get_user,
};

pub async fn get_channel(
    db: &PgPool,
    channel_id: &str,
    guild_id: &str,
) -> Result<Channel, (StatusCode, Json<ErrorMessage>)> {
    let maybe_channel = sqlx::query_as!(
        Channel,
        "SELECT * FROM channels WHERE id = $1 AND guild_id = $2;",
        channel_id,
        guild_id
    )
    .fetch_optional(db)
    .await
    .map_err(|_| OVTError::InternalServerError.to_resp())?;

    if let Some(channel) = maybe_channel {
        Ok(channel)
    } else {
        Err(OVTError::ChannelNotFound.to_resp())
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateGuildChannel {
    #[validate(pattern = r"^[a-b0-9_-]+$")]
    #[validate(min_length = 1)]
    #[validate(max_length = 32)]
    name: String,
}

// TODO: foreign servers
pub async fn create_guild_channel(
    headers: HeaderMap,
    Path(guild_id): Path<String>,
    State(state): State<OVTState>,
    Json(model): Json<CreateGuildChannel>,
) -> Result<Json<Channel>, (StatusCode, Json<ErrorMessage>)> {
    let user = get_user(&headers, &state.key, &state.pg).await?;
    let guild = Guild::from_id(&state.pg, guild_id)
        .await
        .map_err(|_| OVTError::GuildNotFound.to_resp())?;
    verify_permissions(&state.pg, &user, &guild, GuildPermissions::MANAGE_CHANNELS).await?;

    let channel = sqlx::query_as!(
        Channel,
        "INSERT INTO channels (id, name, guild_id, position) VALUES ($1, $2, $3, 0) RETURNING *;",
        uuid7::uuid7().to_string(),
        model.name.trim(),
        &guild.id
    )
    .fetch_one(&state.pg)
    .await
    .map_err(|_| OVTError::InternalServerError.to_resp())?;

    Ok(Json(channel))
}

#[derive(Debug, Deserialize, Validate)]
pub struct ModifyGuildChannel {
    #[validate(pattern = r"^[a-b0-9_-]+$")]
    #[validate(min_length = 1)]
    #[validate(max_length = 32)]
    name: String,
    #[validate(minimum = 0)]
    #[validate(maximum = 200)]
    position: u32,
}

// TODO: foreign servers
pub async fn modify_guild_channel(
    headers: HeaderMap,
    Path((guild_id, channel_id)): Path<(String, String)>,
    State(state): State<OVTState>,
    Json(model): Json<ModifyGuildChannel>,
) -> Result<Json<Channel>, (StatusCode, Json<ErrorMessage>)> {
    let user = get_user(&headers, &state.key, &state.pg).await?;
    let guild = Guild::from_id(&state.pg, guild_id)
        .await
        .map_err(|_| OVTError::GuildNotFound.to_resp())?;
    verify_permissions(&state.pg, &user, &guild, GuildPermissions::MANAGE_CHANNELS).await?;

    let channel = sqlx::query_as!(
        Channel,
        "UPDATE channels SET name = $1, position = $2 WHERE id = $3 AND guild_id = $4 RETURNING *;",
        &model.name.trim(),
        model.position as i32,
        &channel_id,
        &guild.id
    )
    .fetch_optional(&state.pg)
    .await
    .map_err(|_| OVTError::InternalServerError.to_resp())?;

    if let Some(modified_channel) = channel {
        Ok(Json(modified_channel))
    } else {
        Err(OVTError::ChannelNotFound.to_resp())
    }
}

pub async fn delete_guild_channel(
    headers: HeaderMap,
    Path((guild_id, channel_id)): Path<(String, String)>,
    State(state): State<OVTState>,
) -> Result<(StatusCode, String), (StatusCode, Json<ErrorMessage>)> {
    let user = get_user(&headers, &state.key, &state.pg).await?;
    let guild = Guild::from_id(&state.pg, guild_id)
        .await
        .map_err(|_| OVTError::GuildNotFound.to_resp())?;
    get_channel(&state.pg, &channel_id, &guild.id).await?;
    verify_permissions(&state.pg, &user, &guild, GuildPermissions::MANAGE_CHANNELS).await?;

    sqlx::query!(
        "DELETE FROM channels WHERE id = $1 AND guild_id = $2;",
        &channel_id,
        &guild.id
    )
    .execute(&state.pg)
    .await
    .map_err(|_| OVTError::InternalServerError.to_resp())?;

    Ok((StatusCode::NO_CONTENT, "".to_string()))
}

pub fn router() -> Router<OVTState> {
    Router::<OVTState>::new()
        .route("/guilds/:guild_id/channels", post(create_guild_channel))
        .route(
            "/guilds/:guild_id/channels/:channel_id",
            patch(modify_guild_channel).delete(delete_guild_channel),
        )
}
