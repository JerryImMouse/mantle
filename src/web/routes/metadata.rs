//! Module which provides routes for bulk operations over UserMetadata

use crate::{
    state::AppState,
    web::{middleware::authenticate, routes::RouteResult},
};
use axum::{
    Json, Router,
    extract::{Path, State},
    middleware,
    response::IntoResponse,
    routing::delete,
};
use reqwest::StatusCode;

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/{key}", delete(bulk_delete).put(bulk_update))
        .layer(middleware::from_fn_with_state(state, authenticate))
}

#[tracing::instrument(skip(state))]
async fn bulk_delete(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> RouteResult<impl IntoResponse> {
    state.metadata.bulk_delete(&key).await?;
    Ok(StatusCode::OK)
}

#[tracing::instrument(skip(state))]
async fn bulk_update(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Json(value): Json<serde_json::Value>,
) -> RouteResult<impl IntoResponse> {
    state.metadata.bulk_update(&key, &value).await?;
    Ok(StatusCode::OK)
}
