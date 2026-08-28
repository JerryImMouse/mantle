use crate::{
    config::runtime::RuntimeConfig,
    db::{
        self, MantleDb,
        identities::{Identity, IdentityProvider},
    },
    error::InternalError,
    integrations::discord::{
        client::DiscordClient,
        models::{DiscordUserModel, PartialGuildModel},
    },
    web::error::WebError,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;
use uuid::Uuid;

const DISCORD_AUTH_URL_BASE: &str = "https://discord.com/oauth2/authorize?scope=identify+guilds+guilds.members.read&response_type=code";

const CURRENT_USER_CACHE_KEY: &str = "discord.current_user";
const GUILDS_CACHE_KEY: &str = "discord.guilds";

#[derive(Error, Debug)]
pub enum DiscordOAuthError {
    #[error("identity({provider:?}) with id: `{provider_user_id}` was not found")]
    IdentityNotFound {
        provider: IdentityProvider,
        provider_user_id: String,
    },

    #[error("identity doesn't have oauth tokens: {0}")]
    IdentityTokensNotFound(Uuid),
    #[error("discord identity for this user is not found")]
    DiscordIdentityNotFound,

    #[error(transparent)]
    Internal(#[from] InternalError),
}

impl From<DiscordOAuthError> for WebError {
    fn from(value: DiscordOAuthError) -> Self {
        match value {
            DiscordOAuthError::Internal(e) => Self::from(e),
            _ => Self::user(404, Some(value.to_string())),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthState {
    provider: IdentityProvider,
    provider_user_id: String,
    exp: usize,
}

impl OAuthState {
    pub fn sign(&self, secret: &str) -> jsonwebtoken::errors::Result<String> {
        let key = EncodingKey::from_secret(secret.as_bytes());
        jsonwebtoken::encode(&Header::default(), &self, &key)
    }

    pub fn from_token(token: &str, secret: &str) -> jsonwebtoken::errors::Result<OAuthState> {
        let key = DecodingKey::from_secret(secret.as_bytes());
        Ok(jsonwebtoken::decode(token, &key, &Validation::default())?.claims)
    }
}

#[derive(Debug, Clone)]
pub struct DiscordOAuthService {
    client: DiscordClient,
    db: MantleDb,
    config: std::sync::Arc<RuntimeConfig>,
}

impl DiscordOAuthService {
    pub fn new(db: MantleDb, config: std::sync::Arc<RuntimeConfig>) -> Self {
        Self {
            db,
            config,
            client: DiscordClient::new(),
        }
    }

    pub async fn get_guilds(
        &self,
        provider: IdentityProvider,
        provider_user_id: &str,
    ) -> Result<Vec<PartialGuildModel>, DiscordOAuthError> {
        let identity = db::identities::find_linked_identity(
            &self.db,
            provider,
            provider_user_id,
            IdentityProvider::Discord,
        )
        .await
        .map_err(InternalError::from)?
        .ok_or(DiscordOAuthError::DiscordIdentityNotFound)?;
        let guilds = match db::identity_cache::get::<Vec<PartialGuildModel>>(
            &self.db,
            identity.id,
            GUILDS_CACHE_KEY,
        )
        .await
        .map_err(InternalError::from)?
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
                ).await?;
                
                guilds
            }
        };

        Ok(guilds)
    }

    pub async fn get_current_user(
        &self,
        provider: IdentityProvider,
        provider_user_id: &str,
    ) -> Result<DiscordUserModel, DiscordOAuthError> {
        let identity = db::identities::find_linked_identity(
            &self.db,
            provider,
            provider_user_id,
            IdentityProvider::Discord,
        )
        .await
        .map_err(InternalError::from)?
        .ok_or(DiscordOAuthError::DiscordIdentityNotFound)?;

        let user = match db::identity_cache::get::<DiscordUserModel>(
            &self.db,
            identity.id,
            CURRENT_USER_CACHE_KEY,
        )
        .await
        .map_err(InternalError::from)?
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

    pub async fn process_callback(
        &self,
        code: &str,
        state: &str,
    ) -> Result<Identity, DiscordOAuthError> {
        let state = OAuthState::from_token(state, &self.config.discord.state_secret)
            .map_err(InternalError::from)?;

        let now = Utc::now();
        let tokens = self
            .client
            .exchange_code(
                code,
                &self.config.discord.redirect_uri,
                &self.config.discord.client_id,
                &self.config.discord.client_secret,
            )
            .await
            .map_err(InternalError::from)?;

        let discord_user = self
            .client
            .get_current_user(&tokens.access_token)
            .await
            .map_err(InternalError::from)?;

        let identity = match db::identities::find_by_provider_id(
            &self.db,
            state.provider,
            &state.provider_user_id,
        )
        .await
        .map_err(InternalError::from)?
        {
            Some(identity) => identity,
            None => {
                return Err(DiscordOAuthError::IdentityNotFound {
                    provider_user_id: state.provider_user_id,
                    provider: state.provider,
                });
            }
        };

        let discord_identity = db::identities::create(
            &self.db,
            db::identities::CreateIdentityReq {
                mantle_user_id: identity.mantle_user_id,
                provider: IdentityProvider::Discord,
                provider_user_id: discord_user.id.as_str(),
            },
        )
        .await
        .map_err(InternalError::from)?;

        // save oauth tokens
        db::oauth_tokens::create(
            &self.db,
            discord_identity.id,
            tokens.access_token,
            tokens.refresh_token,
            now + Duration::seconds(tokens.expires_in as i64),
        )
        .await
        .map_err(InternalError::from)?;

        // set cache record for current user
        db::identity_cache::set(
            &self.db,
            discord_identity.id,
            CURRENT_USER_CACHE_KEY,
            &discord_user,
            now + Duration::hours(1), // TTL is 1 hour, but may be reduced later
        )
        .await?;

        Ok(discord_identity)
    }

    // https://docs.discord.com/developers/topics/oauth2#authorization-code-grant
    pub fn generate_link(
        &self,
        provider: IdentityProvider,
        provider_user_id: String,
    ) -> Result<String, DiscordOAuthError> {
        let state = OAuthState {
            provider,
            provider_user_id,
            exp: (Utc::now() + Duration::minutes(10)).timestamp() as usize,
        };

        let state = state
            .sign(&self.config.discord.state_secret)
            .map_err(InternalError::from)?;
        let client_id = &self.config.discord.client_id;

        let redirect_uri =
            Url::parse(&self.config.discord.redirect_uri).map_err(InternalError::from)?;

        let mut url = Url::parse(DISCORD_AUTH_URL_BASE).map_err(InternalError::from)?;
        url.query_pairs_mut()
            .append_pair("client_id", &client_id)
            .append_pair("response_type", "code")
            .append_pair("scope", "identify guilds guilds.members.read")
            .append_pair("state", &state)
            .append_pair("redirect_uri", redirect_uri.as_str());

        Ok(url.to_string())
    }

    async fn access_token(&self, identity: &Identity) -> Result<String, DiscordOAuthError> {
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

            let new_tokens = self
                .client
                .refresh_token(
                    &tokens.refresh_token,
                    &self.config.discord.client_id,
                    &self.config.discord.client_secret,
                )
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

        Err(DiscordOAuthError::IdentityTokensNotFound(identity.id))
    }
}
