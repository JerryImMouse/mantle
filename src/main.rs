use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use crate::{
    config::{Config, SharedConfig},
    db::MantleDb,
    error::InternalError,
    integrations::discord::DiscordClient,
    services::{AccountService, DiscordOAuthService, DiscordService},
    state::{AppState, AppStateInternal},
};

pub mod config;
pub mod db;
pub mod error;
pub mod integrations;
pub mod services;
pub mod state;
pub mod web;

#[tokio::main]
#[tracing::instrument]
async fn main() -> Result<(), InternalError> {
    setup_tracing();

    let config = Config::load_from_file("config.toml")?
        .apply_env()
        .validate()?
        .into_shared();

    let db = db::setup(&config).await?;

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&addr).await?;

    tracing::info!(%addr, "server started");

    let state = build_state(config, db);

    let router = web::build_router(state.clone())
        .layer(TraceLayer::new_for_http())
        .with_state(state);
    axum::serve(listener, router).await?;
    Ok(())
}

fn build_state(config: SharedConfig, db: MantleDb) -> AppState {
    let discord_client = DiscordClient::new();
    AppStateInternal::new_shared(
        config.clone(),
        AccountService::new(db.clone()),
        DiscordOAuthService::new(db.clone(), discord_client.clone(), config.clone()),
        DiscordService::new(db, discord_client, config),
    )
}

fn setup_tracing() {
    use tracing_subscriber::EnvFilter;

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "mantle=debug,tower_http=debug,warn".into());

    let sub = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .finish();

    tracing::subscriber::set_global_default(sub).expect("Failed to setup `tracing`");
}
