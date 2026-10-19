use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PluginConfig {
    pub require_login_on_join: bool,
    pub min_password_length: usize,
    pub max_password_length: usize,
    pub session_timeout_seconds: u64,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            require_login_on_join: true,
            min_password_length: 8,
            max_password_length: 64,
            session_timeout_seconds: 300,
        }
    }
}

pub struct LoadedConfig {
    pub config: PluginConfig,
    pub path: PathBuf,
}

pub fn load_or_create(data_dir: &Path) -> Result<LoadedConfig, String> {
    fs::create_dir_all(data_dir)
        .map_err(|error| format!("failed to create {}: {error}", data_dir.display()))?;

    let path = data_dir.join("config.json");

    if !path.exists() {
        let config = PluginConfig::default();
        let encoded = serde_json::to_string_pretty(&config)
            .map_err(|error| format!("failed to encode default config: {error}"))?;
        fs::write(&path, encoded)
            .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
        return Ok(LoadedConfig { config, path });
    }

    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let config = serde_json::from_str(&raw)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))?;

    Ok(LoadedConfig { config, path })
}
