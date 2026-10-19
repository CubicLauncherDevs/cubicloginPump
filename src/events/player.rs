use pumpkin_plugin_api::Server;
use pumpkin_plugin_api::events::{
    BlockBreakEvent, BlockPlaceEvent, CraftItemEvent, EventHandler, InventoryClickEvent,
    InventoryDragEvent, InventoryOpenEvent, PlayerArmorStandManipulateEvent, PlayerBedEnterEvent,
    PlayerBucketEntityEvent, PlayerChangeWorldEvent, PlayerChatEvent, PlayerCommandSendEvent,
    PlayerDropItemEvent, PlayerEditBookEvent, PlayerEggThrowEvent, PlayerElytraBoostEvent,
    PlayerFishEvent, PlayerHarvestBlockEvent, PlayerInteractAtEntityEvent,
    PlayerInteractEntityEvent, PlayerInteractEvent, PlayerItemConsumeEvent, PlayerItemDamageEvent,
    PlayerItemMendEvent, PlayerJoinEvent, PlayerLeaveEvent, PlayerMoveEvent, PlayerOpenSignEvent,
    PlayerPortalEvent, PlayerShearEntityEvent, PlayerSwapHandsEvent, PlayerTakeLecternBookEvent,
    PlayerUnleashEntityEvent,
};
use pumpkin_plugin_api::events_wit::{
    BlockBreakEventData, BlockPlaceEventData, CraftItemEventData, InventoryClickEventData,
    InventoryDragEventData, InventoryOpenEventData, PlayerArmorStandManipulateEventData,
    PlayerBedEnterEventData, PlayerBucketEntityEventData, PlayerChangeWorldEventData,
    PlayerChatEventData, PlayerCommandSendEventData, PlayerDropItemEventData,
    PlayerEditBookEventData, PlayerEggThrowEventData, PlayerElytraBoostEventData,
    PlayerFishEventData, PlayerHarvestBlockEventData, PlayerInteractAtEntityEventData,
    PlayerInteractEntityEventData, PlayerInteractEventData, PlayerItemConsumeEventData,
    PlayerItemDamageEventData, PlayerItemMendEventData, PlayerJoinEventData, PlayerLeaveEventData,
    PlayerMoveEventData, PlayerOpenSignEventData, PlayerPortalEventData,
    PlayerShearEntityEventData, PlayerSwapHandsEventData, PlayerTakeLecternBookEventData,
    PlayerUnleashEntityEventData,
};
use pumpkin_plugin_api::text::TextComponent;

use crate::state::SharedState;
use crate::storage::{normalize_username, validate_username};

fn needs_authentication(state: &SharedState, username: &str) -> bool {
    state
        .read()
        .map(|state| state.requires_authentication(username))
        .unwrap_or(false)
}

fn blocked_message() -> TextComponent {
    TextComponent::text(
        "You must register or log in first: /register <password> <password> or /login <password>.",
    )
    .click_suggest_command("/login ")
    .hover_show_text(TextComponent::text("Click to fill in the login command"))
}

fn is_auth_command(command: &str) -> bool {
    let name = command.trim_start_matches('/').split_whitespace().next();

    matches!(name, Some("register" | "login"))
}

pub struct PlayerJoinHandler {
    state: SharedState,
}

impl PlayerJoinHandler {
    pub fn new(state: SharedState) -> Self {
        Self { state }
    }
}

impl EventHandler<PlayerJoinEvent> for PlayerJoinHandler {
    fn handle(&self, _server: Server, event: PlayerJoinEventData) -> PlayerJoinEventData {
        let username = normalize_username(&event.player.get_name());
        let (require_login, store) = {
            let mut state = match self.state.write() {
                Ok(state) => state,
                Err(_) => return event,
            };
            state.clear_authenticated(&username);
            (state.config.require_login_on_join, state.store.clone())
        };

        tracing::info!(
            player = %username,
            require_login,
            "player joined"
        );

        if require_login && validate_username(&username).is_ok() {
            event.player.set_movement_locked(true);
        }

        if require_login
            && validate_username(&username).is_ok()
            && let Some(store) = store
        {
            match store.get(&username) {
                Ok(Some(_)) => {
                    tracing::info!(player = %username, "account found, prompting login");
                    event.player.send_system_message(login_message(), false);
                }
                Ok(None) => {
                    tracing::info!(player = %username, "no account found, prompting registration");
                    event.player.send_system_message(register_message(), false);
                }
                Err(error) => {
                    tracing::error!(
                        error = %error,
                        player = %username,
                        "failed to read account store on join"
                    );
                }
            }
        }

        event
    }
}

