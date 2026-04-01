//! Audio integration — bridges the audio crate with the TUI application.
//!
//! All functionality is behind the `audio` feature flag.
//! When disabled, the functions are no-ops.
//!
//! Because `rodio::OutputStream` is not `Send`/`Sync`, the engine lives in a
//! thread-local — initialised once on the main thread.
//!
//! Audio settings (mute state) are persisted to a JSON file in the data
//! directory so they survive restarts.

#[cfg(feature = "audio")]
use monster_battle_audio::{AudioEngine, Sfx, tracks};

#[cfg(feature = "audio")]
use std::cell::RefCell;

#[cfg(feature = "audio")]
thread_local! {
    static ENGINE: RefCell<Option<AudioEngine>> = const { RefCell::new(None) };
}

// ── Settings persistence ────────────────────────────────────────

#[cfg(feature = "audio")]
fn settings_path() -> std::path::PathBuf {
    let base = if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        std::path::PathBuf::from(xdg)
    } else if let Some(home) = std::env::var_os("HOME") {
        std::path::PathBuf::from(home).join(".local").join("share")
    } else {
        std::path::PathBuf::from(".")
    };
    base.join("monster-battle").join("settings.json")
}

#[cfg(feature = "audio")]
#[derive(serde::Serialize, serde::Deserialize, Default)]
struct AudioSettings {
    #[serde(default)]
    muted: bool,
}

