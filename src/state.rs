use crate::{
    config::SharedConfig,
    services::{AccountService, DiscordOAuthService, DiscordService, MetadataService},
};
use std::sync::Arc;

pub struct AppStateInternal {
    pub config: SharedConfig,
    pub account: AccountService,
    pub discord_oauth: DiscordOAuthService,
    pub discord: DiscordService,
    pub metadata: MetadataService,
}

impl AppStateInternal {
    pub fn new_shared(
        config: SharedConfig,
        account: AccountService,
        discord_oauth: DiscordOAuthService,
        discord: DiscordService,
        metadata: MetadataService,
    ) -> AppState {
        Arc::new(AppStateInternal {
            config,
            account,
            discord_oauth,
            discord,
            metadata,
        })
    }
}

pub type AppState = std::sync::Arc<AppStateInternal>;
