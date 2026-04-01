//! # Monster Battle — Application Android
//!
//! Point d'entrée Android — injecte la PlatformConfig et délègue
//! tout le reste à la crate partagée `monster-battle-mobile-ui`.

use bevy::prelude::*;

use monster_battle_mobile_ui::platform::PlatformConfig;
use monster_battle_mobile_ui::game::{GamePlugin, GameScreen};
use monster_battle_mobile_ui::ui::UiPlugin;
use monster_battle_mobile_ui::sprites::SpritePlugin;
use monster_battle_mobile_ui::audio::AudioPlugin;
use monster_battle_mobile_ui::connection::ConnectionPlugin;

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

    #[cfg(target_os = "android")]
    let config = PlatformConfig {
        safe_top: 48.0,
        safe_bottom: 52.0,
        data_dir: std::path::PathBuf::from("/data/data/com.ajustor.monsterbattle/files"),
    };

    #[cfg(not(target_os = "android"))]
    let config = PlatformConfig {
        safe_top: 16.0,
        safe_bottom: 16.0,
        data_dir: {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            std::path::PathBuf::from(format!("{}/.local/share/monster-battle", home))
        },
    };

    App::new()
        .insert_resource(config)
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "🐉 Monster Battle".to_string(),
                        resolution: (480., 854.).into(),
                        resizable: true,
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
