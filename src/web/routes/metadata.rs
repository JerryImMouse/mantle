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

#[cfg_attr(
    feature = "openapi",
    utoipa::path(
        delete,
        tag = "metadata",
        description = "Bulk delete all metadata records with provided key",
        path = "/api/metadata/{key}",
        params(
            ("key" = String, Path, description = "Metadata key to delete"),
        ),
        responses(
            (status = 200),
            (status = "default", body = openapi::ErrorResponse),
        )
    )
)]
#[tracing::instrument(skip(state))]
async fn bulk_delete(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> RouteResult<impl IntoResponse> {
    state.metadata.bulk_delete(&key).await?;
    Ok(StatusCode::OK)
}

#[cfg_attr(
    feature = "openapi",
    utoipa::path(
        put,
        tag = "metadata",
        description = "Bulk update all metadata records with provided key",
        path = "/api/metadata/{key}",
        params(
            ("key" = String, Path, description = "Metadata key to update"),
        ),
        request_body(
            content = serde_json::Value,
            description = "Any JSON value",
        ),
        responses(
            (status = 200),
            (status = "default", body = openapi::ErrorResponse),
        )
    )
)]
#[tracing::instrument(skip(state))]
async fn bulk_update(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Json(value): Json<serde_json::Value>,
) -> RouteResult<impl IntoResponse> {
    state.metadata.bulk_update(&key, &value).await?;
    Ok(StatusCode::OK)
}

#[cfg(feature = "openapi")]
pub mod openapi {
    use super::*;
    pub use crate::web::openapi::ErrorResponse;

    #[derive(utoipa::OpenApi)]
    #[openapi(paths(bulk_delete, bulk_update,))]
    pub struct ApiDoc;
}
