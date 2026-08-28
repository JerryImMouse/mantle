use axum::{
    Json, Router,
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
};
use reqwest::StatusCode;
use serde::Deserialize;

use crate::{db::identities::IdentityProvider, state::AppState, web::routes::RouteResult};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/user", get(get_current_user))
        .route("/user/guilds", get(get_guilds))
}

#[derive(Debug, Deserialize)]
pub struct UserRequestQuery {
    provider: IdentityProvider,
    id: String,
}

#[tracing::instrument(skip(state))]
async fn get_current_user(
    state: State<AppState>,
    req: Query<UserRequestQuery>,
) -> RouteResult<impl IntoResponse> {
    let user = state
        .discord
        .get_current_user(req.provider, &req.id)
        .await?;
    Ok((StatusCode::OK, Json(user)))
}

#[tracing::instrument(skip(state))]
async fn get_guilds(
    state: State<AppState>,
    req: Query<UserRequestQuery>,
) -> RouteResult<impl IntoResponse> {
    let guilds = state.discord.get_guilds(req.provider, &req.id).await?;
    Ok((StatusCode::OK, Json(guilds)))
}
