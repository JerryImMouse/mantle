//! Module which provides routes for bulk operations over UserMetadata

use crate::{
    state::AppState,
    web::{middleware::authenticate, routes::RouteResult},
};
use axum::{
    Router,
    extract::{Path, State},
    middleware,
    response::IntoResponse,
    routing::delete,
};
use reqwest::StatusCode;

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/{key}", delete(bulk_delete))
        .layer(middleware::from_fn_with_state(state, authenticate))
}

async fn bulk_delete(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> RouteResult<impl IntoResponse> {
    state.metadata.bulk_delete(&key).await?;
    Ok(StatusCode::OK)
}
