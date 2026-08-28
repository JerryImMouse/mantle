use crate::integrations::discord::error::DiscordHttpError;

pub type Result<T> = std::result::Result<T, InternalError>;

#[derive(Debug, thiserror::Error)]
pub enum InternalError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("config error: {0}")]
    Config(#[from] crate::config::error::ConfigError),

    #[error("db error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::error::Error),

    #[error("jwt error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("discord http error: {0}")]
    Discord(#[from] DiscordHttpError),

    #[error("failed to parse url: {0}")]
    UrlParse(#[from] url::ParseError),
}
