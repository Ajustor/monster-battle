//! Images de combat (battlebacks + effets) embarquées dans le binaire.
//! Contourne les limitations de AssetServer sur Android où asset_server.load()
//! ne peut pas lire les fichiers depuis l'APK.

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
use monster_battle_core::types::ElementType;

// ── Images embarquées ────────────────────────────────────────────────

// Battlebacks — murs
static WALL_CLOUDS:   &[u8] = include_bytes!("../assets/battlebacks/walls/Clouds.png");
static WALL_FOREST:   &[u8] = include_bytes!("../assets/battlebacks/walls/Forest.png");
static WALL_GRASMAZE: &[u8] = include_bytes!("../assets/battlebacks/walls/GrassMaze.png");
static WALL_LAVACAVE: &[u8] = include_bytes!("../assets/battlebacks/walls/LavaCave.png");
static WALL_ROCKCAVE: &[u8] = include_bytes!("../assets/battlebacks/walls/RockCave.png");

// Battlebacks — sols
static GROUND_CLOUDS:    &[u8] = include_bytes!("../assets/battlebacks/grounds/Clouds.png");
static GROUND_GRASSLAND: &[u8] = include_bytes!("../assets/battlebacks/grounds/Grassland.png");
static GROUND_GRASMAZE:  &[u8] = include_bytes!("../assets/battlebacks/grounds/GrassMaze.png");
static GROUND_LAVA2:     &[u8] = include_bytes!("../assets/battlebacks/grounds/Lava2.png");
static GROUND_ROCKCAVE:  &[u8] = include_bytes!("../assets/battlebacks/grounds/RockCave.png");

// Effets d'attaque
static FIRE_01: &[u8] = include_bytes!("../assets/effects/fire_01.png");
static FIRE_02: &[u8] = include_bytes!("../assets/effects/fire_02.png");
static FIRE_03: &[u8] = include_bytes!("../assets/effects/fire_03.png");
static WATER_01: &[u8] = include_bytes!("../assets/effects/water_01.png");
static WATER_02: &[u8] = include_bytes!("../assets/effects/water_02.png");
static WATER_03: &[u8] = include_bytes!("../assets/effects/water_03.png");
static ELECTRIC_01: &[u8] = include_bytes!("../assets/effects/electric_01.png");
static ELECTRIC_02: &[u8] = include_bytes!("../assets/effects/electric_02.png");
static ELECTRIC_03: &[u8] = include_bytes!("../assets/effects/electric_03.png");
static EARTH_01: &[u8] = include_bytes!("../assets/effects/earth_01.png");
static EARTH_02: &[u8] = include_bytes!("../assets/effects/earth_02.png");
static EARTH_03: &[u8] = include_bytes!("../assets/effects/earth_03.png");
static WIND_01: &[u8] = include_bytes!("../assets/effects/wind_01.png");
static WIND_02: &[u8] = include_bytes!("../assets/effects/wind_02.png");
static WIND_03: &[u8] = include_bytes!("../assets/effects/wind_03.png");
static SHADOW_01: &[u8] = include_bytes!("../assets/effects/shadow_01.png");
static SHADOW_02: &[u8] = include_bytes!("../assets/effects/shadow_02.png");
static SHADOW_03: &[u8] = include_bytes!("../assets/effects/shadow_03.png");
static LIGHT_01: &[u8] = include_bytes!("../assets/effects/light_01.png");
static LIGHT_02: &[u8] = include_bytes!("../assets/effects/light_02.png");
static LIGHT_03: &[u8] = include_bytes!("../assets/effects/light_03.png");
static PLANT_01: &[u8] = include_bytes!("../assets/effects/plant_01.png");
static PLANT_02: &[u8] = include_bytes!("../assets/effects/plant_02.png");
static PLANT_03: &[u8] = include_bytes!("../assets/effects/plant_03.png");
static NORMAL_01: &[u8] = include_bytes!("../assets/effects/normal_01.png");
static NORMAL_02: &[u8] = include_bytes!("../assets/effects/normal_02.png");
static NORMAL_03: &[u8] = include_bytes!("../assets/effects/normal_03.png");

// ── Resource ─────────────────────────────────────────────────────────

/// Handles vers les images de combat chargées au démarrage depuis le binaire.
/// Indexé par `ElementType as usize` :
/// Normal=0, Fire=1, Water=2, Plant=3, Electric=4, Earth=5, Wind=6, Shadow=7, Light=8
#[derive(Resource)]
pub struct BattleImages {
    /// (wall, ground) par type élémentaire.
    pub battlebacks: [(Handle<Image>, Handle<Image>); 9],
    /// [frame0, frame1, frame2] par type élémentaire.
    pub effects: [[Handle<Image>; 3]; 9],
}

