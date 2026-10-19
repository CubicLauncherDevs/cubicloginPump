use pumpkin_plugin_api::command::{
    Arg, ArgumentType, Command, CommandError, CommandNode, CommandSender, ConsumedArgs, StringType,
};
use pumpkin_plugin_api::commands::CommandHandler;
use pumpkin_plugin_api::text::TextComponent;
use pumpkin_plugin_api::{Context, Server};

use crate::permissions;
use crate::state::SharedState;

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
        let password = match args.get_value("password") {
            Arg::Simple(value) => value,
            _ => return Err(CommandError::InvalidConsumption(None)),
        };

        let Some(player) = sender.as_player() else {
            sender.send_error(TextComponent::text("Only players can use /login."));
            return Ok(0);
        };

        let username = player.get_name();
        let registered = self
            .state
            .read()
            .map_err(|_| state_unavailable())?
            .accounts
            .get(&username)
            .is_some();

        if password.is_empty() {
            sender.send_error(TextComponent::text("The password cannot be empty."));
            return Ok(0);
        }

        if !registered {
            sender.send_error(TextComponent::text(
                "No CubicLogin account was found. Use /register first.",
            ));
            return Ok(0);
        }

        sender.send_message(TextComponent::text(
            "Login received. Credential verification is not implemented yet.",
        ));
        Ok(1)
    }
}

fn state_unavailable() -> CommandError {
    CommandError::CommandFailed(TextComponent::text("CubicLogin state is unavailable."))
}
