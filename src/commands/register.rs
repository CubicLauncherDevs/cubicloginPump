use std::sync::Arc;

use pumpkin_plugin_api::command::{
    Arg, ArgumentType, Command, CommandError, CommandNode, CommandSender, ConsumedArgs, StringType,
};
use pumpkin_plugin_api::commands::CommandHandler;
use pumpkin_plugin_api::{Context, Server};
use zeroize::Zeroize;

use crate::config::DEFAULT_PREFIX;
use crate::db::hashing;
use crate::i18n::{MessageTexts, prefixed};
use crate::permissions;
use crate::state::SharedState;
use crate::storage::{
    AccountRecord, normalize_username, now_secs, validate_password, validate_username,
};

pub fn register_command(context: &Context, state: SharedState) -> Result<(), String> {
    let command = Command::new(&["register".to_string()], "Create a CubicLogin account").then(
        CommandNode::argument("password", &ArgumentType::String(StringType::SingleWord)).then(
            CommandNode::argument(
                "confirm_password",
                &ArgumentType::String(StringType::SingleWord),
            )
            .execute(RegisterCommand { state }),
        ),
    );

    context.register_command(command, permissions::REGISTER);
    Ok(())
}

struct RegisterCommand {
    state: SharedState,
}

impl CommandHandler for RegisterCommand {
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

        let mut confirm_password = match args.get_value("confirm_password") {
            Arg::Simple(value) => value,
            _ => {
                password.zeroize();
                return Err(CommandError::InvalidConsumption(None));
            }
        };

        let Some(player) = sender.as_player() else {
            let (messages, prefix) = ui_fallback();
            sender.send_error(prefixed(&prefix, &messages.only_players_register));
            password.zeroize();
            confirm_password.zeroize();
            return Ok(0);
        };

        let (store, config, pepper, messages, prefix) = {
            let state = match self.state.read() {
                Ok(state) => state,
                Err(_) => {
                    password.zeroize();
                    confirm_password.zeroize();
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
            confirm_password.zeroize();
            return Ok(0);
        };

        let username = normalize_username(&player.get_name());

        if let Err(error) = validate_username(&username) {
            let text = error.message(&messages);
            sender.send_error(prefixed(&prefix, &text));
            password.zeroize();
            confirm_password.zeroize();
            return Ok(0);
        }

        if password != confirm_password {
            sender.send_error(prefixed(&prefix, &messages.register_mismatch));
            password.zeroize();
            confirm_password.zeroize();
            return Ok(0);
        }
        confirm_password.zeroize();

        if let Err(error) = validate_password(
            &password,
            config.min_password_length,
            config.max_password_length,
        ) {
            let text = error.message(&messages);
            sender.send_error(prefixed(&prefix, &text));
            password.zeroize();
            return Ok(0);
        }

        match store.get(&username) {
            Ok(Some(_)) => {
                sender.send_error(prefixed(&prefix, &messages.account_exists));
                password.zeroize();
                return Ok(0);
            }
            Ok(None) => {}
            Err(error) => {
                tracing::error!(error = %error, "failed to read account store");
                sender.send_error(prefixed(&prefix, &messages.database_error));
                password.zeroize();
                return Ok(0);
            }
        }

        let argon2_config = crate::db::argon2_config(&config);
        let (salt, hash) =
            match hashing::hash_password(&password, pepper.as_deref(), &argon2_config) {
                Ok(result) => result,
                Err(error) => {
                    tracing::error!(error = %error, "failed to hash password");
                    sender.send_error(prefixed(&prefix, &messages.password_error));
                    password.zeroize();
                    return Ok(0);
                }
            };
        password.zeroize();

        let now = now_secs();
        let record = AccountRecord {
            version: AccountRecord::FORMAT_VERSION,
            salt,
            argon2_memory_kib: argon2_config.memory_kib,
            argon2_iterations: argon2_config.iterations,
            argon2_parallelism: argon2_config.parallelism,
            hash,
            created_at: now,
            updated_at: now,
            last_login: 0,
            failed_attempts: 0,
            locked_until: 0,
        };

        match store.create(&username, &record) {
            Ok(true) => {
                if let Ok(mut state) = self.state.write() {
                    state.mark_authenticated(&username);
                }
                player.set_movement_locked(false);

                tracing::info!(player = %username, "registered and authenticated CubicLogin account");
                sender.send_message(prefixed(&prefix, &messages.register_success));
                Ok(1)
            }
            Ok(false) => {
                sender.send_error(prefixed(&prefix, &messages.account_exists));
                Ok(0)
            }
            Err(error) => {
                tracing::error!(error = %error, "failed to persist account");
                sender.send_error(prefixed(&prefix, &messages.account_save_error));
                Ok(0)
            }
        }
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
