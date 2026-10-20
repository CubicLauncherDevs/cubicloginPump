use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const DEFAULT_LANGUAGE: &str = "en";
pub const DEFAULT_PREFIX: &str = "&b[CubicLogin]&r ";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EncryptionMode {
    Required,
    Auto,
    Off,
}

impl EncryptionMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Required => "required",
            Self::Auto => "auto",
            Self::Off => "off",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PluginConfig {
    pub language: String,
    pub prefix: String,
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
            language: DEFAULT_LANGUAGE.to_string(),
            prefix: DEFAULT_PREFIX.to_string(),
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

    let path = data_dir.join("config.toml");
    let legacy_path = data_dir.join("config.json");

    if !path.exists() {
        if legacy_path.exists() {
            return migrate_json(data_dir, &legacy_path, &path);
        }

        let config = PluginConfig::default();
        write_config(&path, &config)?;
        return Ok(LoadedConfig { config, path });
    }

    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let config: PluginConfig = toml::from_str(&raw)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))?;

    ensure_all_options(&path, &raw)?;

    Ok(LoadedConfig { config, path })
}

fn migrate_json(data_dir: &Path, legacy_path: &Path, path: &Path) -> Result<LoadedConfig, String> {
    let raw = fs::read_to_string(legacy_path)
        .map_err(|error| format!("failed to read {}: {error}", legacy_path.display()))?;
    let config: PluginConfig = serde_json::from_str(&raw)
        .map_err(|error| format!("failed to parse {}: {error}", legacy_path.display()))?;

    write_config(path, &config)?;

    let backup = data_dir.join("config.json.bak");
    fs::rename(legacy_path, &backup).map_err(|error| {
        format!(
            "failed to rename {} to {}: {error}",
            legacy_path.display(),
            backup.display()
        )
    })?;

    tracing::info!(
        from = %legacy_path.display(),
        to = %path.display(),
        backup = %backup.display(),
        "migrated configuration to TOML"
    );

    Ok(LoadedConfig {
        config,
        path: path.to_path_buf(),
    })
}

fn write_config(path: &Path, config: &PluginConfig) -> Result<(), String> {
    fs::write(path, render_config(config))
        .map_err(|error| format!("failed to write {}: {error}", path.display()))
}

fn render_config(config: &PluginConfig) -> String {
    let language = toml_text(&config.language);
    let prefix = toml_text(&config.prefix);
    let encryption = toml_text(config.encryption.as_str());
    let db_key_env = toml_text(&config.db_key_env);
    let pepper_env = toml_text(&config.pepper_env);

    format!(
        r#"# ============================================================
#  CubicLogin configuration
# ============================================================
#
#  Reference: docs/configuration.md
#  Language files: ./lang/<code>.toml
#  Colors: use '&' followed by a legacy color code (e.g. &b, &l, &r).

# ------------------------------------------------------------
#  Language and messages
# ------------------------------------------------------------
# Built-in languages: "en", "es". For a custom language, create
# ./lang/<code>.toml and set language to that code.
language = {language}

# Prefix added to every player-facing message.
prefix = {prefix}

# ------------------------------------------------------------
#  Authentication
# ------------------------------------------------------------
# Require players to log in before playing.
require_login_on_join = {require_login_on_join}

# Password length limits.
min_password_length = {min_password_length}
max_password_length = {max_password_length}

# Reserved for future session handling (currently unused).
session_timeout_seconds = {session_timeout_seconds}

# Lockout policy after repeated failed login attempts.
# Set max_failed_attempts to 0 to disable lockouts.
max_failed_attempts = {max_failed_attempts}
lockout_seconds = {lockout_seconds}

# ------------------------------------------------------------
#  Password hashing (Argon2id)
# ------------------------------------------------------------
argon2_memory_kib = {argon2_memory_kib}
argon2_iterations = {argon2_iterations}
argon2_parallelism = {argon2_parallelism}

# ------------------------------------------------------------
#  Database
# ------------------------------------------------------------
# Encryption mode: "required", "auto", or "off".
#   required - a valid key must be set in db_key_env
#   auto     - use the key when present, otherwise store plaintext
#   off      - never encrypt (not recommended)
encryption = {encryption}

# Environment variable containing the 32-byte (64 hex character) database key.
db_key_env = {db_key_env}

# Optional environment variable with an Argon2 pepper.
pepper_env = {pepper_env}
"#,
        require_login_on_join = config.require_login_on_join,
        min_password_length = config.min_password_length,
        max_password_length = config.max_password_length,
        session_timeout_seconds = config.session_timeout_seconds,
        max_failed_attempts = config.max_failed_attempts,
        lockout_seconds = config.lockout_seconds,
        argon2_memory_kib = config.argon2_memory_kib,
        argon2_iterations = config.argon2_iterations,
        argon2_parallelism = config.argon2_parallelism,
    )
}

fn toml_text(value: &str) -> String {
    toml_edit::Value::from(value.to_string()).to_string()
}

