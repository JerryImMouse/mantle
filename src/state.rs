use crate::{
    config::SharedConfig,
    services::{AccountService, DiscordOAuthService, DiscordService},
};
use std::sync::Arc;

pub struct AppStateInternal {
    pub config: SharedConfig,
    pub account: AccountService,
    pub discord_oauth: DiscordOAuthService,
    pub discord: DiscordService,
}

impl AppStateInternal {
    pub fn new_shared(
        config: SharedConfig,
        account: AccountService,
        discord_oauth: DiscordOAuthService,
        discord: DiscordService,
    ) -> AppState {
        Arc::new(AppStateInternal {
            config: config,
            account,
            discord_oauth,
            discord,
        })
    }
}

pub type AppState = std::sync::Arc<AppStateInternal>;