impl BattleImages {
    pub fn battleback(&self, element: ElementType) -> (Handle<Image>, Handle<Image>) {
        self.battlebacks[element as usize].clone()
    }

    pub fn effect_frames(&self, element: ElementType) -> [Handle<Image>; 3] {
        self.effects[element as usize].clone()
    }
}

// ── Startup system ────────────────────────────────────────────────────

fn decode_png(bytes: &[u8]) -> Image {
    Image::from_buffer(
        bytes,
        ImageType::Extension("png"),
        CompressedImageFormats::NONE,
        true,
        ImageSampler::Default,
        RenderAssetUsages::RENDER_WORLD,
    )
    .expect("failed to decode embedded PNG")
}

/// Charge toutes les images de combat depuis les données embarquées.
/// À appeler en Startup avant tout système de combat.
pub fn setup_battle_images(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    // Handles de murs partagés
    let clouds_wall   = images.add(decode_png(WALL_CLOUDS));
    let forest_wall   = images.add(decode_png(WALL_FOREST));
    let grasmaze_wall = images.add(decode_png(WALL_GRASMAZE));
    let lavacave_wall = images.add(decode_png(WALL_LAVACAVE));
    let rockcave_wall = images.add(decode_png(WALL_ROCKCAVE));

    // Handles de sols partagés
    let clouds_ground    = images.add(decode_png(GROUND_CLOUDS));
    let grassland_ground = images.add(decode_png(GROUND_GRASSLAND));
    let grasmaze_ground  = images.add(decode_png(GROUND_GRASMAZE));
    let lava2_ground     = images.add(decode_png(GROUND_LAVA2));
    let rockcave_ground  = images.add(decode_png(GROUND_ROCKCAVE));

    // ElementType order: Normal=0, Fire=1, Water=2, Plant=3, Electric=4,
    //                    Earth=5, Wind=6, Shadow=7, Light=8
    let battlebacks = [
        (grasmaze_wall.clone(), grassland_ground.clone()), // Normal
        (lavacave_wall.clone(), lava2_ground.clone()),     // Fire
        (clouds_wall.clone(),   clouds_ground.clone()),    // Water
        (forest_wall.clone(),   grasmaze_ground.clone()),  // Plant
        (rockcave_wall.clone(), rockcave_ground.clone()),  // Electric
        (rockcave_wall.clone(), rockcave_ground.clone()),  // Earth
        (clouds_wall.clone(),   clouds_ground.clone()),    // Wind
        (lavacave_wall.clone(), rockcave_ground.clone()),  // Shadow
        (clouds_wall.clone(),   grassland_ground.clone()), // Light
    ];

    let effects = [
        // Normal
        [images.add(decode_png(NORMAL_01)), images.add(decode_png(NORMAL_02)), images.add(decode_png(NORMAL_03))],
        // Fire
        [images.add(decode_png(FIRE_01)),   images.add(decode_png(FIRE_02)),   images.add(decode_png(FIRE_03))],
        // Water
        [images.add(decode_png(WATER_01)),  images.add(decode_png(WATER_02)),  images.add(decode_png(WATER_03))],
        // Plant
        [images.add(decode_png(PLANT_01)),  images.add(decode_png(PLANT_02)),  images.add(decode_png(PLANT_03))],
        // Electric
        [images.add(decode_png(ELECTRIC_01)), images.add(decode_png(ELECTRIC_02)), images.add(decode_png(ELECTRIC_03))],
        // Earth
        [images.add(decode_png(EARTH_01)),  images.add(decode_png(EARTH_02)),  images.add(decode_png(EARTH_03))],
        // Wind
        [images.add(decode_png(WIND_01)),   images.add(decode_png(WIND_02)),   images.add(decode_png(WIND_03))],
        // Shadow
        [images.add(decode_png(SHADOW_01)), images.add(decode_png(SHADOW_02)), images.add(decode_png(SHADOW_03))],
        // Light
        [images.add(decode_png(LIGHT_01)),  images.add(decode_png(LIGHT_02)),  images.add(decode_png(LIGHT_03))],
    ];

    log::info!("BattleImages chargées depuis le binaire ({} battlebacks, {} effets)",
        battlebacks.len(), effects.iter().map(|e| e.len()).sum::<usize>());
    commands.insert_resource(BattleImages { battlebacks, effects });
}
