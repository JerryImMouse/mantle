use std::net::Ipv4Addr;

use crate::config::{Config, DatabaseConfig, DiscordConfig, ServerConfig, error::ConfigError};

#[derive(Debug)]
pub struct RuntimeConfig {
    pub server: RuntimeServerConfig,
    pub database: RuntimeDatabaseConfig,
    pub discord: RuntimeDiscordConfig,
}

#[derive(Debug)]
pub struct RuntimeServerConfig {
    pub host: Ipv4Addr,
    pub port: u16,
}

#[derive(Debug)]
pub struct RuntimeDiscordConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub state_secret: String,
}

#[derive(Debug)]
pub struct RuntimeDatabaseConfig {
    pub url: String,
}

impl Config {
    pub fn validate(self) -> Result<RuntimeConfig, ConfigError> {
        let server = self.server.validate()?;
        let database = self.database.validate()?;
        let discord = self.discord.validate()?;
        Ok(RuntimeConfig {
            server,
            database,
            discord,
        })
    }
}

impl DatabaseConfig {
    fn validate(self) -> Result<RuntimeDatabaseConfig, ConfigError> {
        Ok(RuntimeDatabaseConfig { url: self.url })
    }
}

impl DiscordConfig {
    fn validate(self) -> Result<RuntimeDiscordConfig, ConfigError> {
        Ok(RuntimeDiscordConfig {
            client_id: self.client_id,
            client_secret: self.client_secret,
            redirect_uri: self.redirect_uri,
            state_secret: self.state_secret,
        })
    }
}

impl ServerConfig {
    fn validate(self) -> Result<RuntimeServerConfig, ConfigError> {
        let addr: std::net::Ipv4Addr = self
            .host
            .parse()
            .map_err(|_| ConfigError::invalid_host(self.host.clone()))?;

        if self.port <= 1000 {
            return Err(ConfigError::invalid_port(self.port));
        }

        Ok(RuntimeServerConfig {
            host: addr,
            port: self.port,
        })
    }
}
