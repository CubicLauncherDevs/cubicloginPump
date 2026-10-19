use pumpkin_plugin_api::command::{
    Arg, ArgumentType, Command, CommandError, CommandNode, CommandSender, ConsumedArgs, StringType,
};
use pumpkin_plugin_api::commands::CommandHandler;
use pumpkin_plugin_api::text::TextComponent;
use pumpkin_plugin_api::{Context, Server};

use crate::permissions;
use crate::state::SharedState;

pub fn register_command(context: &Context, state: SharedState) -> Result<(), String> {
    let command = Command::new(&["register".to_string()], "Create a CubicLogin account").then(
        CommandNode::argument("password", &ArgumentType::String(StringType::SingleWord))
            .execute(RegisterCommand { state }),
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
        let password = match args.get_value("password") {
            Arg::Simple(value) => value,
            _ => return Err(CommandError::InvalidConsumption(None)),
        };

        let Some(player) = sender.as_player() else {
            sender.send_error(TextComponent::text("Only players can use /register."));
            return Ok(0);
        };

        let username = player.get_name();
        let (min_length, max_length, exists) = {
            let state = self.state.read().map_err(|_| state_unavailable())?;
            (
                state.config.min_password_length,
                state.config.max_password_length,
                state.accounts.contains(&username),
            )
        };

        if password.len() < min_length {
            sender.send_error(TextComponent::text(&format!(
                "The password must contain at least {min_length} characters."
            )));
            return Ok(0);
        }

        if password.len() > max_length {
            sender.send_error(TextComponent::text(&format!(
                "The password must contain at most {max_length} characters."
            )));
            return Ok(0);
        }

        if exists {
            sender.send_error(TextComponent::text(
                "A CubicLogin account already exists for this player.",
            ));
            return Ok(0);
        }

        sender.send_message(TextComponent::text(
            "Registration received. Account persistence is not implemented yet.",
        ));
        Ok(1)
    }
}

fn state_unavailable() -> CommandError {
    CommandError::CommandFailed(TextComponent::text("CubicLogin state is unavailable."))
}
