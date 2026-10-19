use pumpkin_plugin_api::Server;
use pumpkin_plugin_api::events::{EventHandler, PlayerJoinEvent, PlayerLeaveEvent};
use pumpkin_plugin_api::events_wit::{PlayerJoinEventData, PlayerLeaveEventData};
use pumpkin_plugin_api::text::TextComponent;

use crate::state::SharedState;
use crate::storage::{normalize_username, validate_username};

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
        let (require_login, store) = self
            .state
            .read()
            .map(|state| (state.config.require_login_on_join, state.store.clone()))
            .unwrap_or((true, None));

        tracing::info!(
            player = %username,
            require_login,
            "player joined"
        );

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
    TextComponent::text("Welcome! Create your account with /register <password>.")
        .click_suggest_command("/register ")
        .hover_show_text(TextComponent::text("Click to fill in the register command"))
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
