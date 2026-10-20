use std::fs;
use std::path::Path;

use pumpkin_plugin_api::text::TextComponent;
use serde::{Deserialize, Serialize};

const EN_SOURCE: &str = include_str!("lang/en.toml");
const ES_SOURCE: &str = include_str!("lang/es.toml");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageTexts {
    pub join_login: String,
    pub join_register: String,
    pub blocked: String,
    pub hover_login: String,
    pub hover_register: String,
    pub only_players_login: String,
    pub only_players_register: String,
    pub login_empty_password: String,
    pub login_locked: String,
    pub login_success: String,
    pub login_failed: String,
    pub register_mismatch: String,
    pub register_success: String,
    pub account_exists: String,
    pub account_save_error: String,
    pub username_too_short: String,
    pub username_too_long: String,
    pub username_invalid: String,
    pub password_too_short: String,
    pub password_too_long: String,
    pub storage_unavailable: String,
    pub database_error: String,
    pub password_error: String,
    pub state_unavailable: String,
}

impl Default for MessageTexts {
    fn default() -> Self {
        toml::from_str(EN_SOURCE).expect("embedded English language file must be valid")
    }
}

pub fn load(data_dir: &Path, language: &str) -> Result<MessageTexts, String> {
    let lang_dir = data_dir.join("lang");
    fs::create_dir_all(&lang_dir)
        .map_err(|error| format!("failed to create {}: {error}", lang_dir.display()))?;

    write_if_missing(&lang_dir.join("en.toml"), EN_SOURCE)?;
    write_if_missing(&lang_dir.join("es.toml"), ES_SOURCE)?;

    let code = normalize_code(language);
    let path = lang_dir.join(format!("{code}.toml"));

    let raw = match fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(error) => {
            tracing::warn!(
                language = %code,
                error = %error,
                "language file unavailable, using English"
            );
            EN_SOURCE.to_string()
        }
    };

    match parse_with_fallback(&raw) {
        Ok(messages) => {
            tracing::info!(language = %code, path = %path.display(), "loaded language messages");
            Ok(messages)
        }
        Err(error) => {
            tracing::error!(
                language = %code,
                error = %error,
                "invalid language file, using English"
            );
            parse_with_fallback(EN_SOURCE)
        }
    }
}

fn write_if_missing(path: &Path, contents: &str) -> Result<(), String> {
    if path.exists() {
        return Ok(());
    }

    fs::write(path, contents)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))
}

fn normalize_code(language: &str) -> String {
    let code = language.trim().to_ascii_lowercase();

    if code.is_empty()
        || !code
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        tracing::warn!(language = %language, "invalid language code, using English");
        return "en".to_string();
    }

    code
}

fn parse_with_fallback(raw: &str) -> Result<MessageTexts, String> {
    let mut base: toml::Table = toml::from_str(EN_SOURCE)
        .map_err(|error| format!("failed to parse embedded English language file: {error}"))?;
    let overrides: toml::Table =
        toml::from_str(raw).map_err(|error| format!("failed to parse language file: {error}"))?;

    base.extend(overrides);
    base.try_into()
        .map_err(|error: toml::de::Error| format!("failed to decode language file: {error}"))
}

pub fn format_text(template: &str, vars: &[(&str, &str)]) -> String {
    let mut text = template.to_string();

    for (key, value) in vars {
        text = text.replace(&format!("{{{key}}}"), value);
    }

    text
}

pub fn component(text: &str) -> TextComponent {
    TextComponent::from_legacy_string_with_code(text, '&')
}

pub fn prefixed(prefix: &str, text: &str) -> TextComponent {
    component(&format!("{prefix}{text}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_languages_share_keys() {
        let english: toml::Table = toml::from_str(EN_SOURCE).unwrap();
        let spanish: toml::Table = toml::from_str(ES_SOURCE).unwrap();

        let mut english_keys: Vec<_> = english.keys().cloned().collect();
        let mut spanish_keys: Vec<_> = spanish.keys().cloned().collect();
        english_keys.sort();
        spanish_keys.sort();

        assert_eq!(english_keys, spanish_keys);
    }

    #[test]
    fn missing_keys_fall_back_to_english() {
        let messages = parse_with_fallback("login_success = \"Ok\"").unwrap();

        assert_eq!(messages.login_success, "Ok");
        assert_eq!(messages.login_failed, MessageTexts::default().login_failed);
    }

    #[test]
    fn formats_placeholders() {
        assert_eq!(
            format_text("Try again in {seconds} seconds.", &[("seconds", "30")]),
            "Try again in 30 seconds."
        );
    }

    #[test]
    fn normalizes_language_codes() {
        assert_eq!(normalize_code(" ES "), "es");
        assert_eq!(normalize_code("../evil"), "en");
        assert_eq!(normalize_code(""), "en");
    }
}