#[cfg(feature = "audio")]
fn load_settings() -> AudioSettings {
    let path = settings_path();
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

#[cfg(feature = "audio")]
fn save_settings(settings: &AudioSettings) {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(settings) {
        let _ = std::fs::write(&path, json);
    }
}

/// Initialise the audio engine.  Safe to call multiple times — only the first
/// call on the main thread matters.
/// Restores the mute state from the persisted settings file.
#[cfg(feature = "audio")]
pub fn init() {
    ENGINE.with(|cell| {
        let mut opt = cell.borrow_mut();
        if opt.is_none()
            && let Some(engine) = AudioEngine::try_new()
        {
            let settings = load_settings();
            engine.set_muted(settings.muted);
            *opt = Some(engine);
        }
    });
}

#[cfg(not(feature = "audio"))]
pub fn init() {}

#[cfg(feature = "audio")]
fn with_engine(f: impl FnOnce(&AudioEngine)) {
    ENGINE.with(|cell| {
        if let Some(ref e) = *cell.borrow() {
            f(e);
        }
    });
}

// ── Music ───────────────────────────────────────────────────────

/// Start playing the title / main menu theme.
pub fn play_title_music() {
    #[cfg(feature = "audio")]
    with_engine(|e| e.play_track(&tracks::title_theme()));
}

/// Start playing the battle theme.
pub fn play_battle_music() {
    #[cfg(feature = "audio")]
    with_engine(|e| e.play_track(&tracks::battle_theme()));
}

/// Play the victory fanfare.
pub fn play_victory_music() {
    #[cfg(feature = "audio")]
    with_engine(|e| e.play_track(&tracks::victory_fanfare()));
}

/// Play the defeat theme.
pub fn play_defeat_music() {
    #[cfg(feature = "audio")]
    with_engine(|e| e.play_track(&tracks::defeat_theme()));
}

/// Play the exploration / monster list theme.
pub fn play_exploration_music() {
    #[cfg(feature = "audio")]
    with_engine(|e| e.play_track(&tracks::exploration_theme()));
}

/// Play the breeding theme.
pub fn play_breeding_music() {
    #[cfg(feature = "audio")]
    with_engine(|e| e.play_track(&tracks::breeding_theme()));
}

/// Play the cemetery theme.
pub fn play_cemetery_music() {
    #[cfg(feature = "audio")]
    with_engine(|e| e.play_track(&tracks::cemetery_theme()));
}

/// Stop music.
#[allow(dead_code)]
pub fn stop_music() {
    #[cfg(feature = "audio")]
    with_engine(|e| e.stop_music());
}

// ── SFX ─────────────────────────────────────────────────────────

/// Play the attack hit SFX.
pub fn sfx_hit() {
    #[cfg(feature = "audio")]
    with_engine(|e| e.play_sfx(Sfx::Hit));
}

/// Play the critical hit SFX.
pub fn sfx_critical_hit() {
    #[cfg(feature = "audio")]
    with_engine(|e| e.play_sfx(Sfx::CriticalHit));
}

/// Play the menu selection SFX.
pub fn sfx_menu_select() {
    #[cfg(feature = "audio")]
    with_engine(|e| e.play_sfx(Sfx::MenuSelect));
}

/// Play the menu cursor move SFX.
pub fn sfx_menu_move() {
    #[cfg(feature = "audio")]
    with_engine(|e| e.play_sfx(Sfx::MenuMove));
}

/// Play the level-up jingle.
pub fn sfx_level_up() {
    #[cfg(feature = "audio")]
    with_engine(|e| e.play_sfx(Sfx::LevelUp));
}

/// Play the monster death SFX.
pub fn sfx_monster_death() {
    #[cfg(feature = "audio")]
    with_engine(|e| e.play_sfx(Sfx::MonsterDeath));
}

/// Play the heal SFX.
pub fn sfx_heal() {
    #[cfg(feature = "audio")]
    with_engine(|e| e.play_sfx(Sfx::Heal));
}

/// Play the match found SFX.
pub fn sfx_match_found() {
    #[cfg(feature = "audio")]
    with_engine(|e| e.play_sfx(Sfx::MatchFound));
}

/// Play the flee SFX.
pub fn sfx_flee() {
    #[cfg(feature = "audio")]
    with_engine(|e| e.play_sfx(Sfx::Flee));
}

// ── Volume / mute ───────────────────────────────────────────────

/// Toggle mute on/off and persist the setting.
pub fn toggle_mute() {
    #[cfg(feature = "audio")]
    {
        with_engine(|e| e.toggle_mute());
        // Persist the new mute state
        let muted = is_muted();
        save_settings(&AudioSettings { muted });
    }
}

/// Is audio muted?
#[allow(dead_code)]
pub fn is_muted() -> bool {
    #[cfg(feature = "audio")]
    {
        let mut result = false;
        ENGINE.with(|cell| {
            if let Some(ref e) = *cell.borrow() {
                result = e.is_muted();
            }
        });
        result
    }
    #[cfg(not(feature = "audio"))]
    {
        true
    }
}

/// Set music volume (0.0–1.0).
#[allow(dead_code)]
pub fn set_music_volume(_vol: f32) {
    #[cfg(feature = "audio")]
    with_engine(|e| e.set_music_volume(_vol));
}

/// Set SFX volume (0.0–1.0).
#[allow(dead_code)]
pub fn set_sfx_volume(_vol: f32) {
    #[cfg(feature = "audio")]
    with_engine(|e| e.set_sfx_volume(_vol));
}

/// Get music volume.
#[allow(dead_code)]
pub fn music_volume() -> f32 {
    #[cfg(feature = "audio")]
    {
        let mut vol = 0.5;
        ENGINE.with(|cell| {
            if let Some(ref e) = *cell.borrow() {
                vol = e.music_volume();
            }
        });
        vol
    }
    #[cfg(not(feature = "audio"))]
    {
        0.0
    }
}

/// Get SFX volume.
#[allow(dead_code)]
pub fn sfx_volume() -> f32 {
    #[cfg(feature = "audio")]
    {
        let mut vol = 0.7;
        ENGINE.with(|cell| {
            if let Some(ref e) = *cell.borrow() {
                vol = e.sfx_volume();
            }
        });
        vol
    }
    #[cfg(not(feature = "audio"))]
    {
        0.0
    }
}
