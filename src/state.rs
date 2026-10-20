use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::config::PluginConfig;
use crate::db::AccountRepo;
use crate::i18n::MessageTexts;
use crate::storage::now_secs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthPrompt {
    Login,
    Register,
    Fallback,
}

#[derive(Debug, Clone)]
pub struct AuthSession {
    pub player_name: String,
    pub deadline: Option<u64>,
}

#[derive(Default)]
pub struct PluginState {
    pub config: PluginConfig,
    pub messages: Arc<MessageTexts>,
    pub store: Option<Arc<dyn AccountRepo>>,
    pub pepper: Option<Vec<u8>>,
    pub data_dir: PathBuf,
    pub authenticated: HashSet<String>,
    pub pending_auth: HashMap<String, AuthSession>,
}

impl PluginState {
    pub fn mark_authenticated(&mut self, username: &str) {
        self.authenticated.insert(username.to_string());
        self.pending_auth.remove(username);
    }

    pub fn clear_authenticated(&mut self, username: &str) {
        self.authenticated.remove(username);
        self.pending_auth.remove(username);
    }

    pub fn start_authentication(
        &mut self,
        username: &str,
        player_name: &str,
        timeout_seconds: u64,
    ) {
        let deadline = (timeout_seconds > 0).then(|| now_secs().saturating_add(timeout_seconds));

        self.pending_auth.insert(
            username.to_string(),
            AuthSession {
                player_name: player_name.to_string(),
                deadline,
            },
        );
    }

    pub fn finish_authentication(&mut self, username: &str) -> Option<AuthSession> {
        self.pending_auth.remove(username)
    }

    pub fn requires_authentication(&self, username: &str) -> bool {
        self.config.require_login_on_join && !self.authenticated.contains(username)
    }
}

pub type SharedState = Arc<RwLock<PluginState>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authentication_is_required_until_marked() {
        let mut state = PluginState::default();
        state.config.require_login_on_join = true;

        assert!(state.requires_authentication("steve"));

        state.mark_authenticated("steve");
        assert!(!state.requires_authentication("steve"));

        state.clear_authenticated("steve");
        assert!(state.requires_authentication("steve"));
    }

    #[test]
    fn authentication_is_not_required_when_disabled() {
        let mut state = PluginState::default();
        state.config.require_login_on_join = false;

        assert!(!state.requires_authentication("steve"));
    }

    #[test]
    fn pending_authentication_tracks_a_deadline() {
        let mut state = PluginState::default();

        state.start_authentication("steve", "Steve", 300);
        let session = state.pending_auth.get("steve").unwrap();

        assert_eq!(session.player_name, "Steve");
        assert!(session.deadline.is_some());
    }

    #[test]
    fn zero_timeout_disables_the_deadline() {
        let mut state = PluginState::default();

        state.start_authentication("steve", "Steve", 0);

        assert!(
            state
                .pending_auth
                .get("steve")
                .is_some_and(|session| session.deadline.is_none())
        );
    }

    #[test]
    fn authenticating_clears_the_pending_screen() {
        let mut state = PluginState::default();

        state.start_authentication("steve", "Steve", 300);
        state.mark_authenticated("steve");

        assert!(!state.pending_auth.contains_key("steve"));
        assert!(!state.requires_authentication("steve"));
    }

    #[test]
    fn leaving_clears_the_pending_session() {
        let mut state = PluginState::default();

        state.start_authentication("steve", "Steve", 300);
        state.clear_authenticated("steve");

        assert!(!state.pending_auth.contains_key("steve"));
    }
}
