use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use super::crypto::Cipher;
use super::{AccountRepo, aad};
use crate::storage::AccountRecord;

const STORE_FILE: &str = "accounts.db";
const STORE_VERSION: u8 = 1;

#[derive(Serialize, Deserialize)]
struct StoreFile {
    version: u8,
    records: BTreeMap<String, Vec<u8>>,
}

impl Default for StoreFile {
    fn default() -> Self {
        Self {
            version: STORE_VERSION,
            records: BTreeMap::new(),
        }
    }
}

pub struct FallbackStore {
    path: PathBuf,
    cipher: Option<Cipher>,
    state: Mutex<StoreFile>,
}

impl FallbackStore {
    pub fn open(data_dir: &Path, cipher: Option<Cipher>) -> Result<Self, String> {
        std::fs::create_dir_all(data_dir)
            .map_err(|error| format!("failed to create {}: {error}", data_dir.display()))?;

        let path = data_dir.join(STORE_FILE);
        let state = if path.exists() {
            let bytes = std::fs::read(&path)
                .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
            let file: StoreFile = postcard::from_bytes(&bytes)
                .map_err(|error| format!("failed to parse {}: {error}", path.display()))?;

            if file.version != STORE_VERSION {
                return Err(format!(
                    "unsupported account store version {} (expected {STORE_VERSION})",
                    file.version
                ));
            }

            file
        } else {
            StoreFile::default()
        };

        Ok(Self {
            path,
            cipher,
            state: Mutex::new(state),
        })
    }

    fn encode(&self, username: &str, record: &AccountRecord) -> Result<Vec<u8>, String> {
        let bytes = postcard::to_allocvec(record)
            .map_err(|error| format!("failed to serialize account: {error}"))?;

        match &self.cipher {
            Some(cipher) => cipher.encrypt(&aad(username), &bytes),
            None => Ok(bytes),
        }
    }

    fn decode(&self, username: &str, data: &[u8]) -> Result<AccountRecord, String> {
        let bytes = match &self.cipher {
            Some(cipher) => cipher.decrypt(&aad(username), data)?,
            None => data.to_vec(),
        };

        postcard::from_bytes(&bytes)
            .map_err(|error| format!("failed to deserialize account: {error}"))
    }

    fn persist(&self, file: &StoreFile) -> Result<(), String> {
        let bytes = postcard::to_allocvec(file)
            .map_err(|error| format!("failed to serialize account store: {error}"))?;

        let temporary = self.path.with_extension("db.tmp");

        if write_sync(&temporary, &bytes).is_ok() {
            if std::fs::rename(&temporary, &self.path).is_ok() {
                return Ok(());
            }

            let _ = std::fs::remove_file(&temporary);
        }

        write_sync(&self.path, &bytes)
    }
}

fn write_sync(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = std::fs::File::create(path)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
    file.write_all(bytes)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
    file.sync_all()
        .map_err(|error| format!("failed to flush {}: {error}", path.display()))?;
    Ok(())
}

impl AccountRepo for FallbackStore {
    fn get(&self, username: &str) -> Result<Option<AccountRecord>, String> {
        let state = self
            .state
            .lock()
            .map_err(|_| "account store lock poisoned".to_string())?;

        match state.records.get(username) {
            Some(data) => self.decode(username, data).map(Some),
            None => Ok(None),
        }
    }

    fn create(&self, username: &str, record: &AccountRecord) -> Result<bool, String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "account store lock poisoned".to_string())?;

        if state.records.contains_key(username) {
            return Ok(false);
        }

        let value = self.encode(username, record)?;
        state.records.insert(username.to_string(), value);
        self.persist(&state)?;

        Ok(true)
    }

    fn upsert(&self, username: &str, record: &AccountRecord) -> Result<(), String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "account store lock poisoned".to_string())?;

        let value = self.encode(username, record)?;
        state.records.insert(username.to_string(), value);
        self.persist(&state)
    }

    fn all(&self) -> Result<Vec<(String, AccountRecord)>, String> {
        let state = self
            .state
            .lock()
            .map_err(|_| "account store lock poisoned".to_string())?;

        let mut accounts = Vec::with_capacity(state.records.len());
        for (username, data) in &state.records {
            accounts.push((username.clone(), self.decode(username, data)?));
        }

        Ok(accounts)
    }
}