const FIELD_COMMENTS: &[(&str, &str)] = &[
    (
        "language",
        "Language used for player-facing messages. Built-in: \"en\", \"es\".",
    ),
    (
        "prefix",
        "Prefix added to every player-facing message. Supports '&' color codes.",
    ),
    (
        "require_login_on_join",
        "Require players to authenticate before playing.",
    ),
    ("min_password_length", "Minimum password length."),
    ("max_password_length", "Maximum password length."),
    (
        "session_timeout_seconds",
        "Reserved for future session handling (currently unused).",
    ),
    (
        "max_failed_attempts",
        "Failed login attempts before a temporary lockout (0 disables).",
    ),
    ("lockout_seconds", "Lockout duration in seconds."),
    ("argon2_memory_kib", "Argon2id memory cost in KiB."),
    ("argon2_iterations", "Argon2id iterations."),
    ("argon2_parallelism", "Argon2id parallelism."),
    (
        "encryption",
        "Database encryption mode: \"required\", \"auto\", or \"off\".",
    ),
    (
        "db_key_env",
        "Environment variable with the 32-byte hex database key.",
    ),
    (
        "pepper_env",
        "Optional environment variable with an Argon2 pepper.",
    ),
];

fn field_comment(key: &str) -> Option<&'static str> {
    FIELD_COMMENTS
        .iter()
        .find(|(name, _)| *name == key)
        .map(|(_, comment)| *comment)
}

fn ensure_all_options(path: &Path, raw: &str) -> Result<(), String> {
    let mut document: toml_edit::DocumentMut = raw
        .parse()
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))?;
    let defaults = toml_edit::ser::to_document(&PluginConfig::default())
        .map_err(|error| format!("failed to inspect default config: {error}"))?;

    let mut changed = false;

    for (key, item) in defaults.as_table() {
        if !document.as_table().contains_key(key) {
            document.as_table_mut().insert(key, item.clone());

            if let Some(comment) = field_comment(key)
                && let Some(mut key_mut) = document.as_table_mut().key_mut(key)
            {
                key_mut
                    .leaf_decor_mut()
                    .set_prefix(format!("\n# {comment}\n"));
            }

            changed = true;
        }
    }

    if changed {
        fs::write(path, document.to_string())
            .map_err(|error| format!("failed to update {}: {error}", path.display()))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!(
            "cubiclogin-config-{tag}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn creates_toml_with_english_defaults() {
        let dir = temp_dir("default");
        let loaded = load_or_create(&dir).unwrap();

        assert_eq!(loaded.config.language, "en");
        assert_eq!(loaded.config.prefix, DEFAULT_PREFIX);
        assert!(dir.join("config.toml").exists());

        let raw = fs::read_to_string(dir.join("config.toml")).unwrap();
        assert!(raw.contains("language = \"en\""));
        assert!(raw.contains("prefix = "));
        assert!(raw.contains("#  CubicLogin configuration"));
        assert!(raw.contains("#  Authentication"));
        assert!(raw.contains("#  Password hashing (Argon2id)"));
        assert!(raw.contains("#  Database"));

        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn preserves_comments_when_adding_missing_options() {
        let dir = temp_dir("comments");
        fs::write(
            dir.join("config.toml"),
            "# custom header\nprefix = \"&c[MyServer]&r \"\n",
        )
        .unwrap();

        let loaded = load_or_create(&dir).unwrap();
        assert_eq!(loaded.config.prefix, "&c[MyServer]&r ");

        let raw = fs::read_to_string(dir.join("config.toml")).unwrap();
        assert!(raw.contains("# custom header"));
        assert!(raw.contains("prefix = \"&c[MyServer]&r \""));
        assert!(raw.contains("# Language used for player-facing messages"));

        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn migrates_json_config() {
        let dir = temp_dir("migrate");
        fs::write(
            dir.join("config.json"),
            r#"{"require_login_on_join":false,"min_password_length":10}"#,
        )
        .unwrap();

        let loaded = load_or_create(&dir).unwrap();

        assert!(!loaded.config.require_login_on_join);
        assert_eq!(loaded.config.min_password_length, 10);
        assert_eq!(loaded.config.language, "en");
        assert!(dir.join("config.toml").exists());
        assert!(dir.join("config.json.bak").exists());
        assert!(!dir.join("config.json").exists());

        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn fills_missing_options() {
        let dir = temp_dir("missing");
        fs::write(dir.join("config.toml"), "require_login_on_join = false\n").unwrap();

        let loaded = load_or_create(&dir).unwrap();
        assert!(!loaded.config.require_login_on_join);
        assert_eq!(loaded.config.language, "en");

        let raw = fs::read_to_string(dir.join("config.toml")).unwrap();
        assert!(raw.contains("language = \"en\""));
        assert!(raw.contains("require_login_on_join = false"));

        fs::remove_dir_all(dir).ok();
    }
}
