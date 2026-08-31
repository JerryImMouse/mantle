use crate::web::dto::account::*;
use crate::web::dto::{IdentityDto, UserMetadataDto, UserRequestQuery};
use crate::{
    state::AppState,
    web::{middleware::authenticate, routes::RouteResult},
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
        .route(
            "/metadata/{key}",
            post(set_metadata).get(get_metadata).delete(delete_metadata),
        )
        .layer(middleware::from_fn_with_state(state, authenticate))
}

#[cfg_attr(
    feature = "openapi",
    utoipa::path(
        delete,
        tag = "account",
        description = "Returns identities linked to provided identity",
        path = "/api/account/identities",
        params(UserRequestQuery),
        responses(
            (status = 200, body = Vec<IdentityDto>),
            (status = "default", body = openapi::ErrorResponse),
        )
    )
)]
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

#[cfg_attr(
    feature = "openapi",
    utoipa::path(
        post,
        tags = ["account", "metadata"],
        description = "Sets metadata linked with this mantle account",
        path = "/api/account/metadata/{key}",
        params(
            UserRequestQuery,
            ("key" = String, Path, description = "Metadata key to set")
        ),
        request_body(
            content = MetadataRequestBody,
            description = "Any JSON value to make metadata value and a private flag",
        ),
        responses(
            (status = 200, body = UserMetadataDto),
            (status = "default", body = openapi::ErrorResponse),
        )
    )
)]
#[tracing::instrument(skip(state))]
async fn set_metadata(
    State(state): State<AppState>,
    Query(req): Query<UserRequestQuery>,
    Path(key): Path<String>,
    Json(body): Json<MetadataRequestBody>,
) -> RouteResult<impl IntoResponse> {
    let metadata = UserMetadataDto::from(
        state
            .account
            .metadata_set(req.provider, &req.id, &key, &body.value, body.private)
            .await?,
    );
    Ok((StatusCode::OK, Json(metadata)))
}

#[cfg_attr(
    feature = "openapi",
    utoipa::path(
        get,
        tags = ["account", "metadata"],
        description = "Gets metadata linked with this mantle account",
        path = "/api/account/metadata/{key}",
        params(
            UserRequestQuery,
            ("key" = String, Path, description = "Metadata key to get")
        ),
        responses(
            (status = 200, body = UserMetadataDto),
            (status = 404, description = "Metadata with provided key is not found"),
            (status = "default", body = openapi::ErrorResponse),
        )
    )
)]
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

#[cfg_attr(
    feature = "openapi",
    utoipa::path(
        delete,
        tags = ["account", "metadata"],
        description = "Deletes metadata by key linked with this mantle account",
        path = "/api/account/metadata/{key}",
        params(
            UserRequestQuery,
            ("key" = String, Path, description = "Metadata key to delete")
        ),
        responses(
            (status = 200, body = UserMetadataDto),
            (status = 404, description = "Metadata with provided key is not found"),
            (status = "default", body = openapi::ErrorResponse),
        )
    )
)]
async fn delete_metadata(
    State(state): State<AppState>,
    Query(req): Query<UserRequestQuery>,
    Path(key): Path<String>,
) -> RouteResult<Response> {
    // had to do Response here instead of impl IntoResponse
    // because of different return types
    let metadata = match state
        .account
        .metadata_delete(req.provider, &req.id, &key)
        .await?
    {
        Some(metadata) => UserMetadataDto::from(metadata),
        None => return Ok(StatusCode::NOT_FOUND.into_response()),
    };

    Ok((StatusCode::OK, Json(metadata)).into_response())
}

#[cfg(feature = "openapi")]
pub mod openapi {
    use super::*;
    pub use crate::web::openapi::ErrorResponse;

    #[derive(utoipa::OpenApi)]
    #[openapi(
        paths(get_linked_identities, get_metadata, delete_metadata, set_metadata,),
        components(schemas(UserMetadataDto, IdentityDto,))
    )]
    pub struct ApiDoc;
}
