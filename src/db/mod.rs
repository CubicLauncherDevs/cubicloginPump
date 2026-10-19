pub mod crypto;
pub mod hashing;

mod fallback;
mod redb_store;

use std::path::Path;
use std::sync::Arc;

use crate::config::{EncryptionMode, PluginConfig};
use crate::storage::AccountRecord;

pub use crypto::Cipher;
pub use hashing::Argon2Config;

pub trait AccountRepo: Send + Sync {
    fn get(&self, username: &str) -> Result<Option<AccountRecord>, String>;

    fn create(&self, username: &str, record: &AccountRecord) -> Result<bool, String>;

    fn upsert(&self, username: &str, record: &AccountRecord) -> Result<(), String>;

    #[allow(dead_code)]
    fn all(&self) -> Result<Vec<(String, AccountRecord)>, String>;
}

pub fn open(data_dir: &Path, config: &PluginConfig) -> Result<Arc<dyn AccountRepo>, String> {
    let cipher = resolve_cipher(config)?;

    match redb_store::RedbStore::open(data_dir, cipher.clone()) {
        Ok(store) => {
            tracing::info!("using redb account database");
            Ok(Arc::new(store))
        }
        Err(error) => {
            tracing::warn!(
                error = %error,
                "redb database unavailable, using embedded fallback store"
            );
            Ok(Arc::new(fallback::FallbackStore::open(data_dir, cipher)?))
        }
    }
}

pub fn argon2_config(config: &PluginConfig) -> Argon2Config {
    Argon2Config {
        memory_kib: config.argon2_memory_kib,
        iterations: config.argon2_iterations,
        parallelism: config.argon2_parallelism,
    }
}

pub fn resolve_pepper(config: &PluginConfig) -> Option<Vec<u8>> {
    match std::env::var(&config.pepper_env) {
        Ok(value) if !value.is_empty() => Some(value.into_bytes()),
        _ => None,
    }
}

pub(crate) fn aad(username: &str) -> Vec<u8> {
    let mut aad = Vec::with_capacity(username.len() + 1);
    aad.extend_from_slice(username.as_bytes());
    aad.push(AccountRecord::FORMAT_VERSION);
    aad
}

fn resolve_cipher(config: &PluginConfig) -> Result<Option<Cipher>, String> {
    match config.encryption {
        EncryptionMode::Off => Ok(None),
        EncryptionMode::Auto => match std::env::var(&config.db_key_env) {
            Ok(value) if !value.is_empty() => Ok(Some(Cipher::from_hex(&value)?)),
            _ => {
                tracing::warn!(
                    env = %config.db_key_env,
                    "encryption key not set; storing records without encryption"
                );
                Ok(None)
            }
        },
        EncryptionMode::Required => {
            let value = std::env::var(&config.db_key_env).map_err(|_| {
                format!(
                    "encryption is required but the {} environment variable is not set",
                    config.db_key_env
                )
            })?;

            if value.is_empty() {
                return Err(format!(
                    "encryption is required but the {} environment variable is empty",
                    config.db_key_env
                ));
            }

            Ok(Some(Cipher::from_hex(&value)?))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;

    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "cubiclogin-{tag}-{}-{}",
            std::process::id(),
            crate::storage::now_secs()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn record() -> AccountRecord {
        AccountRecord {
            version: AccountRecord::FORMAT_VERSION,
            salt: vec![1, 2, 3, 4],
            argon2_memory_kib: 64,
            argon2_iterations: 1,
            argon2_parallelism: 1,
            hash: vec![5, 6, 7, 8],
            created_at: 1,
            updated_at: 1,
            last_login: 0,
            failed_attempts: 0,
            locked_until: 0,
        }
    }

    fn roundtrip(store: Arc<dyn AccountRepo>) {
        assert!(store.get("steve").unwrap().is_none());
        assert!(store.create("steve", &record()).unwrap());
        assert!(!store.create("steve", &record()).unwrap());
        assert!(store.get("steve").unwrap().is_some());

        let mut updated = record();
        updated.last_login = 42;
        store.upsert("steve", &updated).unwrap();
        assert_eq!(store.get("steve").unwrap().unwrap().last_login, 42);
        assert_eq!(store.all().unwrap().len(), 1);
    }

    #[test]
    fn fallback_roundtrip_with_encryption() {
        let dir = temp_dir("fallback");
        let cipher = Some(Cipher::from_hex(&"11".repeat(32)).unwrap());
        let store = Arc::new(fallback::FallbackStore::open(&dir, cipher).unwrap());
        roundtrip(store);
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn redb_roundtrip_with_encryption() {
        let dir = temp_dir("redb");
        let cipher = Some(Cipher::from_hex(&"22".repeat(32)).unwrap());
        let store = Arc::new(redb_store::RedbStore::open(&dir, cipher).unwrap());
        roundtrip(store);
        std::fs::remove_dir_all(dir).ok();
    }
}
