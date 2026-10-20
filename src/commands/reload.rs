use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use pumpkin_plugin_api::command::{
    Command, CommandError, CommandNode, CommandSender, ConsumedArgs,
};
use pumpkin_plugin_api::commands::CommandHandler;
use pumpkin_plugin_api::{Context, Server};

use crate::config::{self, DEFAULT_PREFIX, PluginConfig};
use crate::db;
use crate::i18n::{self, MessageTexts, format_text, prefixed};
use crate::permissions;
use crate::state::SharedState;
use crate::storage::{normalize_username, validate_username};

pub fn register(context: &Context, state: SharedState) -> Result<(), String> {
    let command = Command::new(&["cubiclogin".to_string()], "Manage CubicLogin")
        .execute(UsageCommand {
            state: Arc::clone(&state),
        })
        .then(CommandNode::literal("reload").execute(ReloadCommand { state }));

    context.register_command(command, permissions::ADMIN);
    Ok(())
}

struct Snapshot {
    data_dir: PathBuf,
    config: PluginConfig,
    messages: Arc<MessageTexts>,
    prefix: String,
    authenticated: HashSet<String>,
}

struct UsageCommand {
    state: SharedState,
}

impl CommandHandler for UsageCommand {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let (messages, prefix) = match self.state.read() {
            Ok(state) => (Arc::clone(&state.messages), state.config.prefix.clone()),
            Err(_) => return Err(state_unavailable()),
        };

        sender.send_error(prefixed(&prefix, &messages.reload_usage));
        Ok(0)
    }
}

struct ReloadCommand {
    state: SharedState,
}

impl CommandHandler for ReloadCommand {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let snapshot = match self.state.read() {
            Ok(state) => Snapshot {
                data_dir: state.data_dir.clone(),
                config: state.config.clone(),
                messages: Arc::clone(&state.messages),
                prefix: state.config.prefix.clone(),
                authenticated: state.authenticated.clone(),
            },
            Err(_) => return Err(state_unavailable()),
        };

        let loaded = match config::load_or_create(&snapshot.data_dir) {
            Ok(loaded) => loaded,
            Err(error) => {
                tracing::error!(error = %error, "failed to reload configuration");
                send_error_message(&sender, &snapshot.messages, &snapshot.prefix, &error);
                return Ok(0);
            }
        };

        let restart_fields = restart_required_fields(&snapshot.config, &loaded.config);
        if !restart_fields.is_empty() {
            let fields = restart_fields.join(", ");
            let text = format_text(
                &snapshot.messages.reload_restart_required,
                &[("fields", &fields)],
            );
            sender.send_error(prefixed(&snapshot.prefix, &text));
            tracing::warn!(fields = %fields, "reload requires a server restart");
            return Ok(0);
        }

        let messages = match i18n::load(&snapshot.data_dir, &loaded.config.language) {
            Ok(messages) => messages,
            Err(error) => {
                tracing::error!(error = %error, "failed to reload language messages");
                send_error_message(&sender, &snapshot.messages, &snapshot.prefix, &error);
                return Ok(0);
            }
        };

        let pepper = db::resolve_pepper(&loaded.config);
        let config_path = loaded.path;
        let language = loaded.config.language.clone();
        let require_login = loaded.config.require_login_on_join;
        let prefix = loaded.config.prefix.clone();
        let success = messages.reload_success.clone();

        {
            let mut state = self
                .state
                .write()
                .map_err(|_| CommandError::CommandFailed(state_unavailable_text()))?;

            state.config = loaded.config;
            state.messages = Arc::new(messages);
            state.pepper = pepper;
        }

        for player in server.get_all_players() {
            let username = normalize_username(&player.get_name());
            let should_lock = require_login
                && validate_username(&username).is_ok()
                && !snapshot.authenticated.contains(&username);
            player.set_movement_locked(should_lock);
        }

        sender.send_message(prefixed(&prefix, &success));
        tracing::info!(
            config = %config_path.display(),
            language = %language,
            require_login,
            "reloaded configuration and language messages"
        );

        Ok(1)
    }
}

fn restart_required_fields(old: &PluginConfig, new: &PluginConfig) -> Vec<&'static str> {
    let mut fields = Vec::new();

    if old.encryption != new.encryption {
        fields.push("encryption");
    }
    if old.db_key_env != new.db_key_env {
        fields.push("db_key_env");
    }
    if old.pepper_env != new.pepper_env {
        fields.push("pepper_env");
    }

    fields
}

fn send_error_message(sender: &CommandSender, messages: &MessageTexts, prefix: &str, error: &str) {
    let text = format_text(&messages.reload_failed, &[("error", error)]);
    sender.send_error(prefixed(prefix, &text));
}

fn ui_fallback() -> (Arc<MessageTexts>, String) {
    (
        Arc::new(MessageTexts::default()),
        DEFAULT_PREFIX.to_string(),
    )
}

fn state_unavailable_text() -> pumpkin_plugin_api::text::TextComponent {
    let (messages, prefix) = ui_fallback();
    prefixed(&prefix, &messages.state_unavailable)
}

fn state_unavailable() -> CommandError {
    CommandError::CommandFailed(state_unavailable_text())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::EncryptionMode;

    #[test]
    fn runtime_changes_do_not_require_restart() {
        let old = PluginConfig::default();
        let mut new = old.clone();
        new.language = "es".to_string();
        new.prefix = "&c[Test]&r ".to_string();
        new.require_login_on_join = false;
        new.min_password_length = 10;
        new.max_password_length = 80;
        new.max_failed_attempts = 3;
        new.lockout_seconds = 60;
        new.argon2_memory_kib = 4096;
        new.argon2_iterations = 3;
        new.argon2_parallelism = 2;

        assert!(restart_required_fields(&old, &new).is_empty());
    }

    #[test]
    fn database_changes_require_restart() {
        let old = PluginConfig::default();
        let mut new = old.clone();
        new.encryption = EncryptionMode::Off;
        new.db_key_env = "OTHER_DB_KEY".to_string();
        new.pepper_env = "OTHER_PEPPER".to_string();

        assert_eq!(
            restart_required_fields(&old, &new),
            vec!["encryption", "db_key_env", "pepper_env"]
        );
    }
}
