use axum::{Router, extract::State, http::StatusCode, response::IntoResponse, routing::get};
use serde_json::json;

use crate::{state::AppState, web::routes::RouteResult};

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(health))
}

#[tracing::instrument(skip(state))]
async fn health(State(state): State<AppState>) -> RouteResult<impl IntoResponse> {
    Ok((
        StatusCode::OK,
        axum::Json(json!({
            "status": "ok",
            "bindto": format!("{}:{}", state.config.server.host, state.config.server.port),
        })),
    ))
}
