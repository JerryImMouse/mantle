use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ExchangeCodeRequest<'a> {
    pub grant_type: &'a str,
    pub code: &'a str,
    pub redirect_uri: &'a str,
}

#[derive(Debug, Serialize)]
pub struct RefreshTokenRequest<'a> {
    pub grant_type: &'a str,
    pub refresh_token: &'a str,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Snowflake(String);

impl Snowflake {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Deserialize)]
pub struct AccessTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,

    pub token_type: String,
    pub scope: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordUserModel {
    pub id: Snowflake,
    pub username: String,
    pub discriminator: String,
    pub global_name: Option<String>,
    pub avatar: Option<String>,
    pub bot: Option<bool>,
    pub system: Option<bool>,
    pub mfa_enabled: Option<bool>,
    pub locale: Option<String>
}

// https://docs.discord.com/developers/resources/user#guild-preview-object
#[derive(Debug, Serialize, Deserialize)]
pub struct PartialGuildModel {
    pub id: Snowflake,
    pub name: String,
    pub icon: Option<String>,
    pub splash: Option<String>,
    pub discovery_splash: Option<String>,
    pub features: Vec<String>,
    pub approximate_member_count: Option<usize>,
    pub approximate_presence_count: Option<usize>,
    pub description: Option<String>,
}
