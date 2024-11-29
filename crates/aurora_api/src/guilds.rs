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

use aurora_db::{guild::Guild, guild_member::GuildMember, user::User, FromId};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{patch, post},
    Json, Router,
};
use serde::Deserialize;
use sqlx::PgPool;

use crate::{
    error::{ErrorMessage, OVTError},
    flags::GuildPermissions,
    state::OVTState,
    token::get_user,
};

async fn verify_permissions(
    db: &PgPool,
    user: &User,
    guild: &Guild,
    required_permissions: GuildPermissions,
) -> Result<(), (StatusCode, Json<ErrorMessage>)> {
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

#[derive(Deserialize)]
pub struct CreateGuild {
    name: String,
}

pub async fn create_guild(
    headers: HeaderMap,
    State(state): State<OVTState>,
    Json(model): Json<CreateGuild>,
) -> Result<Json<Guild>, (StatusCode, Json<ErrorMessage>)> {
    let user = get_user(&headers, &state.key, &state.pg).await?;

    let mut tx = state
        .pg
        .begin()
        .await
        .map_err(|_| OVTError::InternalServerError.to_resp())?;

    let guild = sqlx::query_as!(
        Guild,
        "INSERT INTO guilds (id, owner_id, name, permissions) VALUES ($1, $2, $3, 0) RETURNING *;",
        uuid7::uuid7().to_string(),
        &user.id,
        model.name
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

    tx.commit()
        .await
        .map_err(|_| OVTError::InternalServerError.to_resp())?;

    Ok(Json(guild))
}

#[derive(Deserialize)]
pub struct ModifyGuild {
    #[serde(default)]
    name: Option<String>,
}

// TODO: foreign servers
pub async fn modify_guild(
    headers: HeaderMap,
    Path(guild_id): Path<String>,
    State(state): State<OVTState>,
    Json(model): Json<ModifyGuild>,
) -> Result<Json<Guild>, (StatusCode, Json<ErrorMessage>)> {
    let user = get_user(&headers, &state.key, &state.pg).await?;
    let guild = Guild::from_id(&state.pg, guild_id)
        .await
        .map_err(|_| OVTError::GuildNotFound.to_resp())?;
    verify_permissions(&state.pg, &user, &guild, GuildPermissions::MODIFY_GUILD).await?;

    let modified_guild = sqlx::query_as!(
        Guild,
        "UPDATE guilds SET name = $2 WHERE id = $1 RETURNING *;",
        &guild.id,
        model.name
    )
    .fetch_one(&state.pg)
    .await
    .map_err(|_| OVTError::InternalServerError.to_resp())?;

    Ok(Json(modified_guild))
}

pub fn router() -> Router<OVTState> {
    Router::<OVTState>::new()
        .route("/guilds", post(create_guild))
        .route("/guilds/:guild_id", patch(modify_guild))
}
