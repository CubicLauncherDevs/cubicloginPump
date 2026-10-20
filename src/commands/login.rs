use std::sync::Arc;

use pumpkin_plugin_api::command::{
    Arg, ArgumentType, Command, CommandError, CommandNode, CommandSender, ConsumedArgs, StringType,
};
use pumpkin_plugin_api::commands::CommandHandler;
use pumpkin_plugin_api::{Context, Server};
use zeroize::Zeroize;

use crate::config::DEFAULT_PREFIX;
use crate::db::hashing;
use crate::i18n::{MessageTexts, format_text, prefixed};
use crate::permissions;
use crate::state::SharedState;
use crate::storage::{
    is_locked, normalize_username, now_secs, register_failure, register_success, validate_username,
};

pub fn register(context: &Context, state: SharedState) -> Result<(), String> {
    let command = Command::new(
        &["login".to_string()],
        "Authenticate with your CubicLogin account",
    )
    .then(
        CommandNode::argument("password", &ArgumentType::String(StringType::SingleWord))
            .execute(LoginCommand { state }),
    );

    context.register_command(command, permissions::LOGIN);
    Ok(())
}

struct LoginCommand {
    state: SharedState,
}

impl CommandHandler for LoginCommand {
    fn handle(
        &self,
        sender: CommandSender,
        _server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let mut password = match args.get_value("password") {
            Arg::Simple(value) => value,
            _ => return Err(CommandError::InvalidConsumption(None)),
        };

        let Some(player) = sender.as_player() else {
            let (messages, prefix) = ui_fallback();
            sender.send_error(prefixed(&prefix, &messages.only_players_login));
            password.zeroize();
            return Ok(0);
        };

        let (store, config, pepper, messages, prefix) = {
            let state = match self.state.read() {
                Ok(state) => state,
                Err(_) => {
                    password.zeroize();
                    return Err(state_unavailable());
                }
            };

            (
                state.store.clone(),
                state.config.clone(),
                state.pepper.clone(),
                Arc::clone(&state.messages),
                state.config.prefix.clone(),
            )
        };

        let Some(store) = store else {
            sender.send_error(prefixed(&prefix, &messages.storage_unavailable));
            password.zeroize();
            return Ok(0);
        };

        let username = normalize_username(&player.get_name());

        if let Err(error) = validate_username(&username) {
            let text = error.message(&messages);
            sender.send_error(prefixed(&prefix, &text));
            password.zeroize();
            return Ok(0);
        }

        if password.is_empty() {
            sender.send_error(prefixed(&prefix, &messages.login_empty_password));
            return Ok(0);
        }

        let now = now_secs();
        let argon2_config = crate::db::argon2_config(&config);

        let record = match store.get(&username) {
            Ok(record) => record,
            Err(error) => {
                tracing::error!(error = %error, "failed to read account store");
                sender.send_error(prefixed(&prefix, &messages.database_error));
                password.zeroize();
                return Ok(0);
            }
        };

        if let Some(existing) = &record
            && is_locked(existing, now)
        {
            let remaining = (existing.locked_until - now).to_string();
            let text = format_text(&messages.login_locked, &[("seconds", &remaining)]);
            sender.send_error(prefixed(&prefix, &text));
            password.zeroize();
            return Ok(0);
        }

        let verified = match &record {
            Some(existing) => {
                match hashing::verify_password(&password, pepper.as_deref(), existing) {
                    Ok(matches) => matches,
                    Err(error) => {
                        tracing::error!(error = %error, "failed to verify password");
                        sender.send_error(prefixed(&prefix, &messages.password_error));
                        password.zeroize();
                        return Ok(0);
                    }
                }
            }
            None => {
                let dummy = hashing::dummy_record(&argon2_config);
                let _ = hashing::verify_password(&password, pepper.as_deref(), &dummy);
                false
            }
        };
        password.zeroize();

        if verified {
            let mut updated = record.expect("verified records always exist");
            register_success(&mut updated, now);

            if let Err(error) = store.upsert(&username, &updated) {
                tracing::error!(error = %error, "failed to update account after login");
            }

            if let Ok(mut state) = self.state.write() {
                state.mark_authenticated(&username);
            }
            player.set_movement_locked(false);

            tracing::info!(player = %username, "authenticated CubicLogin account");
            sender.send_message(prefixed(&prefix, &messages.login_success));
            return Ok(1);
        }

        if let Some(mut existing) = record {
            register_failure(
                &mut existing,
                config.max_failed_attempts,
                config.lockout_seconds,
                now,
            );

            if let Err(error) = store.upsert(&username, &existing) {
                tracing::error!(error = %error, "failed to update failed login attempts");
            }
        }

        sender.send_error(prefixed(&prefix, &messages.login_failed));
        Ok(0)
    }
}

fn ui_fallback() -> (Arc<MessageTexts>, String) {
    (
        Arc::new(MessageTexts::default()),
        DEFAULT_PREFIX.to_string(),
    )
}

fn state_unavailable() -> CommandError {
    let (messages, prefix) = ui_fallback();
    CommandError::CommandFailed(prefixed(&prefix, &messages.state_unavailable))
}
