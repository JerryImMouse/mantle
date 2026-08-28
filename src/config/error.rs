#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("invalid host endpoint provided: {0}, it should be a valid IP address")]
    InvalidHost(String),
    #[error("invalid port provided: {0}, it should be valid integer between 0-65535")]
    InvalidPort(u16),

    #[error("failed to deserialize configuration: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("failed to load configuration from: {file}: {io}")]
    ConfigIoError {
        io: std::io::Error,
        file: std::path::PathBuf,
    }
}

impl ConfigError {
    pub fn invalid_port(port: u16) -> ConfigError {
        ConfigError::InvalidPort(port)
    }

    pub fn invalid_host(str: String) -> ConfigError {
        ConfigError::InvalidHost(str)
    }

    pub fn io_error(io: std::io::Error, file: std::path::PathBuf) -> ConfigError {
        ConfigError::ConfigIoError { io, file }
    }
}
