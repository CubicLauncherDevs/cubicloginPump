mod player;

use pumpkin_plugin_api::Context;
use pumpkin_plugin_api::events::EventPriority;

use crate::state::SharedState;

pub fn register(context: &Context, state: SharedState) -> Result<(), String> {
    context.register_event_handler(
        player::PlayerJoinHandler::new(state.clone()),
        EventPriority::Normal,
        false,
    )?;
    context.register_event_handler(
        player::PlayerLeaveHandler::new(state),
        EventPriority::Normal,
        false,
    )?;
    Ok(())
}
