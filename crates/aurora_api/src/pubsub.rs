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

use aurora_db::{channel::Channel, guild::Guild, message::Message, user::User};
use axum::{extract::Json, http::StatusCode};
use redis::{aio::MultiplexedConnection, AsyncCommands};
use serde::Serialize;

use crate::error::{ErrorMessage, OVTError};

#[derive(Serialize)]
#[serde(tag = "t")]
pub enum Event<'a> {
    GuildCreate(&'a Guild),
    GuildUpdate(&'a Guild),
    GuildDelete(&'a str),
    MemberJoin(&'a User),
    MessageCreate(&'a Message),
    MessageModified(&'a Message),
    MessageDelete(&'a Message),
    ChannelCreate(&'a Channel),
    ChannelModified(&'a Channel),
    ChannelDelete(&'a str),
}

pub async fn publish<'a>(
    conn: &mut MultiplexedConnection,
    channel: &str,
    event: Event<'a>,
) -> Result<(), (StatusCode, Json<ErrorMessage>)> {
    let message =
        serde_json::to_string(&event).map_err(|_| OVTError::InternalServerError.to_resp())?;

    let _: () = conn.publish(channel, &message)
        .await
        .map_err(|_| OVTError::InternalServerError.to_resp())?;

    Ok(())
}
