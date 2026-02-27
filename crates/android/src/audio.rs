//! Audio integration — musiques et effets sonores.
//!
//! Utilise le moteur de synthèse `monster-battle-audio` (rodio)
//! pour jouer les musiques et SFX du jeu.
//!
//! L'`AudioEngine` n'étant pas `Send`, il est stocké en tant que
//! ressource `NonSend` dans Bevy.

use bevy::prelude::*;
use monster_battle_audio::{AudioEngine, Sfx, tracks};

use crate::game::GameScreen;

// ═══════════════════════════════════════════════════════════════════
//  Ressource audio (NonSend car rodio::OutputStream n'est pas Send)
// ═══════════════════════════════════════════════════════════════════

/// Wrapper autour de l'engine audio pour l'intégrer à Bevy.
pub struct AudioState {
    pub engine: AudioEngine,
}

// ═══════════════════════════════════════════════════════════════════
//  Plugin audio
// ═══════════════════════════════════════════════════════════════════

/// Plugin qui initialise l'audio et joue les musiques aux transitions d'écran.
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        // Tenter d'initialiser l'engine audio
        if let Some(engine) = AudioEngine::try_new() {
            log::info!("Audio engine initialise avec succes");
            app.insert_non_send_resource(AudioState { engine });
        } else {
            log::warn!("Pas de peripherique audio — musique desactivee");
            return; // Pas d'audio → pas de systèmes à enregistrer
        }

        app
            // ── Musiques par écran ─────────────────────────────
            .add_systems(OnEnter(GameScreen::MainMenu), play_title_music)
            .add_systems(OnEnter(GameScreen::MonsterList), play_exploration_music)
            .add_systems(OnEnter(GameScreen::Battle), play_battle_music)
            .add_systems(OnEnter(GameScreen::Cemetery), play_cemetery_music)
            .add_systems(OnEnter(GameScreen::BreedingSearching), play_breeding_music)
            .add_systems(OnEnter(GameScreen::BreedingNaming), play_breeding_music)
            .add_systems(OnEnter(GameScreen::BreedingResult), play_breeding_music);
    }
}

// ═══════════════════════════════════════════════════════════════════
//  Systèmes de musique par écran
// ═══════════════════════════════════════════════════════════════════

fn play_title_music(audio: NonSend<AudioState>) {
    audio.engine.play_track(&tracks::title_theme());
}

fn play_exploration_music(audio: NonSend<AudioState>) {
    audio.engine.play_track(&tracks::exploration_theme());
}

fn play_battle_music(audio: NonSend<AudioState>) {
    audio.engine.play_track(&tracks::battle_theme());
}

fn play_cemetery_music(audio: NonSend<AudioState>) {
    audio.engine.play_track(&tracks::cemetery_theme());
}

fn play_breeding_music(audio: NonSend<AudioState>) {
    audio.engine.play_track(&tracks::breeding_theme());
}

// ═══════════════════════════════════════════════════════════════════
//  Fonctions utilitaires pour les SFX (appelées depuis les écrans)
// ═══════════════════════════════════════════════════════════════════

/// Joue un SFX si l'audio est disponible.
pub fn play_sfx(audio: &Option<NonSend<AudioState>>, sfx: Sfx) {
    if let Some(audio) = audio {
        audio.engine.play_sfx(sfx);
    }
}

/// Joue le jingle de victoire.
pub fn play_victory(audio: &NonSend<AudioState>) {
    audio.engine.play_track(&tracks::victory_fanfare());
}

/// Joue le thème de défaite.
pub fn play_defeat(audio: &NonSend<AudioState>) {
    audio.engine.play_track(&tracks::defeat_theme());
}
