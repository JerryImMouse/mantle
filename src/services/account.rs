use uuid::Uuid;

use crate::db::identities::{CreateIdentityReq, Identity, IdentityProvider};
use crate::db::metadata::UserMetadata;
use crate::db::{self, MantleDb};
use crate::error::InternalError;
use crate::web::WebError;

type Result<T> = std::result::Result<T, AccountError>;

#[derive(Debug, thiserror::Error)]
pub enum AccountError {
    // TODO: Remove duplication here
    #[error("identity({provider:?}) with id: `{provider_user_id}` was not found")]
    IdentityNotFound {
        provider: IdentityProvider,
        provider_user_id: String,
    },

    #[error(transparent)]
    Internal(#[from] InternalError),
}

impl From<AccountError> for WebError {
    fn from(value: AccountError) -> Self {
        match value {
            AccountError::Internal(e) => WebError::from(e),
            e => WebError::user(400, e.to_string()),
        }
    }
}

#[derive(Debug)]
pub struct UserCheckResult {
    pub mantle_user_id: Uuid,
    identities: Vec<Identity>,
}

impl UserCheckResult {
    pub fn has(&self, identity: IdentityProvider) -> bool {
        self.identities.iter().any(|i| i.provider == identity)
    }
}

#[derive(Debug, Clone)]
pub struct AccountService {
    db: MantleDb,
}

impl AccountService {
    pub fn new(db: MantleDb) -> Self {
        Self { db }
    }

    pub async fn check(
        &self,
        provider: IdentityProvider,
        provider_user_id: &str,
    ) -> Result<UserCheckResult> {
        let identity =
            match db::identities::find_by_provider_id(&self.db, provider, provider_user_id)
                .await
                .map_err(InternalError::from)?
            {
                Some(identity) => identity,
                None => {
                    let user = db::users::create(&self.db)
                        .await
                        .map_err(InternalError::from)?;
                    db::identities::create(
                        &self.db,
                        CreateIdentityReq {
                            mantle_user_id: user.id,
                            provider,
                            provider_user_id,
                        },
                    )
                    .await
                    .map_err(InternalError::from)?
                }
            };

        let identities = db::identities::all_for_user(&self.db, identity.mantle_user_id)
            .await
            .map_err(InternalError::from)?;

        Ok(UserCheckResult {
            mantle_user_id: identity.mantle_user_id,
            identities,
        })
    }

    pub async fn find_linked_identities(
        &self,
        provider: IdentityProvider,
        provider_user_id: &str,
    ) -> Result<Vec<Identity>> {
        db::identities::find_linked_identities(&self.db, provider, provider_user_id)
            .await
            .map_err(InternalError::from)
            .map_err(AccountError::from)
    }

    pub async fn metadata_set(
        &self,
        provider: IdentityProvider,
        provider_user_id: &str,
        key: &str,
        value: &serde_json::Value,
        private: Option<bool>,
    ) -> Result<UserMetadata> {
        let identity = db::identities::find_by_provider_id(&self.db, provider, provider_user_id)
            .await
            .map_err(InternalError::from)?
            .ok_or(AccountError::IdentityNotFound {
                provider,
                provider_user_id: provider_user_id.into(),
            })?;

        let metadata = db::metadata::set(&self.db, identity.mantle_user_id, key, value, private)
            .await
            .map_err(InternalError::from)?;

        Ok(metadata)
    }

    pub async fn metadata_get(
        &self,
        provider: IdentityProvider,
        provider_user_id: &str,
        key: &str,
    ) -> Result<Option<UserMetadata>> {
        let identity = db::identities::find_by_provider_id(&self.db, provider, provider_user_id)
            .await
            .map_err(InternalError::from)?
            .ok_or(AccountError::IdentityNotFound {
                provider,
                provider_user_id: provider_user_id.into(),
            })?;

        db::metadata::get(&self.db, identity.mantle_user_id, key)
            .await
            .map_err(InternalError::from)
            .map_err(AccountError::from)
    }

    pub async fn metadata_delete(
        &self,
        provider: IdentityProvider,
        provider_user_id: &str,
        key: &str,
    ) -> Result<Option<UserMetadata>> {
        let identity = db::identities::find_by_provider_id(&self.db, provider, provider_user_id)
            .await
            .map_err(InternalError::from)?
            .ok_or(AccountError::IdentityNotFound {
                provider,
                provider_user_id: provider_user_id.into(),
            })?;

        db::metadata::delete(&self.db, identity.mantle_user_id, key)
            .await
            .map_err(InternalError::from)
            .map_err(AccountError::from)
    }
}
