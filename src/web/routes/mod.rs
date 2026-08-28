pub mod health;
pub mod auth;
pub mod discord;

use axum::Router;

use crate::{state::AppState, web::error::WebError};
pub type RouteResult<T> = std::result::Result<T, WebError>;

pub fn build_router() -> Router<AppState> {
    Router::new()
        .nest("/api/health", health::routes())
        .nest("/api/auth", auth::routes())
        .nest("/api/discord", discord::routes())
}
