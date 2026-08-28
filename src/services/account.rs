use uuid::Uuid;

use crate::db::identities::{CreateIdentityReq, Identity, IdentityProvider};
use crate::db::{self, MantleDb};
use crate::error::{InternalError, Result};

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
        let identity = match db::identities::find_by_provider_id(
            &self.db,
            provider,
            provider_user_id,
        )
        .await?
        {
            Some(identity) => identity,
            None => {
                let user = db::users::create(&self.db).await?;
                db::identities::create(
                    &self.db,
                    CreateIdentityReq {
                        mantle_user_id: user.id,
                        provider,
                        provider_user_id,
                    },
                )
                .await?
            }
        };

        let identities = db::identities::all_for_user(&self.db, identity.mantle_user_id).await?;

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
    }
}
