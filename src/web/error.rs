use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::error::InternalError;

#[derive(Debug, Serialize, Deserialize)]
pub struct WebErrorDetails {
    status_code: u16,
    error_msg: Option<String>,
}

#[derive(Debug)]
pub enum WebError {
    User(WebErrorDetails),
    Internal(InternalError),
}

impl WebError {
    pub fn user(status_code: u16, error_msg: Option<String>) -> Self {
        Self::User(WebErrorDetails { status_code, error_msg })
    }
}

impl std::fmt::Display for WebError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let WebError::Internal(e) = self {
            write!(f, "Internal error occured: {e}")
        } else {
            write!(f, "Error logged: {self:?}")
        }
    }
}

impl std::error::Error for WebError {}

impl axum::response::IntoResponse for WebError {
    fn into_response(self) -> axum::response::Response {
        match self {
            WebError::Internal(e) => {
                tracing::error!("{e}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            WebError::User(user) => {
                let status_code = user.status_code;
                (
                    StatusCode::from_u16(status_code)
                        .expect("Status code had to be between 100 and 1000"),
                    axum::Json(user),
                )
                    .into_response()
            }
        }
    }
}

impl From<InternalError> for WebError {
    fn from(value: InternalError) -> Self {
        Self::Internal(value)
    }
}
