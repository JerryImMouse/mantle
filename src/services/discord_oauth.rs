use crate::{
    config::SharedConfig,
    db::{
        self, MantleDb,
        identities::{Identity, IdentityProvider},
        oauth_tokens,
    },
    error::InternalError,
    integrations::discord::DiscordClient,
    services::CACHE_TTL_HOURS,
    web::WebError,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;
use uuid::Uuid;

const DISCORD_AUTH_URL_BASE: &str = "https://discord.com/oauth2/authorize";

#[derive(Error, Debug)]
pub enum DiscordOAuthError {
    #[error("identity({provider:?}) with id: `{provider_user_id}` was not found")]
    IdentityNotFound {
        provider: IdentityProvider,
        provider_user_id: String,
    },

    #[error("discord identity for this user already exists")]
    IdentityDuplicate,

    #[error("identity doesn't have oauth tokens: {0}")]
    IdentityTokensNotFound(Uuid),

    #[error("discord identity for this user is not found")]
    DiscordIdentityNotFound,

    #[error("provided state has been expired, try request a new link")]
    ExpiredState,

    #[error(transparent)]
    Internal(#[from] InternalError),
}

impl From<DiscordOAuthError> for WebError {
    fn from(value: DiscordOAuthError) -> Self {
        match value {
            DiscordOAuthError::Internal(e) => Self::from(e),
            DiscordOAuthError::IdentityDuplicate => Self::user(409, Some(value.to_string())),
            DiscordOAuthError::ExpiredState => Self::user(400, Some(value.to_string())),
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
    config: SharedConfig,
}

impl DiscordOAuthService {
    pub fn new(db: MantleDb, client: DiscordClient, config: SharedConfig) -> Self {
        Self { db, config, client }
    }

    #[tracing::instrument(skip(self))]
    pub async fn process_callback(
        &self,
        code: &str,
        state: &str,
    ) -> Result<Identity, DiscordOAuthError> {
        let state = OAuthState::from_token(state, &self.config.discord.state_secret).map_err(
            |e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    DiscordOAuthError::ExpiredState
                }
                _ => InternalError::from(e).into(),
            },
        )?;

        // lookup for duplicate Discord identity
        let existing_discord = db::identities::find_linked_identity(
            &self.db,
            state.provider,
            &state.provider_user_id,
            IdentityProvider::Discord,
        )
        .await
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

        // we don't allow identity rewrites, only token updates
        if let Some(existing_discord) = existing_discord {
            if existing_discord.provider_user_id != discord_user.id.as_str() {
                return Err(DiscordOAuthError::IdentityDuplicate);
            } else {
                tracing::warn!(identity_id = %existing_discord.id, "discord tokens were refreshed");

                // we allow oauth tokens rewrites on the same discord user
                oauth_tokens::update(
                    &self.db,
                    existing_discord.id,
                    &tokens.access_token,
                    &tokens.refresh_token,
                    now + Duration::seconds(tokens.expires_in as i64),
                )
                .await
                .map_err(InternalError::from)?;

                db::identity_cache::set(
                    &self.db,
                    existing_discord.id,
                    super::discord::CURRENT_USER_CACHE_KEY,
                    &discord_user,
                    now + Duration::hours(CACHE_TTL_HOURS), // TTL is 1 hour, but may be reduced later
                )
                .await?;

                return Ok(existing_discord);
            }
        }

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
            super::discord::CURRENT_USER_CACHE_KEY,
            &discord_user,
            now + Duration::hours(CACHE_TTL_HOURS), // TTL is 1 hour, but may be reduced later
        )
        .await?;

        Ok(discord_identity)
    }

    #[tracing::instrument(skip(self))]
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
            .append_pair("client_id", client_id)
            .append_pair("response_type", "code")
            .append_pair("scope", "identify guilds guilds.members.read")
            .append_pair("state", &state)
            .append_pair("redirect_uri", redirect_uri.as_str());

        Ok(url.to_string())
    }
}
