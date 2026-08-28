use axum::{Router, extract::State, http::StatusCode, response::IntoResponse, routing::get};
use serde::Serialize;

use crate::{state::AppState, web::routes::RouteResult};
pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(health))
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Serialize)]
struct HealthResponseBody {
    status: String,
    bind_to: String,
}

#[cfg_attr(
    feature = "openapi",
    utoipa::path(
        get,
        tag = "health",
        description = "Health of an application",
        path = "/api/health",
        security(()),
        responses(
            (status = 200, body = HealthResponseBody)
        )
    )
)]
#[tracing::instrument(skip(state))]
async fn health(State(state): State<AppState>) -> RouteResult<impl IntoResponse> {
    Ok((
        StatusCode::OK,
        axum::Json(HealthResponseBody {
            status: "ok".to_string(),
            bind_to: format!("{}:{}", state.config.server.host, state.config.server.port),
        }),
    ))
}

#[cfg(feature = "openapi")]
pub mod openapi {
    use super::*;

    #[derive(utoipa::OpenApi)]
    #[openapi(paths(health), components(schemas(HealthResponseBody,)))]
    pub struct ApiDoc;
}
