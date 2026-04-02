//! # Monster Battle — Application iOS
//!
//! Point d'entrée iOS — injecte la PlatformConfig et délègue
//! tout le reste à la crate partagée `monster-battle-mobile-ui`.

use bevy::prelude::*;

use monster_battle_mobile_ui::platform::PlatformConfig;
use monster_battle_mobile_ui::game::{GamePlugin, GameScreen};
use monster_battle_mobile_ui::ui::UiPlugin;
use monster_battle_mobile_ui::sprites::SpritePlugin;
use monster_battle_mobile_ui::audio::AudioPlugin;
use monster_battle_mobile_ui::connection::ConnectionPlugin;

/// Point d'entrée iOS — appelé depuis AppDelegate.swift.
#[unsafe(no_mangle)]
pub extern "C" fn ios_main() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .try_init();
    log::info!("🐉 Monster Battle — démarrage iOS");

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());

    let config = PlatformConfig {
        safe_top: 44.0,
        safe_bottom: 34.0,
        data_dir: std::path::PathBuf::from(format!("{}/Documents/monster-battle", home)),
    };

    App::new()
        .insert_resource(config)
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "🐉 Monster Battle".to_string(),
                        resolution: (480., 854.).into(),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .init_state::<GameScreen>()
        .add_plugins(GamePlugin)
        .add_plugins(UiPlugin)
        .add_plugins(SpritePlugin)
        .add_plugins(AudioPlugin)
        .add_plugins(ConnectionPlugin)
        .run();
}
