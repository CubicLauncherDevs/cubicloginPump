use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EncryptionMode {
    Required,
    Auto,
    Off,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PluginConfig {
    pub require_login_on_join: bool,
    pub min_password_length: usize,
    pub max_password_length: usize,
    pub session_timeout_seconds: u64,
    pub max_failed_attempts: u32,
    pub lockout_seconds: u64,
    pub argon2_memory_kib: u32,
    pub argon2_iterations: u32,
    pub argon2_parallelism: u32,
    pub encryption: EncryptionMode,
    pub db_key_env: String,
    pub pepper_env: String,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            require_login_on_join: true,
            min_password_length: 8,
            max_password_length: 64,
            session_timeout_seconds: 300,
            max_failed_attempts: 5,
            lockout_seconds: 300,
            argon2_memory_kib: 19456,
            argon2_iterations: 2,
            argon2_parallelism: 1,
            encryption: EncryptionMode::Required,
            db_key_env: "CUBICLOGIN_DB_KEY".to_string(),
            pepper_env: "CUBICLOGIN_PEPPER".to_string(),
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
    let parsed: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))?;
    let config: PluginConfig = serde_json::from_value(parsed.clone())
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))?;

    ensure_all_options(&path, &parsed, &config)?;

    Ok(LoadedConfig { config, path })
}

fn ensure_all_options(
    path: &Path,
    parsed: &serde_json::Value,
    config: &PluginConfig,
) -> Result<(), String> {
    let defaults = serde_json::to_value(PluginConfig::default())
        .map_err(|error| format!("failed to inspect default config: {error}"))?;

    let (serde_json::Value::Object(defaults), serde_json::Value::Object(current)) =
        (&defaults, parsed)
    else {
        return Ok(());
    };

    if defaults.keys().all(|key| current.contains_key(key)) {
        return Ok(());
    }

    let merged = serde_json::to_string_pretty(config)
        .map_err(|error| format!("failed to encode config: {error}"))?;
    fs::write(path, merged)
        .map_err(|error| format!("failed to update {}: {error}", path.display()))?;

    Ok(())
}
