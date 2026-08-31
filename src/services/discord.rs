use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::{
    config::SharedConfig,
    db::{
        self, MantleDb,
        identities::{Identity, IdentityProvider},
    },
    error::InternalError,
    integrations::discord::{
        DiscordClient, DiscordUserModel, GuildMemberModel, PartialGuildModel, Snowflake,
    },
    utils::tokens::RefreshLock,
    web::WebError,
};

pub const CACHE_TTL_HOURS: i64 = 1;

pub const CURRENT_USER_CACHE_KEY: &str = "discord.current_user";
pub const GUILDS_CACHE_KEY: &str = "discord.guilds";
pub const GUILD_MEMBER_CACHE_KEY: &str = "discord.guild_member";

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
            _ => Self::user(404, value.to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiscordService {
    db: MantleDb,
    client: DiscordClient,
    config: SharedConfig,
    refresh_lock: RefreshLock,
}

impl DiscordService {
    pub fn new(db: MantleDb, client: DiscordClient, config: SharedConfig) -> Self {
        Self {
            db,
            client,
            config,
            refresh_lock: Default::default(),
        }
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
                    Utc::now() + Duration::hours(CACHE_TTL_HOURS), // TTL is 1 hour for this
                )
                .await?;
                user
            }
        };

        Ok(user)
    }

    pub async fn get_guild_member(
        &self,
        provider: IdentityProvider,
        provider_user_id: &str,
        guild_id: Snowflake,
    ) -> Result<GuildMemberModel> {
        let cache_key = format!("{GUILD_MEMBER_CACHE_KEY}:{guild_id}");
        let identity = db::identities::find_linked_identity(
            &self.db,
            provider,
            provider_user_id,
            IdentityProvider::Discord,
        )
        .await
        .map_err(InternalError::from)?
        .ok_or(DiscordError::DiscordIdentityNotFound)?;

        let guild_member =
            match db::identity_cache::get::<GuildMemberModel>(&self.db, identity.id, &cache_key)
                .await?
            {
                Some(guild_member) => guild_member,
                None => {
                    tracing::debug!(
                        mantle_user = %identity.mantle_user_id,
                        discord_identity = %identity.id,
                        %guild_id,
                        "cache miss: requsting new `get_guild_member` from discord API"
                    );

                    let token = self.access_token(&identity).await?;
                    let guild_member = self
                        .client
                        .get_guild_member(&token, guild_id)
                        .await
                        .map_err(InternalError::from)?;

                    db::identity_cache::set(
                        &self.db,
                        identity.id,
                        &cache_key,
                        &guild_member,
                        Utc::now() + Duration::hours(CACHE_TTL_HOURS),
                    )
                    .await?;

                    guild_member
                }
            };

        Ok(guild_member)
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
                    Utc::now() + Duration::hours(CACHE_TTL_HOURS),
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
        loop {
            let tokens = db::oauth_tokens::find_by_id(&self.db, identity.id)
                .await
                .map_err(InternalError::from)?
                .ok_or(DiscordError::IdentityHasNoTokens(identity.id))?;

            if tokens.expires_at > Utc::now() + Duration::minutes(1) {
                return Ok(tokens.access_token);
            }

            // try acquire a lock before refresh
            let guard = match self.refresh_lock.try_lock(identity) {
                Ok(guard) => guard,
                Err(notify) => {
                    notify.notified().await;
                    continue;
                }
            };

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

            // we will explicitly drop it here so rustc won't do its magic
            // (probably it won't do it anyway because of Drop impl, but still)
            drop(guard);

            return Ok(new_tokens.access_token);
        }
    }
}
