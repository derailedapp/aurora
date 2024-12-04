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

use aurora_api::token::Claims;
use jsonwebtoken::DecodingKey;
use rustler::{types::tuple::make_tuple, Encoder, Env, Error, Term};

mod atoms {
    rustler::atoms! {
        ok,
        error,
        invalid_token
    }
}

fn load(_: Env, _: Term) -> bool {
    dotenvy::dotenv().unwrap();
    true
}

#[rustler::nif]
fn get_token_session_id(env: Env, token: String) -> Result<Term, Error> {
    let claims = Claims::from_token(
        &token,
        &DecodingKey::from_secret(std::env::var("JWT_SECRET_KEY").unwrap().as_bytes()),
    );

    if let Ok(c) = claims {
        Ok(make_tuple(env, &[atoms::ok().to_term(env), c.sub.encode(env)]))
    } else {
        Err(Error::Term(Box::new(atoms::invalid_token())))
    }
}

#[rustler::nif]
fn get_chronological_id() -> String {
    uuid7::uuid7().to_string()
}

rustler::init!("Elixir.Derailed.DB.Rs", load = load);
