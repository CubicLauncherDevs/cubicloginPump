mod commands;
mod config;
mod db;
mod events;
mod i18n;
mod permissions;
mod state;
mod storage;

use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use pumpkin_plugin_api::{Context, Plugin, PluginMetadata, Result, register_plugin};

use crate::state::PluginState;

pub struct CubicLoginPlugin {
    state: Arc<RwLock<PluginState>>,
}

impl Plugin for CubicLoginPlugin {
    fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(PluginState::default())),
        }
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "CubicLogin".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            authors: vec!["CubicLauncherDevs".to_string()],
            description: "Authentication plugin for Pumpkin servers".to_string(),
            dependencies: Vec::new(),
            permissions: vec![
                pumpkin_plugin_api::permissions::FS_READ_DATA.to_string(),
                pumpkin_plugin_api::permissions::FS_WRITE_DATA.to_string(),
            ],
        }
    }

    fn on_load(&self, context: Context) -> Result<()> {
        let data_dir = PathBuf::from(context.get_data_folder());
        let loaded_config = config::load_or_create(&data_dir)?;
        let messages = i18n::load(&data_dir, &loaded_config.config.language)?;
        let language = loaded_config.config.language.clone();
        let pepper = db::resolve_pepper(&loaded_config.config);
        let store = db::open(&data_dir, &loaded_config.config)?;

        {
            let mut state = self
                .state
                .write()
                .map_err(|_| "plugin state lock poisoned".to_string())?;

            state.config = loaded_config.config;
            state.messages = Arc::new(messages);
            state.store = Some(store);
            state.pepper = pepper;
            state.data_dir = data_dir.clone();
        }

        permissions::register(&context)?;
        commands::register(&context, Arc::clone(&self.state))?;
        events::register(&context, Arc::clone(&self.state))?;

        tracing::info!(
            config = %loaded_config.path.display(),
            data_dir = %data_dir.display(),
            language = %language,
            "CubicLogin loaded"
        );

        Ok(())
    }

    fn on_unload(&self, context: Context) -> Result<()> {
        for player in context.get_server().get_all_players() {
            player.set_movement_locked(false);
        }

        if let Ok(mut state) = self.state.write() {
            state.authenticated.clear();
        }

        tracing::info!("CubicLogin unloaded");
        Ok(())
    }
}

register_plugin!(CubicLoginPlugin);
