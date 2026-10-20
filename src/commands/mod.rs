mod login;
mod register;
mod reload;

use pumpkin_plugin_api::Context;

use crate::state::SharedState;

pub fn register(context: &Context, state: SharedState) -> Result<(), String> {
    login::register(context, state.clone())?;
    register::register_command(context, state.clone())?;
    reload::register(context, state)?;
    Ok(())
}
