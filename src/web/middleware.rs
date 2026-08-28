use axum::{
    extract::{Request, State},
    middleware::Next,
    response::IntoResponse,
};

use crate::{state::AppState, web::WebError};

#[derive(Debug, thiserror::Error)]
pub enum AuthenticateError {
    #[error("provide an authorization header with a bearer token")]
    NoAuthorizationHeader,
    #[error("invalid authorization header, should be 'Bearer <token>' ")]
    InvalidAuthorizationHeader,

    #[error("unauthorized api token")]
    UnauthorizedApiToken,
}

impl From<AuthenticateError> for WebError {
    fn from(value: AuthenticateError) -> Self {
        match value {
            AuthenticateError::NoAuthorizationHeader | AuthenticateError::UnauthorizedApiToken => {
                Self::user(401, Some(value.to_string()))
            }
            AuthenticateError::InvalidAuthorizationHeader => {
                Self::user(400, Some(value.to_string()))
            }
        }
    }
}

pub async fn authenticate(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let authorization = match req.headers().get("authorization") {
        Some(authorization) => match authorization.to_str() {
            Ok(authorization) => authorization,
            Err(e) => {
                return Err(WebError::user(
                    400,
                    Some(format!("invalid authorization header value: {e}")),
                ));
            }
        },
        None => return Err(WebError::from(AuthenticateError::NoAuthorizationHeader)),
    };

    if !authorization.starts_with("Bearer ") {
        return Err(WebError::from(
            AuthenticateError::InvalidAuthorizationHeader,
        ));
    }

    let (_, key) = authorization
        .split_once(' ')
        .ok_or(AuthenticateError::InvalidAuthorizationHeader)?;

    if key != state.config.server.api_secret {
        return Err(WebError::from(AuthenticateError::UnauthorizedApiToken));
    }

    Ok(next.run(req).await)
}
