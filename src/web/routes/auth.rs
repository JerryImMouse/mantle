use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::{
    db::identities::IdentityProvider,
    state::AppState,
    web::{middleware::authenticate, routes::RouteResult},
};

pub fn routes(state: AppState) -> Router<AppState> {
    let protected = Router::new()
        .route("/check", post(check))
        .route("/link", get(generate_link))
        .layer(middleware::from_fn_with_state(state, authenticate));

    Router::new()
        .route("/callback", get(callback))
        .merge(protected)
}

#[derive(Debug, Deserialize)]
struct CheckRequestBody {
    user_id: String,
}

#[derive(Debug, Serialize)]
struct CheckResponseBody {
    status: String,
}

#[tracing::instrument(skip(state))]
async fn check(
    state: State<AppState>,
    req: Json<CheckRequestBody>,
) -> RouteResult<impl IntoResponse> {
    let result = state
        .account
        .check(IdentityProvider::SS14, &req.user_id)
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

#[derive(Debug, Serialize, Deserialize)]
struct CallbackRequestQuery {
    code: String,
    state: String,
}

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

#[derive(Debug, Deserialize)]
struct GenerateLinkRequestQuery {
    user_id: String,
}

#[derive(Debug, Serialize)]
struct GenerateLinkResponseBody {
    link: String,
}

async fn generate_link(
    state: State<AppState>,
    req: Query<GenerateLinkRequestQuery>,
) -> RouteResult<impl IntoResponse> {
    let link = state
        .discord_oauth
        .generate_link(IdentityProvider::SS14, req.0.user_id)?;
    Ok((StatusCode::OK, Json(GenerateLinkResponseBody { link })))
}
