use super::models::UserRequestQuery;
use crate::{
    state::AppState,
    web::{
        middleware::authenticate,
        routes::{
            RouteResult,
            models::{IdentityDto, UserMetadataDto},
        },
    },
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use reqwest::StatusCode;

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/identities", get(get_linked_identities))
        .route("/metadata/{key}", post(set_metadata).get(get_metadata))
        .layer(middleware::from_fn_with_state(state, authenticate))
}

#[tracing::instrument(skip(state))]
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

#[tracing::instrument(skip(state))]
async fn set_metadata(
    State(state): State<AppState>,
    Query(req): Query<UserRequestQuery>,
    Path(key): Path<String>,
    Json(value): Json<serde_json::Value>,
) -> RouteResult<impl IntoResponse> {
    let metadata = UserMetadataDto::from(
        state
            .account
            .metadata_set(req.provider, &req.id, &key, &value)
            .await?,
    );
    Ok((StatusCode::OK, Json(metadata)))
}

#[tracing::instrument(skip(state))]
async fn get_metadata(
    State(state): State<AppState>,
    Query(req): Query<UserRequestQuery>,
    Path(key): Path<String>,
) -> RouteResult<Response> {
    // had to do Response here instead of impl IntoResponse
    // because of different return types
    let metadata = match state
        .account
        .metadata_get(req.provider, &req.id, &key)
        .await?
    {
        Some(metadata) => UserMetadataDto::from(metadata),
        None => return Ok(StatusCode::NOT_FOUND.into_response()),
    };

    Ok((StatusCode::OK, Json(metadata)).into_response())
}
