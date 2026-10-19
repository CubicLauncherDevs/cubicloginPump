mod commands;
mod config;
mod events;
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
        let loaded_accounts = storage::load_or_create(&data_dir)?;

        {
            let mut state = self
                .state
                .write()
                .map_err(|_| "plugin state lock poisoned".to_string())?;

            state.config = loaded_config.config;
            state.accounts = loaded_accounts.store;
            state.data_dir = data_dir.clone();
        }

        permissions::register(&context)?;
        commands::register(&context, Arc::clone(&self.state))?;
        events::register(&context, Arc::clone(&self.state))?;

        tracing::info!(
            config = %loaded_config.path.display(),
            accounts = %loaded_accounts.path.display(),
            data_dir = %data_dir.display(),
            "CubicLogin loaded"
        );

        Ok(())
    }

    fn on_unload(&self, _context: Context) -> Result<()> {
        tracing::info!("CubicLogin unloaded");
        Ok(())
    }
}

register_plugin!(CubicLoginPlugin);
