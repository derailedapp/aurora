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
    actor::Actor, channel::Channel, guild::Guild, guild_member::GuildMember, message::Message,
};
use axum::{extract::Json, http::StatusCode};
use redis::{aio::MultiplexedConnection, AsyncCommands};
use serde::{Deserialize, Serialize};

use crate::error::{ErrorMessage, OVTError};

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "t", content = "d")]
pub enum Event {
    GuildCreate(Guild),
    GuildUpdate(Guild),
    GuildDelete(String),
    MemberJoin(Actor),
    MemberLeave(GuildMember),
    MessageCreate(Message),
    MessageModified(Message),
    MessageDelete(Message),
    ChannelCreate(Channel),
    ChannelModified(Channel),
    ChannelDelete(String),
}

pub async fn publish(
    conn: &mut MultiplexedConnection,
    channel: &str,
    event: Event,
) -> Result<(), (StatusCode, Json<ErrorMessage>)> {
    let message =
        serde_json::to_string(&event).map_err(|_| OVTError::InternalServerError.to_resp())?;

    let _: () = conn
        .publish(channel, &message)
        .await
        .map_err(|_| OVTError::InternalServerError.to_resp())?;

    Ok(())
}
