mod login;
mod register;

use pumpkin_plugin_api::Context;

use crate::state::SharedState;

pub fn register(context: &Context, state: SharedState) -> Result<(), String> {
    login::register(context, state.clone())?;
    register::register_command(context, state)?;
    Ok(())
}
