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

use bitflags::bitflags;

bitflags! {
    pub struct GuildPermissions: u64 {
        const MODIFY_GUILD = 1;
        const MANAGE_CHANNELS = 1 << 1;
        const VIEW_MESSAGE_HISTORY = 1 << 2;
        const SEND_MESSAGE = 1 << 3;
        const MANAGE_MESSAGES = 1 << 4;
        const VIEW_GUILD_INVITE_LIST = 1 << 5;
        const CREATE_INVITES = 1 << 6;
        const MANAGE_INVITES = 1 << 7;
    }
}
