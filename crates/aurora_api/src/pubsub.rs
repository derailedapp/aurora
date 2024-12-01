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
use serde::Serialize;

use crate::error::ErrorMessage;

#[derive(Serialize, Clone)]
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

pub async fn publish_user(
    _user_id: &str,
    _event: Event,
) -> Result<(), (StatusCode, Json<ErrorMessage>)> {
    Ok(())
}

pub async fn publish_guild(
    _guild_id: &str,
    _event: Event,
) -> Result<(), (StatusCode, Json<ErrorMessage>)> {
    Ok(())
}
