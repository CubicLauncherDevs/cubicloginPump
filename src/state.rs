use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::config::PluginConfig;
use crate::storage::AccountStore;

#[derive(Debug, Default)]
pub struct PluginState {
    pub config: PluginConfig,
    pub accounts: AccountStore,
    pub data_dir: PathBuf,
}

pub type SharedState = Arc<RwLock<PluginState>>;
