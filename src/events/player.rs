use pumpkin_plugin_api::Server;
use pumpkin_plugin_api::events::{EventHandler, PlayerJoinEvent, PlayerLeaveEvent};
use pumpkin_plugin_api::events_wit::{PlayerJoinEventData, PlayerLeaveEventData};

use crate::state::SharedState;

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
        let username = event.player.get_name();
        let (require_login, timeout) = self
            .state
            .read()
            .map(|state| {
                (
                    state.config.require_login_on_join,
                    state.config.session_timeout_seconds,
                )
            })
            .unwrap_or((true, 300));

        tracing::info!(
            player = %username,
            require_login,
            session_timeout_seconds = timeout,
            "player joined"
        );

        event
    }
}

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
        let username = event.player.get_name();
        let require_login = self
            .state
            .read()
            .map(|state| state.config.require_login_on_join)
            .unwrap_or(true);

        tracing::info!(
            player = %username,
            require_login,
            "player left"
        );

        event
    }
}
