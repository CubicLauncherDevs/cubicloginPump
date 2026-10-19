use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::config::PluginConfig;
use crate::db::AccountRepo;

#[derive(Default)]
pub struct PluginState {
    pub config: PluginConfig,
    pub store: Option<Arc<dyn AccountRepo>>,
    pub pepper: Option<Vec<u8>>,
    pub data_dir: PathBuf,
}

pub type SharedState = Arc<RwLock<PluginState>>;
