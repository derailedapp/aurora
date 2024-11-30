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

use sqlx::PgPool;

pub mod account;
pub mod account_settings;
pub mod actor;
pub mod channel;
pub mod guild;
pub mod guild_invite;
pub mod guild_member;
pub mod message;
pub mod server;
pub mod session;

pub enum DBError {
    RowNotFound,
    DBErr,
}

pub type FromIdResult<T> = Result<T, DBError>;

pub trait FromId<T>: Sized {
    fn from_id(db: &PgPool, id: T) -> impl std::future::Future<Output = FromIdResult<Self>> + Send;
}
