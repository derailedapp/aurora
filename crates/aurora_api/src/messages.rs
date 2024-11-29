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

use aurora_db::{FromId, guild::Guild, message::Message};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{patch, post},
};
use serde::Deserialize;
use serde_valid::Validate;

use crate::{
    channels::get_channel,
    error::{ErrorMessage, OVTError},
    flags::GuildPermissions,
    guilds::verify_permissions,
    pubsub::{Event, publish},
    state::OVTState,
    token::get_user,
};

#[derive(Deserialize, Validate)]
pub struct GetGuildChannelMessagesFilter {
    #[validate(minimum = 5)]
    #[validate(maximum = 256)]
    limit: i64,
    #[serde(default)]
    before: Option<String>,
    #[serde(default)]
    after: Option<String>,
}

impl Default for GetGuildChannelMessagesFilter {
    fn default() -> Self {
        Self {
            limit: 30,
            before: None,
            after: None,
        }
    }
}

pub async fn get_guild_channel_messages(
    headers: HeaderMap,
    maybe_filters: Option<Query<GetGuildChannelMessagesFilter>>,
    Path((guild_id, channel_id)): Path<(String, String)>,
    State(state): State<OVTState>,
) -> Result<Json<Vec<Message>>, (StatusCode, Json<ErrorMessage>)> {
    let Query(filters) = maybe_filters.unwrap_or_default();

    let user = get_user(&headers, &state.key, &state.pg).await?;
    let guild = Guild::from_id(&state.pg, guild_id)
        .await
        .map_err(|_| OVTError::GuildNotFound.to_resp())?;
    let channel = get_channel(&state.pg, &channel_id, &guild.id).await?;
    verify_permissions(
        &state.pg,
        &user,
        &guild,
        GuildPermissions::VIEW_MESSAGE_HISTORY,
    )
    .await?;

    let messages = sqlx::query_as!(
        Message,
        "SELECT * FROM messages WHERE channel_id = $1 AND id > $2 AND id < $3 LIMIT $4;",
        &channel.id,
        filters.after,
        filters.before,
        filters.limit
    )
    .fetch_all(&state.pg)
    .await
    .map_err(|_| OVTError::InternalServerError.to_resp())?;

    Ok(Json(messages))
}

#[derive(Deserialize, Validate)]
pub struct CreateMessage {
    #[validate(min_length = 1)]
    #[validate(max_length = 2048)]
    content: String,
}

pub async fn create_guild_channel_message(
    headers: HeaderMap,
    Path((guild_id, channel_id)): Path<(String, String)>,
    State(mut state): State<OVTState>,
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

    publish(&mut state.redis, &guild.id, Event::MessageCreate(&message)).await?;

    Ok(Json(message))
}

pub async fn modify_guild_channel_message(
    headers: HeaderMap,
    Path((guild_id, channel_id, message_id)): Path<(String, String, String)>,
    State(mut state): State<OVTState>,
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
        publish(&mut state.redis, &guild.id, Event::MessageModified(&msg)).await?;

        Ok(Json(msg))
    } else {
        Err(OVTError::MessageNotFound.to_resp())
    }
}

pub async fn delete_guild_channel_message(
    headers: HeaderMap,
    Path((guild_id, channel_id, message_id)): Path<(String, String, String)>,
    State(mut state): State<OVTState>,
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

    publish(&mut state.redis, &guild.id, Event::MessageDelete(&message)).await?;

    Ok((StatusCode::NO_CONTENT, "".to_string()))
}

pub fn router() -> Router<OVTState> {
    Router::<OVTState>::new()
        .route(
            "/guilds/:guild_id/channels/:channel_id/messages",
            post(create_guild_channel_message).get(get_guild_channel_messages),
        )
        .route(
            "/guilds/:guild_id/channels/:channel_id/messages/:message_id",
            patch(modify_guild_channel_message).delete(delete_guild_channel_message),
        )
}
