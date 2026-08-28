use reqwest::StatusCode;
use serde::de::DeserializeOwned;

use crate::integrations::discord::{
    error::DiscordHttpError,
    models::{AccessTokenResponse, DiscordUserModel, ExchangeCodeRequest, PartialGuildModel, RefreshTokenRequest},
};

const TOKEN_URL: &str = "https://discord.com/api/oauth2/token";
const API_URL: &str = "https://discord.com/api/v10";

#[derive(Debug, Clone)]
pub struct DiscordClient {
    http: reqwest::Client,
}

impl DiscordClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("Mantle v0.1.0")
            .build()
            .expect("failed to build reqwest client");
        Self { http: client }
    }

    pub async fn get_current_user(
        &self,
        token: &str,
    ) -> Result<DiscordUserModel, DiscordHttpError> {
        let response = self
            .http
            .get(format!("{API_URL}/users/@me"))
            .bearer_auth(token)
            .send()
            .await?;

        let data = try_parse_response(response).await?;
        Ok(data)
    }

    pub async fn get_guilds(
        &self,
        token: &str,
    ) -> Result<Vec<PartialGuildModel>, DiscordHttpError> {
        let response = self
            .http
            .get(format!("{API_URL}/users/@me/guilds"))
            .bearer_auth(token)
            .send()
            .await?;

        let data = try_parse_response(response).await?;
        Ok(data)
    }

    pub async fn exchange_code(
        &self,
        code: &str,
        redirect_uri: &str,
        client_id: &str,
        client_secret: &str,
    ) -> Result<AccessTokenResponse, DiscordHttpError> {
        let response = self
            .http
            .post(TOKEN_URL)
            .form(&ExchangeCodeRequest {
                grant_type: "authorization_code",
                code,
                redirect_uri,
            })
            .basic_auth(client_id, Some(client_secret))
            .send()
            .await?;

        let data = try_parse_response(response).await?;
        Ok(data)
    }

    pub async fn refresh_token(
        &self, 
        refresh_token: &str,
        client_id: &str,
        client_secret: &str,
    ) -> Result<AccessTokenResponse, DiscordHttpError> {
        let response = self
            .http
            .post(TOKEN_URL)
            .form(&RefreshTokenRequest { grant_type: "refresh_token", refresh_token, })
            .basic_auth(client_id, Some(client_secret))
            .send()
            .await?;

        let data = try_parse_response(response).await?;
        Ok(data)
    }
}

async fn try_parse_response<T: DeserializeOwned>(
    response: reqwest::Response,
) -> Result<T, DiscordHttpError> {
    match response.status() {
        StatusCode::TOO_MANY_REQUESTS => return Err(DiscordHttpError::RateLimited),
        StatusCode::INTERNAL_SERVER_ERROR | StatusCode::BAD_REQUEST => {
            let body = response.text().await.ok();

            // log and throw an error
            if let Some(body) = body {
                tracing::error!(%body, "discord client request failed");
                return Err(DiscordHttpError::Unknown(body));
            } else {
                tracing::error!(
                    "discord client request failed"
                );
                return Err(DiscordHttpError::Unknown(String::new()));
            }
        }
        _ => {}
    }
    let status = response.status();

    // try parse JSON response
    let data = match response.json::<T>().await {
        Ok(data) => data,
        Err(e) => {
            tracing::error!(%status, "{e}");
            
            return Err(DiscordHttpError::InvalidResponse);
        }
    };

    Ok(data)
}
