use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post},
};

use crate::{
    db::identities::IdentityProvider,
    state::AppState,
    web::{middleware::authenticate, routes::RouteResult},
};

use crate::web::dto::auth::*;

pub fn routes(state: AppState) -> Router<AppState> {
    let protected = Router::new()
        .route("/check", post(check))
        .route("/link", get(generate_link))
        .layer(middleware::from_fn_with_state(state, authenticate));

    Router::new()
        .route("/callback", get(callback))
        .merge(protected)
}

#[cfg_attr(
    feature = "openapi",
    utoipa::path(
        post,
        description = "Checks whether this External UserID has corresponding MantleUser and Discord Identity",
        path = "/api/auth/check",
        request_body(
            content = CheckRequestBody,
            description = "Provide an External UserID as `user_id`"
        ),
        tag = "auth",
        responses(
            (status = 200, body = CheckResponseBody),
            (status = "default", body = openapi::ErrorResponse),
        )
    )
)]
#[tracing::instrument(skip(state))]
async fn check(
    state: State<AppState>,
    req: Json<CheckRequestBody>,
) -> RouteResult<impl IntoResponse> {
    let result = state
        .account
        .check(IdentityProvider::External, &req.user_id)
        .await?;

    if !result.has(IdentityProvider::Discord) {
        Ok((
            StatusCode::OK,
            Json(CheckResponseBody {
                status: "discord_required".into(),
            }),
        ))
    } else {
        Ok((
            StatusCode::OK,
            Json(CheckResponseBody {
                status: "ok".into(),
            }),
        ))
    }
}

#[cfg_attr(
    feature = "openapi",
    utoipa::path(
        get,
        path = "/api/auth/callback",
        description = "Discord callback should point here",
        request_body(
            content = CallbackRequestQuery,
            description = "This should be supplied by discord itself"
        ),
        tag = "auth",
        security(()),
        responses(
            (status = 200),
            (status = "default", body = openapi::ErrorResponse),
        )
    )
)]
#[tracing::instrument(skip(state))]
async fn callback(
    state: State<AppState>,
    req: Query<CallbackRequestQuery>,
) -> RouteResult<impl IntoResponse> {
    state
        .discord_oauth
        .process_callback(&req.code, &req.state)
        .await?;
    Ok(StatusCode::OK)
}

#[cfg_attr(
    feature = "openapi",
    utoipa::path(
        get,
        path = "/api/auth/link",
        description = "Generate discord OAuth2 link",
        request_body(
            content = GenerateLinkRequestQuery,
            description = "Provide an External UserID as `user_id`"
        ),
        tag = "auth",
        responses(
            (status = 200, body = GenerateLinkResponseBody),
            (status = "default", body = openapi::ErrorResponse),
        )
    )
)]
async fn generate_link(
    state: State<AppState>,
    req: Query<GenerateLinkRequestQuery>,
) -> RouteResult<impl IntoResponse> {
    let link = state
        .discord_oauth
        .generate_link(IdentityProvider::External, req.0.user_id)?;
    Ok((StatusCode::OK, Json(GenerateLinkResponseBody { link })))
}

#[cfg(feature = "openapi")]
pub mod openapi {
    use super::*;
    pub use crate::web::openapi::ErrorResponse;

    #[derive(utoipa::OpenApi)]
    #[openapi(
        paths(check, generate_link, callback,),
        components(schemas(
            CheckRequestBody,
            CheckResponseBody,
            GenerateLinkRequestQuery,
            GenerateLinkResponseBody,
        ))
    )]
    pub struct ApiDoc;
}
