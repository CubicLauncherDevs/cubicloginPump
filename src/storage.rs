use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::i18n::{MessageTexts, format_text};

pub const MIN_USERNAME_LENGTH: usize = 3;
pub const MAX_USERNAME_LENGTH: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsernameError {
    TooShort,
    TooLong,
    InvalidCharacters,
}

impl UsernameError {
    pub fn message(self, messages: &MessageTexts) -> String {
        match self {
            Self::TooShort => {
                let min = MIN_USERNAME_LENGTH.to_string();
                format_text(&messages.username_too_short, &[("min", &min)])
            }
            Self::TooLong => {
                let max = MAX_USERNAME_LENGTH.to_string();
                format_text(&messages.username_too_long, &[("max", &max)])
            }
            Self::InvalidCharacters => messages.username_invalid.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordError {
    TooShort { min: usize },
    TooLong { max: usize },
}

impl PasswordError {
    pub fn message(self, messages: &MessageTexts) -> String {
        match self {
            Self::TooShort { min } => {
                let min = min.to_string();
                format_text(&messages.password_too_short, &[("min", &min)])
            }
            Self::TooLong { max } => {
                let max = max.to_string();
                format_text(&messages.password_too_long, &[("max", &max)])
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountRecord {
    pub version: u8,
    pub salt: Vec<u8>,
    pub argon2_memory_kib: u32,
    pub argon2_iterations: u32,
    pub argon2_parallelism: u32,
    pub hash: Vec<u8>,
    pub created_at: u64,
    pub updated_at: u64,
    pub last_login: u64,
    pub failed_attempts: u32,
    pub locked_until: u64,
}

impl AccountRecord {
    pub const FORMAT_VERSION: u8 = 1;
}

pub fn normalize_username(username: &str) -> String {
    username.trim().to_lowercase()
}

pub fn validate_username(username: &str) -> Result<(), UsernameError> {
    if username.len() < MIN_USERNAME_LENGTH {
        return Err(UsernameError::TooShort);
    }

    if username.len() > MAX_USERNAME_LENGTH {
        return Err(UsernameError::TooLong);
    }

    if !username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(UsernameError::InvalidCharacters);
    }

    Ok(())
}

pub fn validate_password(
    password: &str,
    min_length: usize,
    max_length: usize,
) -> Result<(), PasswordError> {
    if password.len() < min_length {
        return Err(PasswordError::TooShort { min: min_length });
    }

    if password.len() > max_length {
        return Err(PasswordError::TooLong { max: max_length });
    }

    Ok(())
}

pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

pub fn is_locked(record: &AccountRecord, now: u64) -> bool {
    record.locked_until > now
}

pub fn register_success(record: &mut AccountRecord, now: u64) {
    record.failed_attempts = 0;
    record.locked_until = 0;
    record.last_login = now;
    record.updated_at = now;
}

pub fn register_failure(
    record: &mut AccountRecord,
    max_attempts: u32,
    lockout_seconds: u64,
    now: u64,
) {
    record.failed_attempts = record.failed_attempts.saturating_add(1);
    record.updated_at = now;

    if max_attempts > 0 && record.failed_attempts >= max_attempts {
        record.locked_until = now.saturating_add(lockout_seconds);
        record.failed_attempts = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_username() {
        assert_eq!(normalize_username("  Steve "), "steve");
    }

    #[test]
    fn rejects_invalid_usernames() {
        assert!(validate_username("ab").is_err());
        assert!(validate_username("this_name_is_way_too_long").is_err());
        assert!(validate_username("bad name").is_err());
        assert!(validate_username("bad-name").is_err());
        assert!(validate_username("Steve_01").is_ok());
    }

    #[test]
    fn validates_password_length() {
        assert!(validate_password("short", 8, 64).is_err());
        assert!(validate_password("long-enough", 8, 64).is_ok());
        assert!(validate_password(&"a".repeat(65), 8, 64).is_err());
    }

    fn record() -> AccountRecord {
        AccountRecord {
            version: AccountRecord::FORMAT_VERSION,
            salt: Vec::new(),
            argon2_memory_kib: 1,
            argon2_iterations: 1,
            argon2_parallelism: 1,
            hash: Vec::new(),
            created_at: 0,
            updated_at: 0,
            last_login: 0,
            failed_attempts: 0,
            locked_until: 0,
        }
    }

    #[test]
    fn lockout_after_max_attempts() {
        let mut record = record();
        for _ in 0..4 {
            register_failure(&mut record, 5, 300, 1000);
        }
        assert!(!is_locked(&record, 1000));
        assert_eq!(record.failed_attempts, 4);

        register_failure(&mut record, 5, 300, 1000);
        assert_eq!(record.failed_attempts, 0);
        assert!(is_locked(&record, 1000));
        assert!(is_locked(&record, 1299));
        assert!(!is_locked(&record, 1300));
    }

    #[test]
    fn success_clears_failures() {
        let mut record = record();
        register_failure(&mut record, 5, 300, 1000);
        register_success(&mut record, 2000);
        assert_eq!(record.failed_attempts, 0);
        assert_eq!(record.locked_until, 0);
        assert_eq!(record.last_login, 2000);
        assert!(!is_locked(&record, 2000));
    }
}
