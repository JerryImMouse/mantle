use serde::{Deserialize, Serialize};

mod error;
pub use error::ConfigError;

mod runtime;
pub use runtime::*;
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    server: ServerConfig,
    database: DatabaseConfig,
    discord: DiscordConfig,

    #[serde(default)]
    app: AppConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    host: String,
    port: u16,
    api_secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordConfig {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    state_secret: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AppConfig {
    redirect_uri: Option<Url>,
}

impl Config {
    #[tracing::instrument]
    pub fn load_from_file<P: AsRef<std::path::Path> + std::fmt::Debug>(
        path: P,
    ) -> Result<Config, ConfigError> {
        let path = path.as_ref();
        let data = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::io_error(e, path.to_path_buf()))?;

        let de: Config = toml::from_str(&data)?;
        Ok(de)
    }

    #[tracing::instrument]
    pub fn apply_env(mut self) -> Self {
        self.server.apply_env();
        self.database.apply_env();
        self.discord.apply_env();
        self.app.apply_env();
        self
    }
}

impl DatabaseConfig {
    pub fn apply_env(&mut self) {
        override_parse("APP_DATABASE_URL", &mut self.url);
    }
}

impl ServerConfig {
    pub fn apply_env(&mut self) {
        override_string("APP_HOST", &mut self.host);
        override_string("APP_API_SECRET", &mut self.api_secret);
        override_parse("APP_PORT", &mut self.port);
    }
}

impl DiscordConfig {
    pub fn apply_env(&mut self) {
        override_string("APP_DISCORD_CLIENT_ID", &mut self.client_id);
        override_string("APP_DISCORD_CLIENT_SECRET", &mut self.client_secret);
        override_string("APP_DISCORD_REDIRECT_URI", &mut self.redirect_uri);
        override_string("APP_DISCORD_STATE_SECRET", &mut self.state_secret);
    }
}

impl AppConfig {
    pub fn apply_env(&mut self) {
        // some hack, probably can use unsafe or smth to do it, but I won't :P
        let mut url = Url::parse("http://example.com").unwrap();
        if override_parse("APP_CALLBACK_REDIRECT_URI", &mut url) {
            self.redirect_uri = Some(url);
        }
    }
}

fn override_string(key: &str, target: &mut String) {
    if let Ok(v) = dotenvy::var(key) {
        *target = v;
    }
}

fn override_parse<T: std::str::FromStr>(key: &str, target: &mut T) -> bool
where
    T::Err: std::fmt::Debug,
{
    if let Ok(v) = dotenvy::var(key) {
        *target = v.parse().expect("invalid environment value");
        true
    } else {
        false
    }
}
