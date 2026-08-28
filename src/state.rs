use crate::{
    config::runtime::RuntimeConfig,
    services::{account::AccountService, discord_oauth::DiscordOAuthService},
};
use std::sync::Arc;

pub struct AppStateInternal {
    pub config: Arc<RuntimeConfig>,
    pub account: AccountService,
    pub discord_oauth: DiscordOAuthService,
}

impl AppStateInternal {
    pub fn new(
        config: Arc<RuntimeConfig>,
        account: AccountService,
        discord_oauth: DiscordOAuthService,
    ) -> AppStateInternal {
        AppStateInternal {
            config: config,
            account,
            discord_oauth,
        }
    }
}

pub type AppState = std::sync::Arc<AppStateInternal>;
