use aurora_api::token::Claims;
use jsonwebtoken::DecodingKey;
use rustler::{Env, Error, Term};

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
fn get_token_session_id(token: String) -> Result<String, Error> {
    let claims = Claims::from_token(
        &token,
        &DecodingKey::from_secret(std::env::var("JWT_SECRET").unwrap().as_bytes()),
    );

    if let Ok(c) = claims {
        Ok(c.sub)
    } else {
        Err(Error::Term(Box::new(atoms::invalid_token())))
    }
}

#[rustler::nif]
fn get_chronological_id() -> String {
    uuid7::uuid7().to_string()
}

rustler::init!("Elixir.Derailed.DB.Rs", load = load);
