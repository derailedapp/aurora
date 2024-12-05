#[derive(Clone)]
pub struct State {
    pub client: reqwest::Client,
    pub key: vodozemac::Ed25519Keypair,
    pub server: String,
    pub jwt_secret: String,
}
