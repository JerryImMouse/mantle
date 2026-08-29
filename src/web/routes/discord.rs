use axum::{
    Json, Router,
    extract::{Path, Query, State},
    middleware,
    response::IntoResponse,
    routing::get,
};
use reqwest::StatusCode;

use crate::web::dto::UserRequestQuery;
use crate::{
    integrations::discord::Snowflake,
    state::AppState,
    web::{middleware::authenticate, routes::RouteResult},
};

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/user", get(get_current_user))
        .route("/user/guilds", get(get_guilds))
        .route("/user/guilds/{guild_id}/member", get(get_guild_member))
        .layer(middleware::from_fn_with_state(state, authenticate))
}

#[cfg_attr(
    feature = "openapi",
    utoipa::path(
        get,
        description = "Public API wrapper for Discord API, cached under the hood",
        path = "/api/discord/user",
        params(UserRequestQuery),
        tag = "discord",
        responses(
            (status = 200, body = openapi::DiscordUserModel),
            (status = "default", body = openapi::ErrorResponse),
        )
    )
)]
#[tracing::instrument(skip(state))]
async fn get_current_user(
    state: State<AppState>,
    req: Query<UserRequestQuery>,
) -> RouteResult<impl IntoResponse> {
    let user = state
        .discord
        .get_current_user(req.provider, &req.id)
        .await?;
    Ok((StatusCode::OK, Json(user)))
}

#[cfg_attr(
    feature = "openapi",
    utoipa::path(
        get,
        description = "Public API wrapper for Discord API, cached under the hood",
        path = "/api/discord/user/guilds",
        params(UserRequestQuery),
        tag = "discord",
        responses(
            (status = 200, body = Vec<openapi::PartialGuildModel>),
            (status = "default", body = openapi::ErrorResponse),
        )
    )
)]
#[tracing::instrument(skip(state))]
async fn get_guilds(
    state: State<AppState>,
    req: Query<UserRequestQuery>,
) -> RouteResult<impl IntoResponse> {
    let guilds = state.discord.get_guilds(req.provider, &req.id).await?;
    Ok((StatusCode::OK, Json(guilds)))
}

#[cfg_attr(
    feature = "openapi",
    utoipa::path(
        get,
        description = "Public API wrapper for Discord API, cached under the hood",
        path = "/api/discord/user/guilds/{guild_id}/member",
        params(
            UserRequestQuery,
            ("guild_id" = Snowflake, Path, description = "Guild ID to fetch member from")
        ),
        tag = "discord",
        responses(
            (status = 200, body = openapi::GuildMemberModel),
            (status = "default", body = openapi::ErrorResponse),
        )
    )
)]
async fn get_guild_member(
    state: State<AppState>,
    req: Query<UserRequestQuery>,
    Path(guild_id): Path<Snowflake>,
) -> RouteResult<impl IntoResponse> {
    let guild_member = state
        .discord
        .get_guild_member(req.provider, &req.id, guild_id)
        .await?;

    Ok((StatusCode::OK, Json(guild_member)))
}

#[cfg(feature = "openapi")]
pub mod openapi {
    use super::*;
    pub use crate::integrations::discord::{DiscordUserModel, GuildMemberModel, PartialGuildModel};
    pub use crate::web::openapi::ErrorResponse;

    #[derive(utoipa::OpenApi)]
    #[openapi(
        paths(get_current_user, get_guilds, get_guild_member),
        components(schemas(DiscordUserModel, PartialGuildModel))
    )]
    pub struct ApiDoc;
}