fn login_message() -> TextComponent {
    TextComponent::text("Welcome back! Please authenticate with /login <password>.")
        .click_suggest_command("/login ")
        .hover_show_text(TextComponent::text("Click to fill in the login command"))
}

fn register_message() -> TextComponent {
    TextComponent::text("Welcome! Create your account with /register <password> <password>.")
        .click_suggest_command("/register ")
        .hover_show_text(TextComponent::text("Click to fill in the register command"))
}

pub struct PlayerChatHandler {
    state: SharedState,
}

impl PlayerChatHandler {
    pub fn new(state: SharedState) -> Self {
        Self { state }
    }
}

impl EventHandler<PlayerChatEvent> for PlayerChatHandler {
    fn handle(&self, _server: Server, mut event: PlayerChatEventData) -> PlayerChatEventData {
        let username = normalize_username(&event.player.get_name());

        if needs_authentication(&self.state, &username) {
            event.cancelled = true;
            event.player.send_system_message(blocked_message(), false);
        }

        event
    }
}

pub struct PlayerCommandHandler {
    state: SharedState,
}

impl PlayerCommandHandler {
    pub fn new(state: SharedState) -> Self {
        Self { state }
    }
}

impl EventHandler<PlayerCommandSendEvent> for PlayerCommandHandler {
    fn handle(
        &self,
        _server: Server,
        mut event: PlayerCommandSendEventData,
    ) -> PlayerCommandSendEventData {
        let username = normalize_username(&event.player.get_name());

        if needs_authentication(&self.state, &username) && !is_auth_command(&event.command) {
            event.cancelled = true;
            event.player.send_system_message(blocked_message(), false);
        }

        event
    }
}

pub struct PlayerMoveHandler {
    state: SharedState,
}

impl PlayerMoveHandler {
    pub fn new(state: SharedState) -> Self {
        Self { state }
    }
}

impl EventHandler<PlayerMoveEvent> for PlayerMoveHandler {
    fn handle(&self, _server: Server, mut event: PlayerMoveEventData) -> PlayerMoveEventData {
        let username = normalize_username(&event.player.get_name());

        if needs_authentication(&self.state, &username) {
            event.cancelled = true;
        }

        event
    }
}

pub struct BlockBreakHandler {
    state: SharedState,
}

impl BlockBreakHandler {
    pub fn new(state: SharedState) -> Self {
        Self { state }
    }
}

impl EventHandler<BlockBreakEvent> for BlockBreakHandler {
    fn handle(&self, _server: Server, mut event: BlockBreakEventData) -> BlockBreakEventData {
        let blocked = event.player.as_ref().is_some_and(|player| {
            let username = normalize_username(&player.get_name());
            needs_authentication(&self.state, &username)
        });

        if blocked {
            event.cancelled = true;
        }

        event
    }
}

macro_rules! silent_blocker {
    ($name:ident, $event:ty, $data:ty) => {
        pub struct $name {
            state: SharedState,
        }

        impl $name {
            pub fn new(state: SharedState) -> Self {
                Self { state }
            }
        }

        impl EventHandler<$event> for $name {
            fn handle(&self, _server: Server, mut event: $data) -> $data {
                let username = normalize_username(&event.player.get_name());

                if needs_authentication(&self.state, &username) {
                    event.cancelled = true;
                }

                event
            }
        }
    };
}

