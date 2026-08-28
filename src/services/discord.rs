use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::{
    config::SharedConfig,
    db::{
        self, MantleDb,
        identities::{Identity, IdentityProvider},
    },
    error::InternalError,
    integrations::discord::{DiscordClient, DiscordUserModel, PartialGuildModel},
    web::WebError,
};

pub const CURRENT_USER_CACHE_KEY: &str = "discord.current_user";
pub const GUILDS_CACHE_KEY: &str = "discord.guilds";

type Result<T> = std::result::Result<T, DiscordError>;

#[derive(Debug, thiserror::Error)]
pub enum DiscordError {
    #[error("discord identity was not found")]
    DiscordIdentityNotFound,

    #[error("provided identity(`{0}`) has no oauth tokens")]
    IdentityHasNoTokens(Uuid),

    #[error(transparent)]
    Internal(#[from] InternalError),
}

impl From<DiscordError> for WebError {
    fn from(value: DiscordError) -> Self {
        match value {
            DiscordError::Internal(e) => Self::from(e),
            _ => Self::user(404, Some(value.to_string())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiscordService {
    db: MantleDb,
    client: DiscordClient,
    config: SharedConfig,
}

impl DiscordService {
    pub fn new(db: MantleDb, client: DiscordClient, config: SharedConfig) -> Self {
        Self { db, client, config }
    }

    pub async fn get_current_user(
        &self,
        provider: IdentityProvider,
        provider_user_id: &str,
    ) -> Result<DiscordUserModel> {
        let identity = db::identities::find_linked_identity(
            &self.db,
            provider,
            provider_user_id,
            IdentityProvider::Discord,
        )
        .await
        .map_err(InternalError::from)?
        .ok_or(DiscordError::DiscordIdentityNotFound)?;

        let user = match db::identity_cache::get::<DiscordUserModel>(
            &self.db,
            identity.id,
            CURRENT_USER_CACHE_KEY,
        )
        .await?
        {
            Some(user) => user,
            None => {
                tracing::debug!(mantle_user = %identity.mantle_user_id, discord_identity = %identity.id, "cache miss: requsting new get_current_user from discord API");
                // get new user info and update cache
                let token = self.access_token(&identity).await?;
                let user = self
                    .client
                    .get_current_user(&token)
                    .await
                    .map_err(InternalError::from)?;

                // update cache
                db::identity_cache::set(
                    &self.db,
                    identity.id,
                    CURRENT_USER_CACHE_KEY,
                    &user,
                    Utc::now() + Duration::hours(1), // TTL is 1 hour for this
                )
                .await?;
                user
            }
        };

        Ok(user)
    }

    pub async fn get_guilds(
        &self,
        provider: IdentityProvider,
        provider_user_id: &str,
    ) -> Result<Vec<PartialGuildModel>> {
        let identity = db::identities::find_linked_identity(
            &self.db,
            provider,
            provider_user_id,
            IdentityProvider::Discord,
        )
        .await
        .map_err(InternalError::from)?
        .ok_or(DiscordError::DiscordIdentityNotFound)?;
        let guilds = match db::identity_cache::get::<Vec<PartialGuildModel>>(
            &self.db,
            identity.id,
            GUILDS_CACHE_KEY,
        )
        .await?
        {
            Some(guilds) => guilds,
            None => {
                tracing::debug!(mantle_user = %identity.mantle_user_id, discord_identity = %identity.id, "cache miss: requsting new `get_guilds` from discord API");
                let token = self.access_token(&identity).await?;
                let guilds = self
                    .client
                    .get_guilds(&token)
                    .await
                    .map_err(InternalError::from)?;

                db::identity_cache::set(
                    &self.db,
                    identity.id,
                    GUILDS_CACHE_KEY,
                    &guilds,
                    Utc::now() + Duration::hours(1),
                )
                .await?;

                guilds
            }
        };

        Ok(guilds)
    }

    fn client_credentials(&self) -> (&str, &str) {
        (
            &self.config.discord.client_id,
            &self.config.discord.client_secret,
        )
    }

    async fn access_token(&self, identity: &Identity) -> Result<String> {
        // TODO: Current implementation can allow making refresh races
        // where hundreds of requests can try to get access_token and see expired one ->
        // they all will try to update it and may end up being rate limited.
        // This stuff need some per-identity lock or smth
        let tokens = db::oauth_tokens::find_by_id(&self.db, identity.id)
            .await
            .map_err(InternalError::from)?;

        if let Some(tokens) = tokens {
            if tokens.expires_at > Utc::now() + Duration::minutes(1) {
                return Ok(tokens.access_token);
            }

            let credentials = self.client_credentials();
            let new_tokens = self
                .client
                .refresh_token(&tokens.refresh_token, credentials.0, credentials.1)
                .await
                .map_err(InternalError::from)?;

            let now = Utc::now();
            db::oauth_tokens::update(
                &self.db,
                identity.id,
                &new_tokens.access_token,
                &new_tokens.refresh_token,
                now + Duration::seconds(new_tokens.expires_in as i64),
            )
            .await
            .map_err(InternalError::from)?;

            return Ok(new_tokens.access_token);
        }

        Err(DiscordError::IdentityHasNoTokens(identity.id))
    }
}
