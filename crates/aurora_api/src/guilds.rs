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

use aurora_db::{
    DBError, FromId, guild::Guild, guild_invite::GuildInvite, guild_member::GuildMember, user::User,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{delete, patch, post},
};
use serde::{Deserialize, Serialize};
use serde_valid::Validate;
use sqlx::PgPool;

use crate::{
    error::{ErrorMessage, OVTError},
    flags::GuildPermissions,
    pubsub::{Event, publish},
    state::OVTState,
    token::get_user,
};

pub async fn verify_permissions(
    db: &PgPool,
    user: &User,
    guild: &Guild,
    required_permissions: GuildPermissions,
) -> Result<(), (StatusCode, Json<ErrorMessage>)> {
    if user.id == guild.owner_id {
        return Ok(());
    }

    GuildMember::from_id(db, (&user.id, &guild.id))
        .await
        .map_err(|_| OVTError::GuildNotFound.to_resp())?;

    // TODO: roles
    // NOTE: should be safe since guild permissions
    // are only left alone for foreign servers.
    let everyone_perms = GuildPermissions::from_bits(guild.permissions.unwrap() as u64).unwrap();

    if everyone_perms.contains(required_permissions) {
        Ok(())
    } else {
        Err(OVTError::InvalidPermissions.to_resp())
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateGuild {
    #[validate(min_length = 1)]
    #[validate(max_length = 32)]
    name: String,
}

pub async fn create_guild(
    headers: HeaderMap,
    State(mut state): State<OVTState>,
    Json(model): Json<CreateGuild>,
) -> Result<Json<Guild>, (StatusCode, Json<ErrorMessage>)> {
    let user = get_user(&headers, &state.key, &state.pg).await?;

    let mut tx = state
        .pg
        .begin()
        .await
        .map_err(|_| OVTError::InternalServerError.to_resp())?;

    let perms = GuildPermissions::all().bits();

    let guild = sqlx::query_as!(
        Guild,
        "INSERT INTO guilds (id, owner_id, name, permissions) VALUES ($1, $2, $3, $4) RETURNING *;",
        uuid7::uuid7().to_string(),
        &user.id,
        model.name,
        perms as i64
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|_| OVTError::InternalServerError.to_resp())?;
    sqlx::query!(
        "INSERT INTO guild_members (guild_id, user_id) VALUES ($1, $2);",
        &guild.id,
        &user.id
    )
    .execute(&mut *tx)
    .await
    .map_err(|_| OVTError::InternalServerError.to_resp())?;

    publish(&mut state.redis, &user.id, Event::GuildCreate(&guild)).await?;

    tx.commit()
        .await
        .map_err(|_| OVTError::InternalServerError.to_resp())?;

    Ok(Json(guild))
}

#[derive(Debug, Deserialize, Validate)]
pub struct ModifyGuild {
    #[serde(default)]
    #[validate(min_length = 1)]
    #[validate(max_length = 32)]
    name: Option<String>,
    #[serde(default)]
    permissions: Option<u64>,
}

// TODO: foreign servers
pub async fn modify_guild(
    headers: HeaderMap,
    Path(guild_id): Path<String>,
    State(mut state): State<OVTState>,
    Json(model): Json<ModifyGuild>,
) -> Result<Json<Guild>, (StatusCode, Json<ErrorMessage>)> {
    let user = get_user(&headers, &state.key, &state.pg).await?;
    let guild = Guild::from_id(&state.pg, guild_id)
        .await
        .map_err(|_| OVTError::GuildNotFound.to_resp())?;
    verify_permissions(&state.pg, &user, &guild, GuildPermissions::MODIFY_GUILD).await?;

    if let Some(perms) = model.permissions {
        if GuildPermissions::from_bits(perms).is_none() {
            return Err(OVTError::InvalidPermissionBitflags.to_resp());
        }
    }

    let modified_guild = sqlx::query_as!(
        Guild,
        "UPDATE guilds SET name = $2 WHERE id = $1 RETURNING *;",
        &guild.id,
        model.name
    )
    .fetch_one(&state.pg)
    .await
    .map_err(|_| OVTError::InternalServerError.to_resp())?;

    publish(
        &mut state.redis,
        &guild.id,
        Event::GuildUpdate(&modified_guild),
    )
    .await?;

    Ok(Json(modified_guild))
}

pub async fn delete_guild(
    headers: HeaderMap,
    Path(guild_id): Path<String>,
    State(mut state): State<OVTState>,
) -> Result<(StatusCode, String), (StatusCode, Json<ErrorMessage>)> {
    let user = get_user(&headers, &state.key, &state.pg).await?;
    let guild = Guild::from_id(&state.pg, guild_id)
        .await
        .map_err(|_| OVTError::GuildNotFound.to_resp())?;

    if user.id != guild.id {
        return Err(OVTError::NotGuildOwner.to_resp());
    }

    sqlx::query!("DELETE FROM guilds WHERE id = $1;", &guild.id,)
        .execute(&state.pg)
        .await
        .map_err(|_| OVTError::InternalServerError.to_resp())?;

    publish(&mut state.redis, &user.id, Event::GuildDelete(&guild.id)).await?;

    Ok((StatusCode::NO_CONTENT, "".to_string()))
}

#[derive(Serialize)]
pub struct ReturnedInvite {
    invite: String,
}

// invites

pub async fn use_invite(
    headers: HeaderMap,
    Path(invite_id): Path<String>,
    State(mut state): State<OVTState>,
) -> Result<Json<Guild>, (StatusCode, Json<ErrorMessage>)> {
    let user = get_user(&headers, &state.key, &state.pg).await?;

    let invite = sqlx::query!("SELECT * FROM guild_invites WHERE id = $1;", invite_id)
        .fetch_optional(&state.pg)
        .await
        .map_err(|_| OVTError::InternalServerError.to_resp())?;

    if let Some(inv) = invite {
        let guild = Guild::from_id(&state.pg, inv.guild_id)
            .await
            .map_err(|_| OVTError::GuildNotFound.to_resp())?;
        let member = GuildMember::from_id(&state.pg, (&user.id, &guild.id)).await;

        if member.is_ok() {
            return Err(OVTError::GuildAlreadyJoined.to_resp());
        }
        if let Err(e) = member {
            match e {
                DBError::RowNotFound => {}
                _ => return Err(OVTError::InternalServerError.to_resp()),
            };
        }

        sqlx::query!(
            "INSERT INTO guild_members (user_id, guild_id) VALUES ($1, $2)",
            &user.id,
            &guild.id
        )
        .execute(&state.pg)
        .await
        .map_err(|_| OVTError::InternalServerError.to_resp())?;

        publish(&mut state.redis, &guild.id, Event::MemberJoin(&user)).await?;

        Ok(Json(guild))
    } else {
        Err(OVTError::InviteNotFound.to_resp())
    }
}

// TODO: pagination / limiting
pub async fn get_guild_invites(
    headers: HeaderMap,
    Path(guild_id): Path<String>,
    State(state): State<OVTState>,
) -> Result<Json<Vec<String>>, (StatusCode, Json<ErrorMessage>)> {
    let user = get_user(&headers, &state.key, &state.pg).await?;
    let guild = Guild::from_id(&state.pg, guild_id)
        .await
        .map_err(|_| OVTError::GuildNotFound.to_resp())?;
    verify_permissions(&state.pg, &user, &guild, GuildPermissions::CREATE_INVITES).await?;

    let invites = sqlx::query_as!(
        GuildInvite,
        "SELECT * FROM guild_invites WHERE guild_id = $1;",
        &guild.id
    )
    .fetch_all(&state.pg)
    .await
    .map_err(|_| OVTError::InternalServerError.to_resp())?;

    let invite_strings: Vec<String> = invites.into_iter().map(|v| v.id).collect();

    Ok(Json(invite_strings))
}

pub async fn create_invite(
    headers: HeaderMap,
    Path(guild_id): Path<String>,
    State(state): State<OVTState>,
) -> Result<Json<ReturnedInvite>, (StatusCode, Json<ErrorMessage>)> {
    let user = get_user(&headers, &state.key, &state.pg).await?;
    let guild = Guild::from_id(&state.pg, guild_id)
        .await
        .map_err(|_| OVTError::GuildNotFound.to_resp())?;
    verify_permissions(&state.pg, &user, &guild, GuildPermissions::CREATE_INVITES).await?;

    let invite = sqlx::query_as!(
        GuildInvite,
        "INSERT INTO guild_invites (id, guild_id) VALUES ($1, $2) RETURNING *;",
        uuid7::uuid7().to_string(),
        &guild.id
    )
    .fetch_one(&state.pg)
    .await
    .map_err(|_| OVTError::InternalServerError.to_resp())?;

    Ok(Json(ReturnedInvite { invite: invite.id }))
}

pub async fn delete_invite(
    headers: HeaderMap,
    Path((guild_id, invite_id)): Path<(String, String)>,
    State(state): State<OVTState>,
) -> Result<(StatusCode, String), (StatusCode, Json<ErrorMessage>)> {
    let user = get_user(&headers, &state.key, &state.pg).await?;
    let guild = Guild::from_id(&state.pg, guild_id)
        .await
        .map_err(|_| OVTError::GuildNotFound.to_resp())?;
    verify_permissions(&state.pg, &user, &guild, GuildPermissions::MANAGE_INVITES).await?;

    sqlx::query!(
        "DELETE FROM guild_invites WHERE id = $1 AND guild_id = $2;",
        invite_id,
        &guild.id
    )
    .execute(&state.pg)
    .await
    .map_err(|_| OVTError::InternalServerError.to_resp())?;

    Ok((StatusCode::NO_CONTENT, "".to_string()))
}

pub async fn leave_guild(
    headers: HeaderMap,
    Path(guild_id): Path<String>,
    State(mut state): State<OVTState>,
) -> Result<(StatusCode, String), (StatusCode, Json<ErrorMessage>)> {
    let user = get_user(&headers, &state.key, &state.pg).await?;
    let guild = Guild::from_id(&state.pg, guild_id)
        .await
        .map_err(|_| OVTError::GuildNotFound.to_resp())?;
    verify_permissions(&state.pg, &user, &guild, GuildPermissions::empty()).await?;

    let mem = sqlx::query_as!(
        GuildMember,
        "DELETE FROM guild_members WHERE user_id = $1 RETURNING *;",
        &user.id
    )
    .fetch_one(&state.pg)
    .await
    .map_err(|_| OVTError::InternalServerError.to_resp())?;
    publish(&mut state.redis, &guild.id, Event::MemberLeave(&mem)).await?;

    Ok((StatusCode::NO_CONTENT, "".to_string()))
}

pub fn router() -> Router<OVTState> {
    Router::<OVTState>::new()
        .route("/guilds", post(create_guild))
        .route(
            "/guilds/:guild_id",
            patch(modify_guild).delete(delete_guild),
        )
        .route(
            "/guilds/:guild_id/invites",
            post(create_invite).get(get_guild_invites),
        )
        .route(
            "/guilds/:guild_id/invites/:invite_id",
            delete(delete_invite),
        )
        .route("/invites/:invite_id", post(use_invite))
        .route("/users/@me/guilds/:guild_id", delete(leave_guild))
}
