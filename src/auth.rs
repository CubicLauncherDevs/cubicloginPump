use std::sync::Arc;

use pumpkin_plugin_api::Server;
use pumpkin_plugin_api::player::{
    BedrockDisconnectReason, BedrockKickOptions, JavaKickOptions, Player,
};
use pumpkin_plugin_api::scheduler::SchedulerExt;
use pumpkin_plugin_api::text::TextComponent;
use pumpkin_plugin_api::wit::pumpkin::plugin::status_effect::{
    StatusEffectInstance, StatusEffectType,
};

use crate::i18n::{MessageTexts, component, format_text};
use crate::state::{AuthPrompt, AuthSession, SharedState};
use crate::storage::now_secs;

const FADE_IN_TICKS: i32 = 10;
const FADE_OUT_TICKS: i32 = 20;
const LONG_EFFECT_SECONDS: u32 = 3600;
const TICKS_PER_SECOND: u32 = 20;

pub fn begin(state: &SharedState, player: &Player, username: &str, prompt: AuthPrompt) {
    let (messages, timeout_seconds) = match state.read() {
        Ok(state) => {
            if !state.config.require_login_on_join {
                return;
            }
            (
                Arc::clone(&state.messages),
                state.config.session_timeout_seconds,
            )
        }
        Err(_) => return,
    };

    if let Ok(mut state) = state.write() {
        state.start_authentication(username, &player.get_name(), timeout_seconds);
    }

    show_screen(player, &messages, prompt, timeout_seconds);
}

pub fn finish(state: &SharedState, player: &Player, username: &str) {
    if let Ok(mut state) = state.write() {
        state.finish_authentication(username);
    }

    clear_presentation(player);
}

pub fn clear_presentation(player: &Player) {
    player.remove_effect(StatusEffectType::Blindness);
    player.send_title_animation(0, 0, 0);
    player.show_title(TextComponent::text(""));
    player.show_subtitle(TextComponent::text(""));
    player.show_actionbar(TextComponent::text(""));
}

pub fn spawn_ticker(server: &Server, state: SharedState) -> u32 {
    server.schedule_repeating_task(20, 20, move |server| tick(&server, &state))
}

pub fn tick(server: &Server, state: &SharedState) {
    let (messages, require_login) = match state.read() {
        Ok(state) => (
            Arc::clone(&state.messages),
            state.config.require_login_on_join,
        ),
        Err(_) => return,
    };

    let sessions: Vec<(String, AuthSession)> = match state.read() {
        Ok(state) => state
            .pending_auth
            .iter()
            .map(|(username, session)| (username.clone(), session.clone()))
            .collect(),
        Err(_) => return,
    };

    if !require_login {
        for (_, session) in sessions {
            if let Some(player) = server.get_player_by_name(&session.player_name) {
                clear_presentation(&player);
            }
        }

        if let Ok(mut state) = state.write() {
            state.pending_auth.clear();
        }
        return;
    }

    let now = now_secs();

    for (username, session) in sessions {
        let Some(player) = server.get_player_by_name(&session.player_name) else {
            if let Ok(mut state) = state.write() {
                state.finish_authentication(&username);
            }
            continue;
        };

        let still_pending = state.read().ok().is_some_and(|state| {
            state.pending_auth.get(&username).is_some_and(|current| {
                current.player_name == session.player_name && current.deadline == session.deadline
            })
        });
        if !still_pending {
            continue;
        }

        match session.deadline {
            None => {
                if !player.has_effect(StatusEffectType::Blindness) {
                    apply_blindness(&player, LONG_EFFECT_SECONDS);
                }
            }
            Some(deadline) if now >= deadline => {
                if let Ok(mut state) = state.write() {
                    state.finish_authentication(&username);
                }
                kick_for_timeout(&player, &messages);
            }
            Some(deadline) => {
                let remaining = deadline.saturating_sub(now);
                let text = format_text(
                    &messages.auth_countdown,
                    &[("time", &format_duration(remaining))],
                );
                player.show_actionbar(component(&text));
            }
        }
    }
}

