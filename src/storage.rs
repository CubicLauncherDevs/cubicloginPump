use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub username: String,
    pub password_hash: String,
    pub registered_at: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AccountStore {
    pub accounts: BTreeMap<String, Account>,
}

impl AccountStore {
    pub fn get(&self, username: &str) -> Option<&Account> {
        self.accounts.get(&normalize_username(username))
    }

    pub fn contains(&self, username: &str) -> bool {
        self.accounts.contains_key(&normalize_username(username))
    }
}

pub struct LoadedAccounts {
    pub store: AccountStore,
    pub path: PathBuf,
}

pub fn load_or_create(data_dir: &Path) -> Result<LoadedAccounts, String> {
    fs::create_dir_all(data_dir)
        .map_err(|error| format!("failed to create {}: {error}", data_dir.display()))?;

    let path = data_dir.join("accounts.json");

    if !path.exists() {
        let store = AccountStore::default();
        let encoded = serde_json::to_string_pretty(&store)
            .map_err(|error| format!("failed to encode default accounts: {error}"))?;
        fs::write(&path, encoded)
            .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
        return Ok(LoadedAccounts { store, path });
    }

    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let store = serde_json::from_str(&raw)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))?;

    Ok(LoadedAccounts { store, path })
}

pub fn normalize_username(username: &str) -> String {
    username.to_lowercase()
}
