use serde::{Deserialize, Serialize};
use serde_valid::Validate;

/// The full amount of data represented by the identifier.
#[derive(Serialize, Deserialize)]
pub struct Identifier {
    /// A unique ID used to identify a user, lodge, or guild
    pub id: String,
    /// A set of unique (to this context) IDs used for verifying actions by this identifier
    pub public_keys: Vec<String>,
    /// A domain handle which has a TXT record `_depot` which contains `id`
    pub handle: Option<String>,
    /// The Personality or Geniality server which owns this identifier
    pub server: String,
    /// Whether this identifier is dead or not
    pub tombstone: bool,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct CreateId {
    /// list of ed25519 public keys.
    pub public_keys: Vec<String>,
    #[validate(pattern = r"^(?:localhost|(?:[a-z0-9-]+\.)+[a-z]{2,})(?::\d{1,5})?$")]
    pub server: String,
    pub ts: i64,
}

#[derive(Serialize, Deserialize)]
pub struct PushPublicKeys {
    /// list of ed25519 public keys.
    pub public_keys: Vec<String>,
    /// A timestamp with maximum jitter of one minute
    pub ts: i64,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteIdentifier {
    /// A timestamp with maximum jitter of one minute
    pub ts: i64,
}
