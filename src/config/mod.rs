use serde::{Deserialize, Serialize};

mod error;
pub use error::ConfigError;

mod runtime;
pub use runtime::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    server: ServerConfig,
    database: DatabaseConfig,
    discord: DiscordConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    host: String,
    port: u16,
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

fn override_string(key: &str, target: &mut String) {
    if let Ok(v) = dotenvy::var(key) {
        *target = v;
    }
}

fn override_parse<T: std::str::FromStr>(key: &str, target: &mut T)
where
    T::Err: std::fmt::Debug,
{
    if let Ok(v) = dotenvy::var(key) {
        *target = v.parse().expect("invalid environment value")
    }
}
