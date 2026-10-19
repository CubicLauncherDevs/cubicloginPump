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
        player::PlayerLeaveHandler::new(state.clone()),
        EventPriority::Normal,
        false,
    )?;

    macro_rules! blocking {
        ($handler:expr) => {
            context.register_event_handler($handler, EventPriority::Normal, true)?;
        };
    }

    blocking!(player::BlockBreakHandler::new(state.clone()));
    blocking!(player::BlockPlaceHandler::new(state.clone()));
    blocking!(player::CraftItemHandler::new(state.clone()));
    blocking!(player::InventoryClickHandler::new(state.clone()));
    blocking!(player::InventoryDragHandler::new(state.clone()));
    blocking!(player::InventoryOpenHandler::new(state.clone()));
    blocking!(player::PlayerArmorStandManipulateHandler::new(
        state.clone()
    ));
    blocking!(player::PlayerBedEnterHandler::new(state.clone()));
    blocking!(player::PlayerBucketEntityHandler::new(state.clone()));
    blocking!(player::PlayerChangeWorldHandler::new(state.clone()));
    blocking!(player::PlayerChatHandler::new(state.clone()));
    blocking!(player::PlayerCommandHandler::new(state.clone()));
    blocking!(player::PlayerDropItemHandler::new(state.clone()));
    blocking!(player::PlayerEditBookHandler::new(state.clone()));
    blocking!(player::PlayerEggThrowHandler::new(state.clone()));
    blocking!(player::PlayerElytraBoostHandler::new(state.clone()));
    blocking!(player::PlayerFishHandler::new(state.clone()));
    blocking!(player::PlayerHarvestBlockHandler::new(state.clone()));
    blocking!(player::PlayerInteractAtEntityHandler::new(state.clone()));
    blocking!(player::PlayerInteractEntityHandler::new(state.clone()));
    blocking!(player::PlayerInteractHandler::new(state.clone()));
    blocking!(player::PlayerItemConsumeHandler::new(state.clone()));
    blocking!(player::PlayerItemDamageHandler::new(state.clone()));
    blocking!(player::PlayerItemMendHandler::new(state.clone()));
    blocking!(player::PlayerMoveHandler::new(state.clone()));
    blocking!(player::PlayerOpenSignHandler::new(state.clone()));
    blocking!(player::PlayerPortalHandler::new(state.clone()));
    blocking!(player::PlayerShearEntityHandler::new(state.clone()));
    blocking!(player::PlayerSwapHandsHandler::new(state.clone()));
    blocking!(player::PlayerTakeLecternBookHandler::new(state.clone()));
    blocking!(player::PlayerUnleashEntityHandler::new(state.clone()));

    Ok(())
}
