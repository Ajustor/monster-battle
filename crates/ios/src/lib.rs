//! # Monster Battle — Application iOS
//!
//! Crate frontend mobile utilisant **Bevy** comme moteur de rendu 2D.
//!
//! Réutilise les crates partagées :
//! - `monster-battle-core` : logique de jeu, types, combat
//! - `monster-battle-storage` : persistance chiffrée des monstres
//! - `monster-battle-network` : client WebSocket pour le PvP
//!
//! Les sprites pixel-art 16×16 sont convertis en textures Bevy au runtime
//! via le module [`sprites`].

pub mod audio;
pub mod battle_effects;
pub mod connection;
pub mod game;
pub mod net_task;
pub mod screens;
pub mod sprites;
pub mod ui;
pub mod updater;

use bevy::prelude::*;
use bevy::state::app::AppExtStates;

use game::{GamePlugin, GameScreen};

/// Point d'entrée iOS — appelé depuis AppDelegate.swift.
/// Le nom `ios_main` doit être exposé avec `#[unsafe(no_mangle)]`.
#[unsafe(no_mangle)]
pub extern "C" fn ios_main() {
    // Logging iOS via env_logger (compatible iOS)
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .try_init();
    log::info!("🐉 Monster Battle — démarrage iOS");

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "🐉 Monster Battle".to_string(),
                        resolution: (480., 854.).into(), // 16:9 portrait mobile
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()), // pixel-art : pas de lissage
        )
        .init_state::<GameScreen>()
        .add_plugins(GamePlugin)
        .add_plugins(ui::UiPlugin)
        .add_plugins(sprites::SpritePlugin)
        .add_plugins(audio::AudioPlugin)
        .add_plugins(connection::ConnectionPlugin)
        .run();
}
