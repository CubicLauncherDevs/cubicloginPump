use pumpkin_plugin_api::command::{
    Arg, ArgumentType, Command, CommandError, CommandNode, CommandSender, ConsumedArgs, StringType,
};
use pumpkin_plugin_api::commands::CommandHandler;
use pumpkin_plugin_api::text::TextComponent;
use pumpkin_plugin_api::{Context, Server};
use zeroize::Zeroize;

use crate::db::hashing;
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
            sender.send_error(TextComponent::text("Only players can use /login."));
            password.zeroize();
            return Ok(0);
        };

        let username = normalize_username(&player.get_name());

        if let Err(reason) = validate_username(&username) {
            sender.send_error(TextComponent::text(&reason));
            password.zeroize();
            return Ok(0);
        }

        if password.is_empty() {
            sender.send_error(TextComponent::text("The password cannot be empty."));
            return Ok(0);
        }

        let (store, config, pepper) = {
            let state = self.state.read().map_err(|_| state_unavailable())?;
            let Some(store) = state.store.clone() else {
                drop(state);
                sender.send_error(TextComponent::text("CubicLogin storage is unavailable."));
                password.zeroize();
                return Ok(0);
            };

            (store, state.config.clone(), state.pepper.clone())
        };

        let now = now_secs();
        let argon2_config = crate::db::argon2_config(&config);

        let record = match store.get(&username) {
            Ok(record) => record,
            Err(error) => {
                tracing::error!(error = %error, "failed to read account store");
                sender.send_error(TextComponent::text(
                    "Could not access the account database.",
                ));
                password.zeroize();
                return Ok(0);
            }
        };

        if let Some(existing) = &record
            && is_locked(existing, now)
        {
            let remaining = existing.locked_until - now;
            sender.send_error(TextComponent::text(&format!(
                "Too many failed attempts. Try again in {remaining} seconds."
            )));
            password.zeroize();
            return Ok(0);
        }

        let verified = match &record {
            Some(existing) => {
                match hashing::verify_password(&password, pepper.as_deref(), existing) {
                    Ok(matches) => matches,
                    Err(error) => {
                        tracing::error!(error = %error, "failed to verify password");
                        sender.send_error(TextComponent::text("Could not process the password."));
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

            tracing::info!(player = %username, "authenticated CubicLogin account");
            sender.send_message(TextComponent::text("Login successful."));
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

        sender.send_error(TextComponent::text("Incorrect username or password."));
        Ok(0)
    }
}

fn state_unavailable() -> CommandError {
    CommandError::CommandFailed(TextComponent::text("CubicLogin state is unavailable."))
}