pub fn format_duration(seconds: u64) -> String {
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

fn show_screen(player: &Player, messages: &MessageTexts, prompt: AuthPrompt, timeout_seconds: u64) {
    let (title, subtitle) = match prompt {
        AuthPrompt::Login => (&messages.auth_title_login, &messages.auth_subtitle_login),
        AuthPrompt::Register => (
            &messages.auth_title_register,
            &messages.auth_subtitle_register,
        ),
        AuthPrompt::Fallback => (
            &messages.auth_title_fallback,
            &messages.auth_subtitle_fallback,
        ),
    };

    let stay_ticks = if timeout_seconds == 0 {
        i32::MAX
    } else {
        i32::try_from(timeout_seconds.saturating_mul(u64::from(TICKS_PER_SECOND)))
            .unwrap_or(i32::MAX)
    };

    player.send_title_animation(FADE_IN_TICKS, stay_ticks, FADE_OUT_TICKS);
    player.show_title(component(title));
    player.show_subtitle(component(subtitle));

    if timeout_seconds > 0 {
        let text = format_text(
            &messages.auth_countdown,
            &[("time", &format_duration(timeout_seconds))],
        );
        player.show_actionbar(component(&text));
        apply_blindness(
            player,
            u32::try_from(timeout_seconds).unwrap_or(u32::MAX).max(1),
        );
    } else if !player.has_effect(StatusEffectType::Blindness) {
        apply_blindness(player, LONG_EFFECT_SECONDS);
    }
}

fn apply_blindness(player: &Player, duration_seconds: u32) {
    player.add_effect(StatusEffectInstance {
        effect_type: StatusEffectType::Blindness,
        duration: effect_duration_ticks(duration_seconds),
        amplifier: 0,
        ambient: false,
        show_particles: false,
        show_icon: false,
    });
}

fn effect_duration_ticks(duration_seconds: u32) -> u32 {
    duration_seconds
        .saturating_mul(TICKS_PER_SECOND)
        .min(i32::MAX as u32)
}

fn strip_legacy_codes(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(character) = chars.next() {
        if matches!(character, '&' | '§')
            && chars
                .peek()
                .is_some_and(|next| next.is_ascii_alphanumeric())
        {
            chars.next();
            continue;
        }

        output.push(character);
    }

    output
}

fn kick_for_timeout(player: &Player, messages: &MessageTexts) {
    if let Some(java) = player.as_java() {
        java.kick(JavaKickOptions::new(component(&messages.auth_timeout_kick)));
    } else if let Some(bedrock) = player.as_bedrock() {
        bedrock.kick(&BedrockKickOptions::new(
            BedrockDisconnectReason::Kicked,
            strip_legacy_codes(&messages.auth_timeout_kick),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_remaining_time() {
        assert_eq!(format_duration(0), "0:00");
        assert_eq!(format_duration(9), "0:09");
        assert_eq!(format_duration(59), "0:59");
        assert_eq!(format_duration(60), "1:00");
        assert_eq!(format_duration(300), "5:00");
        assert_eq!(format_duration(3599), "59:59");
    }

    #[test]
    fn converts_effect_seconds_to_ticks() {
        assert_eq!(effect_duration_ticks(1), 20);
        assert_eq!(effect_duration_ticks(60), 1200);
        assert_eq!(effect_duration_ticks(u32::MAX), i32::MAX as u32);
    }

    #[test]
    fn strips_legacy_codes_for_bedrock_messages() {
        assert_eq!(
            strip_legacy_codes("&cAuthentication &lexpired. &f/login"),
            "Authentication expired. /login"
        );
        assert_eq!(strip_legacy_codes("&x&f&c&0&0&0&0Hex"), "Hex");
    }
}
