use crate::{
    config::SharedConfig,
    db::{
        self, MantleDb,
        identities::{Identity, IdentityProvider},
    },
    error::InternalError,
    integrations::discord::DiscordClient,
    web::WebError,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;
use uuid::Uuid;

const DISCORD_AUTH_URL_BASE: &str = "https://discord.com/oauth2/authorize?scope=identify+guilds+guilds.members.read&response_type=code";

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
    config: SharedConfig,
}

impl DiscordOAuthService {
    pub fn new(db: MantleDb, client: DiscordClient, config: SharedConfig) -> Self {
        Self { db, config, client }
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
            super::discord::CURRENT_USER_CACHE_KEY,
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
            .append_pair("client_id", client_id)
            .append_pair("response_type", "code")
            .append_pair("scope", "identify guilds guilds.members.read")
            .append_pair("state", &state)
            .append_pair("redirect_uri", redirect_uri.as_str());

        Ok(url.to_string())
    }
}
