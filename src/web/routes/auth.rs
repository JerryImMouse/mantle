use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    middleware,
    response::{IntoResponse, Redirect, Response},
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
        params(
            CallbackRequestQuery,
        ),
        tag = "auth",
        security(()),
        responses(
            (status = 200, description = "if redirect_uri is not set in the config, this reponse will be returned"),
            (status = 308, description = "if the redirect_uri IS set - then the user will be redirected to specfied URI"),
            (status = "default", body = openapi::ErrorResponse),
        )
    )
)]
#[tracing::instrument(skip(state))]
async fn callback(
    state: State<AppState>,
    req: Query<CallbackRequestQuery>,
) -> RouteResult<Response> {
    state
        .discord_oauth
        .process_callback(&req.code, &req.state)
        .await?;
    if let Some(redirect_uri) = state.config.app.redirect_uri.as_ref() {
        Ok(Redirect::permanent(redirect_uri.as_str()).into_response())
    } else {
        Ok(StatusCode::OK.into_response())
    }
}

#[cfg_attr(
    feature = "openapi",
    utoipa::path(
        get,
        path = "/api/auth/link",
        description = "Generate discord OAuth2 link",
        params(
            GenerateLinkRequestQuery,
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
        components(schemas(CheckRequestBody, CheckResponseBody, GenerateLinkResponseBody,))
    )]
    pub struct ApiDoc;
}
