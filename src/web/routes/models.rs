use crate::db::identities::{Identity, IdentityProvider};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct UserRequestQuery {
    pub provider: IdentityProvider,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdentityDto {
    provider: IdentityProvider,
    provider_user_id: String,
}

impl From<Identity> for IdentityDto {
    fn from(value: Identity) -> Self {
        IdentityDto {
            provider: value.provider,
            provider_user_id: value.provider_user_id,
        }
    }
}
