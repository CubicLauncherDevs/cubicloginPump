use argon2::{Algorithm, Argon2, Params, Version};
use subtle::ConstantTimeEq;
use zeroize::Zeroize;

use crate::storage::AccountRecord;

pub const SALT_LENGTH: usize = 16;
pub const HASH_LENGTH: usize = 32;

#[derive(Debug, Clone, Copy)]
pub struct Argon2Config {
    pub memory_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
}

fn params(config: &Argon2Config, output_len: usize) -> Result<Params, String> {
    Params::new(
        config.memory_kib,
        config.iterations,
        config.parallelism,
        Some(output_len),
    )
    .map_err(|error| format!("invalid argon2 parameters: {error}"))
}

fn argon2(pepper: Option<&[u8]>, params: Params) -> Result<Argon2<'_>, String> {
    match pepper {
        Some(pepper) if !pepper.is_empty() => {
            Argon2::new_with_secret(pepper, Algorithm::Argon2id, Version::V0x13, params)
                .map_err(|error| format!("invalid argon2 pepper: {error}"))
        }
        _ => Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params)),
    }
}

pub fn hash_password(
    password: &str,
    pepper: Option<&[u8]>,
    config: &Argon2Config,
) -> Result<(Vec<u8>, Vec<u8>), String> {
    let mut salt = vec![0u8; SALT_LENGTH];
    getrandom::fill(&mut salt).map_err(|error| format!("failed to gather entropy: {error}"))?;

    let argon = argon2(pepper, params(config, HASH_LENGTH)?)?;
    let mut hash = vec![0u8; HASH_LENGTH];
    argon
        .hash_password_into(password.as_bytes(), &salt, &mut hash)
        .map_err(|error| format!("failed to hash password: {error}"))?;

    Ok((salt, hash))
}

pub fn verify_password(
    password: &str,
    pepper: Option<&[u8]>,
    record: &AccountRecord,
) -> Result<bool, String> {
    let config = Argon2Config {
        memory_kib: record.argon2_memory_kib,
        iterations: record.argon2_iterations,
        parallelism: record.argon2_parallelism,
    };

    let argon = argon2(pepper, params(&config, record.hash.len())?)?;
    let mut hash = vec![0u8; record.hash.len()];
    argon
        .hash_password_into(password.as_bytes(), &record.salt, &mut hash)
        .map_err(|error| format!("failed to hash password: {error}"))?;

    let matches = hash.ct_eq(record.hash.as_slice()).into();
    hash.zeroize();

    Ok(matches)
}

pub fn dummy_record(config: &Argon2Config) -> AccountRecord {
    AccountRecord {
        version: AccountRecord::FORMAT_VERSION,
        salt: vec![0u8; SALT_LENGTH],
        argon2_memory_kib: config.memory_kib,
        argon2_iterations: config.iterations,
        argon2_parallelism: config.parallelism,
        hash: vec![0u8; HASH_LENGTH],
        created_at: 0,
        updated_at: 0,
        last_login: 0,
        failed_attempts: 0,
        locked_until: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fast_config() -> Argon2Config {
        Argon2Config {
            memory_kib: 64,
            iterations: 1,
            parallelism: 1,
        }
    }

    #[test]
    fn hashes_and_verifies() {
        let config = fast_config();
        let (salt, hash) = hash_password("correct horse", None, &config).unwrap();
        let record = AccountRecord {
            version: AccountRecord::FORMAT_VERSION,
            salt,
            argon2_memory_kib: config.memory_kib,
            argon2_iterations: config.iterations,
            argon2_parallelism: config.parallelism,
            hash,
            created_at: 0,
            updated_at: 0,
            last_login: 0,
            failed_attempts: 0,
            locked_until: 0,
        };

        assert!(verify_password("correct horse", None, &record).unwrap());
        assert!(!verify_password("wrong", None, &record).unwrap());
    }

    #[test]
    fn salts_are_unique() {
        let config = fast_config();
        let (first, _) = hash_password("same", None, &config).unwrap();
        let (second, _) = hash_password("same", None, &config).unwrap();
        assert_ne!(first, second);
    }

    #[test]
    fn pepper_changes_result() {
        let config = fast_config();
        let (salt, hash) = hash_password("secret", Some(b"pepper"), &config).unwrap();
        let mut record = dummy_record(&config);
        record.salt = salt;
        record.hash = hash;

        assert!(verify_password("secret", Some(b"pepper"), &record).unwrap());
        assert!(!verify_password("secret", None, &record).unwrap());
    }
}
