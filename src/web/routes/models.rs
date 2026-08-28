use crate::db::{
    identities::{Identity, IdentityProvider},
    metadata::UserMetadata,
};
use chrono::{DateTime, Utc};
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

#[derive(Debug, Serialize, Deserialize)]
pub struct UserMetadataDto {
    value: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<UserMetadata> for UserMetadataDto {
    fn from(value: UserMetadata) -> Self {
        Self {
            value: value.value,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
