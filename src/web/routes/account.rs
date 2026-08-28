use super::models::UserRequestQuery;
use crate::{
    state::AppState,
    web::{
        middleware::authenticate,
        routes::{RouteResult, models::IdentityDto},
    },
};
use axum::{
    Json, Router,
    extract::{Query, State},
    middleware,
    response::IntoResponse,
    routing::get,
};
use reqwest::StatusCode;

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/identities", get(get_linked_identities))
        .layer(middleware::from_fn_with_state(state, authenticate))
}

async fn get_linked_identities(
    State(state): State<AppState>,
    Query(req): Query<UserRequestQuery>,
) -> RouteResult<impl IntoResponse> {
    let identities = state
        .account
        .find_linked_identities(req.provider, &req.id)
        .await?;

    let dto = identities
        .into_iter()
        .map(IdentityDto::from)
        .collect::<Vec<_>>();
    Ok((StatusCode::OK, Json(dto)))
}
