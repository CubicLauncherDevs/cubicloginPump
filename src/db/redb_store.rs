use std::path::Path;

use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};

use super::crypto::Cipher;
use super::{AccountRepo, aad};
use crate::storage::AccountRecord;

const ACCOUNTS: TableDefinition<&str, &[u8]> = TableDefinition::new("accounts");
const DATABASE_FILE: &str = "accounts.redb";

pub struct RedbStore {
    db: Database,
    cipher: Option<Cipher>,
}

impl RedbStore {
    pub fn open(data_dir: &Path, cipher: Option<Cipher>) -> Result<Self, String> {
        std::fs::create_dir_all(data_dir)
            .map_err(|error| format!("failed to create {}: {error}", data_dir.display()))?;

        let path = data_dir.join(DATABASE_FILE);
        let db = Database::create(&path)
            .map_err(|error| format!("failed to open redb database: {error}"))?;

        let transaction = db
            .begin_write()
            .map_err(|error| format!("failed to start redb transaction: {error}"))?;
        {
            transaction
                .open_table(ACCOUNTS)
                .map_err(|error| format!("failed to create redb table: {error}"))?;
        }
        transaction
            .commit()
            .map_err(|error| format!("failed to commit redb table: {error}"))?;

        Ok(Self { db, cipher })
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
}

impl AccountRepo for RedbStore {
    fn get(&self, username: &str) -> Result<Option<AccountRecord>, String> {
        let transaction = self
            .db
            .begin_read()
            .map_err(|error| format!("failed to start redb transaction: {error}"))?;
        let table = transaction
            .open_table(ACCOUNTS)
            .map_err(|error| format!("failed to open redb table: {error}"))?;

        match table
            .get(username)
            .map_err(|error| format!("failed to read account: {error}"))?
        {
            Some(guard) => self.decode(username, guard.value()).map(Some),
            None => Ok(None),
        }
    }

    fn create(&self, username: &str, record: &AccountRecord) -> Result<bool, String> {
        let transaction = self
            .db
            .begin_write()
            .map_err(|error| format!("failed to start redb transaction: {error}"))?;

        let created = {
            let mut table = transaction
                .open_table(ACCOUNTS)
                .map_err(|error| format!("failed to open redb table: {error}"))?;

            let exists = table
                .get(username)
                .map_err(|error| format!("failed to read account: {error}"))?
                .is_some();

            if !exists {
                let value = self.encode(username, record)?;
                table
                    .insert(username, value.as_slice())
                    .map_err(|error| format!("failed to write account: {error}"))?;
            }

            !exists
        };

        transaction
            .commit()
            .map_err(|error| format!("failed to commit account: {error}"))?;

        Ok(created)
    }

    fn upsert(&self, username: &str, record: &AccountRecord) -> Result<(), String> {
        let value = self.encode(username, record)?;

        let transaction = self
            .db
            .begin_write()
            .map_err(|error| format!("failed to start redb transaction: {error}"))?;
        {
            let mut table = transaction
                .open_table(ACCOUNTS)
                .map_err(|error| format!("failed to open redb table: {error}"))?;
            table
                .insert(username, value.as_slice())
                .map_err(|error| format!("failed to write account: {error}"))?;
        }
        transaction
            .commit()
            .map_err(|error| format!("failed to commit account: {error}"))?;

        Ok(())
    }

    fn all(&self) -> Result<Vec<(String, AccountRecord)>, String> {
        let transaction = self
            .db
            .begin_read()
            .map_err(|error| format!("failed to start redb transaction: {error}"))?;
        let table = transaction
            .open_table(ACCOUNTS)
            .map_err(|error| format!("failed to open redb table: {error}"))?;

        let mut accounts = Vec::new();
        for entry in table
            .iter()
            .map_err(|error| format!("failed to list accounts: {error}"))?
        {
            let (key, value) =
                entry.map_err(|error| format!("failed to list accounts: {error}"))?;
            let username = key.value().to_string();
            accounts.push((username.clone(), self.decode(&username, value.value())?));
        }

        Ok(accounts)
    }
}
