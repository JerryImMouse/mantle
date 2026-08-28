#[derive(Debug, thiserror::Error)]
pub enum DiscordHttpError {
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error), 
    #[error("rate limited")]
    RateLimited,
    #[error("invalid response")]
    InvalidResponse,
    #[error("unknown error: {0}")]
    Unknown(String),
}


