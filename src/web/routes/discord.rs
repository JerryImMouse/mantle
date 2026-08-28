use axum::{
    Json, Router,
    extract::{Path, Query, State},
    middleware,
    response::IntoResponse,
    routing::get,
};
use reqwest::StatusCode;
use serde::Deserialize;

use crate::{
    db::identities::IdentityProvider,
    integrations::discord::Snowflake,
    state::AppState,
    web::{middleware::authenticate, routes::RouteResult},
};

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/user", get(get_current_user))
        .route("/user/guilds", get(get_guilds))
        .route("/user/guilds/{guild_id}/member", get(get_guild_member))
        .layer(middleware::from_fn_with_state(state, authenticate))
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

async fn get_guild_member(
    state: State<AppState>,
    req: Query<UserRequestQuery>,
    guild_id: Path<Snowflake>,
) -> RouteResult<impl IntoResponse> {
    let guild_member = state
        .discord
        .get_guild_member(req.provider, &req.id, guild_id.0)
        .await?;

    Ok((StatusCode::OK, Json(guild_member)))
}
