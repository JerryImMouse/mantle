pub mod account;
pub mod auth;
pub mod discord;
pub mod health;
mod models;

use axum::Router;

use crate::{state::AppState, web::error::WebError};
pub type RouteResult<T> = std::result::Result<T, WebError>;

pub fn build_router(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/api/health", health::routes())
        .nest("/api/auth", auth::routes(state.clone()))
        .nest("/api/discord", discord::routes(state.clone()))
        .nest("/api/account", account::routes(state))
}