silent_blocker!(BlockPlaceHandler, BlockPlaceEvent, BlockPlaceEventData);
silent_blocker!(CraftItemHandler, CraftItemEvent, CraftItemEventData);
silent_blocker!(
    InventoryClickHandler,
    InventoryClickEvent,
    InventoryClickEventData
);
silent_blocker!(
    InventoryDragHandler,
    InventoryDragEvent,
    InventoryDragEventData
);
silent_blocker!(
    InventoryOpenHandler,
    InventoryOpenEvent,
    InventoryOpenEventData
);
silent_blocker!(
    PlayerArmorStandManipulateHandler,
    PlayerArmorStandManipulateEvent,
    PlayerArmorStandManipulateEventData
);
silent_blocker!(
    PlayerBedEnterHandler,
    PlayerBedEnterEvent,
    PlayerBedEnterEventData
);
silent_blocker!(
    PlayerBucketEntityHandler,
    PlayerBucketEntityEvent,
    PlayerBucketEntityEventData
);
silent_blocker!(
    PlayerChangeWorldHandler,
    PlayerChangeWorldEvent,
    PlayerChangeWorldEventData
);
silent_blocker!(
    PlayerDropItemHandler,
    PlayerDropItemEvent,
    PlayerDropItemEventData
);
silent_blocker!(
    PlayerEditBookHandler,
    PlayerEditBookEvent,
    PlayerEditBookEventData
);
silent_blocker!(
    PlayerEggThrowHandler,
    PlayerEggThrowEvent,
    PlayerEggThrowEventData
);
silent_blocker!(
    PlayerElytraBoostHandler,
    PlayerElytraBoostEvent,
    PlayerElytraBoostEventData
);
silent_blocker!(PlayerFishHandler, PlayerFishEvent, PlayerFishEventData);
silent_blocker!(
    PlayerHarvestBlockHandler,
    PlayerHarvestBlockEvent,
    PlayerHarvestBlockEventData
);
silent_blocker!(
    PlayerInteractHandler,
    PlayerInteractEvent,
    PlayerInteractEventData
);
silent_blocker!(
    PlayerInteractAtEntityHandler,
    PlayerInteractAtEntityEvent,
    PlayerInteractAtEntityEventData
);
silent_blocker!(
    PlayerInteractEntityHandler,
    PlayerInteractEntityEvent,
    PlayerInteractEntityEventData
);
silent_blocker!(
    PlayerItemConsumeHandler,
    PlayerItemConsumeEvent,
    PlayerItemConsumeEventData
);
silent_blocker!(
    PlayerItemDamageHandler,
    PlayerItemDamageEvent,
    PlayerItemDamageEventData
);
silent_blocker!(
    PlayerItemMendHandler,
    PlayerItemMendEvent,
    PlayerItemMendEventData
);
silent_blocker!(
    PlayerOpenSignHandler,
    PlayerOpenSignEvent,
    PlayerOpenSignEventData
);
silent_blocker!(
    PlayerPortalHandler,
    PlayerPortalEvent,
    PlayerPortalEventData
);
silent_blocker!(
    PlayerShearEntityHandler,
    PlayerShearEntityEvent,
    PlayerShearEntityEventData
);
silent_blocker!(
    PlayerSwapHandsHandler,
    PlayerSwapHandsEvent,
    PlayerSwapHandsEventData
);
silent_blocker!(
    PlayerTakeLecternBookHandler,
    PlayerTakeLecternBookEvent,
    PlayerTakeLecternBookEventData
);
silent_blocker!(
    PlayerUnleashEntityHandler,
    PlayerUnleashEntityEvent,
    PlayerUnleashEntityEventData
);

pub struct PlayerLeaveHandler {
    state: SharedState,
}

impl PlayerLeaveHandler {
    pub fn new(state: SharedState) -> Self {
        Self { state }
    }
}

impl EventHandler<PlayerLeaveEvent> for PlayerLeaveHandler {
    fn handle(&self, _server: Server, event: PlayerLeaveEventData) -> PlayerLeaveEventData {
        let username = normalize_username(&event.player.get_name());
        let require_login = {
            let mut state = match self.state.write() {
                Ok(state) => state,
                Err(_) => return event,
            };
            state.clear_authenticated(&username);
            state.config.require_login_on_join
        };

        tracing::info!(
            player = %username,
            require_login,
            "player left"
        );

        event
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_auth_commands_are_allowed() {
        assert!(is_auth_command("register secret secret"));
        assert!(is_auth_command("login secret"));
        assert!(is_auth_command("/register secret secret"));
        assert!(is_auth_command("/login secret"));
        assert!(!is_auth_command("gamemode creative"));
        assert!(!is_auth_command("registerevil"));
        assert!(!is_auth_command(""));
    }
}
