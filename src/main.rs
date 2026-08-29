use mantle::{
    config::{Config, SharedConfig},
    db::{self, MantleDb},
    error::InternalError,
    integrations::discord::DiscordClient,
    services::{AccountService, DiscordOAuthService, DiscordService, MetadataService},
    state::{AppState, AppStateInternal},
    web,
};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{Layer, layer::SubscriberExt};

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
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Bye!!!");
    Ok(())
}

fn build_state(config: SharedConfig, db: MantleDb) -> AppState {
    let discord_client = DiscordClient::new();
    AppStateInternal::new_shared(
        config.clone(),
        AccountService::new(db.clone()),
        DiscordOAuthService::new(db.clone(), discord_client.clone(), config.clone()),
        DiscordService::new(db.clone(), discord_client, config),
        MetadataService::new(db),
    )
}

fn setup_tracing() {
    use tracing_subscriber::EnvFilter;

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "mantle=debug,tower_http=debug,warn".into());

    let registry = tracing_subscriber::registry().with(
        tracing_subscriber::fmt::layer()
            .with_file(true)
            .with_line_number(true)
            .with_target(false)
            .with_filter(filter),
    );

    tracing::subscriber::set_global_default(registry).expect("Failed to setup `tracing`");
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("shutdown signal received");
}
