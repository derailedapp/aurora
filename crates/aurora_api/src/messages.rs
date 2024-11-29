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

use aurora_db::{guild::Guild, message::Message, FromId};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{patch, post},
    Json, Router,
};
use serde::Deserialize;

use crate::{
    channels::get_channel,
    error::{ErrorMessage, OVTError},
    flags::GuildPermissions,
    guilds::verify_permissions,
    state::OVTState,
    token::get_user,
};

#[derive(Deserialize)]
pub struct CreateMessage {
    content: String,
}

pub async fn create_guild_channel_message(
    headers: HeaderMap,
    Path((guild_id, channel_id)): Path<(String, String)>,
    State(state): State<OVTState>,
    Json(model): Json<CreateMessage>,
) -> Result<Json<Message>, (StatusCode, Json<ErrorMessage>)> {
    let user = get_user(&headers, &state.key, &state.pg).await?;
    let guild = Guild::from_id(&state.pg, guild_id)
        .await
        .map_err(|_| OVTError::GuildNotFound.to_resp())?;
    let channel = get_channel(&state.pg, &channel_id, &guild.id).await?;
    verify_permissions(&state.pg, &user, &guild, GuildPermissions::SEND_MESSAGE).await?;

    let message = sqlx::query_as!(
        Message,
        "INSERT INTO messages (id, author_id, channel_id, content) VALUES ($1, $2, $3, $4) RETURNING *;",
        uuid7::uuid7().to_string(),
        &user.id,
        &channel.id,
        model.content
    ).fetch_one(&state.pg).await.map_err(|_| OVTError::InternalServerError.to_resp())?;

    Ok(Json(message))
}

pub async fn modify_guild_channel_message(
    headers: HeaderMap,
    Path((guild_id, channel_id, message_id)): Path<(String, String, String)>,
    State(state): State<OVTState>,
    Json(model): Json<CreateMessage>,
) -> Result<Json<Message>, (StatusCode, Json<ErrorMessage>)> {
    let user = get_user(&headers, &state.key, &state.pg).await?;
    let guild = Guild::from_id(&state.pg, guild_id)
        .await
        .map_err(|_| OVTError::GuildNotFound.to_resp())?;
    get_channel(&state.pg, &channel_id, &guild.id).await?;

    let message = sqlx::query_as!(
        Message,
        "UPDATE messages SET content = $2 WHERE id = $1 AND author_id = $3 RETURNING *;",
        message_id,
        model.content,
        &user.id
    )
    .fetch_optional(&state.pg)
    .await
    .map_err(|_| OVTError::InternalServerError.to_resp())?;

    if let Some(msg) = message {
        Ok(Json(msg))
    } else {
        Err(OVTError::MessageNotFound.to_resp())
    }
}

pub async fn delete_guild_channel_message(
    headers: HeaderMap,
    Path((guild_id, channel_id, message_id)): Path<(String, String, String)>,
    State(state): State<OVTState>,
) -> Result<(StatusCode, String), (StatusCode, Json<ErrorMessage>)> {
    let user = get_user(&headers, &state.key, &state.pg).await?;
    let guild = Guild::from_id(&state.pg, guild_id)
        .await
        .map_err(|_| OVTError::GuildNotFound.to_resp())?;
    get_channel(&state.pg, &channel_id, &guild.id).await?;
    let message = Message::from_id(&state.pg, message_id)
        .await
        .map_err(|_| OVTError::MessageNotFound.to_resp())?;

    if message.channel_id != channel_id {
        return Err(OVTError::MessageNotFound.to_resp());
    }

    // TODO: refactor when roles happen
    let everyone_perms = GuildPermissions::from_bits(guild.permissions.unwrap() as u64).unwrap();

    let is_message_author = if let Some(author_id) = &message.author_id {
        author_id.eq(&user.id)
    } else {
        false
    };

    if !is_message_author && !everyone_perms.contains(GuildPermissions::MANAGE_MESSAGES) {
        return Err(OVTError::InvalidPermissions.to_resp());
    }

    sqlx::query!("DELETE FROM messages WHERE id = $1;", &message.id)
        .execute(&state.pg)
        .await
        .map_err(|_| OVTError::InternalServerError.to_resp())?;

    Ok((StatusCode::NO_CONTENT, "".to_string()))
}

pub fn router() -> Router<OVTState> {
    Router::<OVTState>::new()
        .route(
            "/guilds/:guild_id/channels/:channel_id/messages",
            post(create_guild_channel_message),
        )
        .route(
            "/guilds/:guild_id/channels/:channel_id/messages/:message_id",
            patch(modify_guild_channel_message).delete(delete_guild_channel_message),
        )
}
