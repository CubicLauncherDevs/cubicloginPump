use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::config::PluginConfig;
use crate::db::AccountRepo;

#[derive(Default)]
pub struct PluginState {
    pub config: PluginConfig,
    pub store: Option<Arc<dyn AccountRepo>>,
    pub pepper: Option<Vec<u8>>,
    pub data_dir: PathBuf,
    pub authenticated: HashSet<String>,
}

impl PluginState {
    pub fn mark_authenticated(&mut self, username: &str) {
        self.authenticated.insert(username.to_string());
    }

    pub fn clear_authenticated(&mut self, username: &str) {
        self.authenticated.remove(username);
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
}
