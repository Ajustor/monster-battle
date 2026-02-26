//! # Monster Battle — Application Android
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

pub mod game;
pub mod screens;
pub mod sprites;
pub mod ui;

use bevy::prelude::*;
use bevy::state::app::AppExtStates;

use game::{GamePlugin, GameScreen};

/// Point d'entrée Bevy (fonctionne sur desktop ET Android).
/// Sur Android, l'activité native appelle cette fonction via `android_activity`.
#[bevy_main]
fn main() {
    // Initialiser le logger Android (logcat) avant tout le reste
    #[cfg(target_os = "android")]
    {
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Info)
                .with_tag("MonsterBattle"),
        );
        log::info!("🐉 Monster Battle — démarrage Android");
    }

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "🐉 Monster Battle".to_string(),
                        resolution: (480., 854.).into(), // 16:9 portrait mobile
                        resizable: true,
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
        .run();
}
