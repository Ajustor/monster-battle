//! Données de sprites pixel-art 16×16 partagées entre les clients TUI et Android.
//!
//! Chaque sprite est défini par une grille 16×16 de caractères de palette :
//! - `'M'` = couleur principale du type
//! - `'D'` = couleur foncée
//! - `'A'` = couleur accent
//! - `'W'` = blanc (yeux, reflets)
//! - `'X'` = brun (tronc, os)
//! - `'.'` = transparent (pas de pixel)
//!
//! Les sprites varient selon le stade de vie (`AgeStage`) :
//! - **Baby** : petite silhouette (~6 px), grands yeux
//! - **Young** : taille intermédiaire (~10 px), traits en développement
//! - **Adult** : taille complète, forme finale
//! - **Old** : même sprite qu'Adult, palette assombrie au rendu
//!
//! Les fichiers par élément (`fire.rs`, `water.rs`, etc.) contiennent
//! les constantes de grille organisées par stade de vie.

pub mod earth;
pub mod electric;
pub mod fire;
pub mod light;
pub mod overlays;
pub mod plant;
pub mod shadow;
pub mod water;
pub mod wind;

use monster_battle_core::AgeStage;
use monster_battle_core::types::ElementType;

// ═══════════════════════════════════════════════════════════════════
//  Types et constantes
// ═══════════════════════════════════════════════════════════════════

/// Largeur et hauteur d'un sprite pixel art (en pixels).
pub const PIXEL_SIZE: usize = 16;

/// Grille 16×16 de caractères de palette.
pub type PixelGrid = [&'static str; PIXEL_SIZE];

/// Grille 16×16 possédée, produite par le blending base + overlay secondaire.
///
/// Les caractères majuscules (`M`, `D`, `A`, `W`, `X`) utilisent la palette du
/// type **primaire**, tandis que les minuscules (`m`, `d`, `a`) utilisent la
/// palette du type **secondaire**.
pub type BlendedGrid = [[u8; PIXEL_SIZE]; PIXEL_SIZE];

// ═══════════════════════════════════════════════════════════════════
//  Palettes de couleurs par type élémentaire
// ═══════════════════════════════════════════════════════════════════

/// Palette RGB pour un type élémentaire (composantes 0–255).
pub struct TypePalette {
    pub main: [u8; 3],
    pub dark: [u8; 3],
    pub accent: [u8; 3],
}

/// Retourne la palette de couleurs pour un type élémentaire.
pub fn type_palette(element: ElementType) -> TypePalette {
    match element {
        ElementType::Fire => TypePalette {
            main: [244, 67, 54],
            dark: [198, 40, 40],
            accent: [255, 138, 101],
        },
        ElementType::Water => TypePalette {
            main: [33, 150, 243],
            dark: [21, 101, 192],
            accent: [100, 181, 246],
        },
        ElementType::Plant => TypePalette {
            main: [76, 175, 80],
            dark: [27, 94, 32],
            accent: [165, 214, 167],
        },
        ElementType::Electric => TypePalette {
            main: [255, 235, 59],
            dark: [245, 127, 23],
            accent: [255, 245, 157],
        },
        ElementType::Earth => TypePalette {
            main: [121, 85, 72],
            dark: [62, 39, 35],
            accent: [188, 170, 164],
        },
        ElementType::Wind => TypePalette {
            main: [0, 188, 212],
            dark: [0, 96, 100],
            accent: [128, 222, 234],
        },
        ElementType::Shadow => TypePalette {
            main: [156, 39, 176],
            dark: [74, 20, 140],
            accent: [206, 147, 216],
        },
        ElementType::Light => TypePalette {
            main: [255, 193, 7],
            dark: [255, 111, 0],
            accent: [255, 224, 130],
        },
        ElementType::Normal => TypePalette {
            main: [158, 158, 158],
            dark: [97, 97, 97],
            accent: [224, 224, 224],
        },
    }
}

// ═══════════════════════════════════════════════════════════════════
//  Accesseurs par type + âge
// ═══════════════════════════════════════════════════════════════════

/// Retourne le sprite pixel art **de face** pour un type élémentaire et un stade de vie.
pub fn get_pixel_sprite(
    primary: ElementType,
    _secondary: Option<ElementType>,
    age: AgeStage,
) -> &'static PixelGrid {
    match (primary, age) {
        // Baby
        (ElementType::Fire, AgeStage::Baby) => &fire::BABY_FRONT,
        (ElementType::Water, AgeStage::Baby) => &water::BABY_FRONT,
        (ElementType::Plant, AgeStage::Baby) => &plant::BABY_FRONT,
        (ElementType::Electric, AgeStage::Baby) => &electric::BABY_FRONT,
        (ElementType::Earth, AgeStage::Baby) => &earth::BABY_FRONT,
        (ElementType::Wind, AgeStage::Baby) => &wind::BABY_FRONT,
        (ElementType::Shadow, AgeStage::Baby) => &shadow::BABY_FRONT,
        (ElementType::Light, AgeStage::Baby) => &light::BABY_FRONT,
        (ElementType::Normal, AgeStage::Baby) => &fire::BABY_FRONT,
        // Young
        (ElementType::Fire, AgeStage::Young) => &fire::YOUNG_FRONT,
        (ElementType::Water, AgeStage::Young) => &water::YOUNG_FRONT,
        (ElementType::Plant, AgeStage::Young) => &plant::YOUNG_FRONT,
        (ElementType::Electric, AgeStage::Young) => &electric::YOUNG_FRONT,
        (ElementType::Earth, AgeStage::Young) => &earth::YOUNG_FRONT,
        (ElementType::Wind, AgeStage::Young) => &wind::YOUNG_FRONT,
        (ElementType::Shadow, AgeStage::Young) => &shadow::YOUNG_FRONT,
        (ElementType::Light, AgeStage::Young) => &light::YOUNG_FRONT,
        (ElementType::Normal, AgeStage::Young) => &fire::YOUNG_FRONT,
        // Adult & Old (Old uses same sprite with palette change at render time)
        (ElementType::Fire, _) => &fire::ADULT_FRONT,
        (ElementType::Water, _) => &water::ADULT_FRONT,
        (ElementType::Plant, _) => &plant::ADULT_FRONT,
        (ElementType::Electric, _) => &electric::ADULT_FRONT,
        (ElementType::Earth, _) => &earth::ADULT_FRONT,
        (ElementType::Wind, _) => &wind::ADULT_FRONT,
        (ElementType::Shadow, _) => &shadow::ADULT_FRONT,
        (ElementType::Light, _) => &light::ADULT_FRONT,
        (ElementType::Normal, _) => &fire::ADULT_FRONT,
    }
}

/// Retourne le sprite pixel art **de dos** pour un type élémentaire et un stade de vie.
pub fn get_pixel_back_sprite(
    primary: ElementType,
    _secondary: Option<ElementType>,
    age: AgeStage,
) -> &'static PixelGrid {
    match (primary, age) {
        // Baby
        (ElementType::Fire, AgeStage::Baby) => &fire::BABY_BACK,
        (ElementType::Water, AgeStage::Baby) => &water::BABY_BACK,
        (ElementType::Plant, AgeStage::Baby) => &plant::BABY_BACK,
        (ElementType::Electric, AgeStage::Baby) => &electric::BABY_BACK,
        (ElementType::Earth, AgeStage::Baby) => &earth::BABY_BACK,
        (ElementType::Wind, AgeStage::Baby) => &wind::BABY_BACK,
        (ElementType::Shadow, AgeStage::Baby) => &shadow::BABY_BACK,
        (ElementType::Light, AgeStage::Baby) => &light::BABY_BACK,
        (ElementType::Normal, AgeStage::Baby) => &fire::BABY_BACK,
        // Young
        (ElementType::Fire, AgeStage::Young) => &fire::YOUNG_BACK,
        (ElementType::Water, AgeStage::Young) => &water::YOUNG_BACK,
        (ElementType::Plant, AgeStage::Young) => &plant::YOUNG_BACK,
        (ElementType::Electric, AgeStage::Young) => &electric::YOUNG_BACK,
        (ElementType::Earth, AgeStage::Young) => &earth::YOUNG_BACK,
        (ElementType::Wind, AgeStage::Young) => &wind::YOUNG_BACK,
        (ElementType::Shadow, AgeStage::Young) => &shadow::YOUNG_BACK,
        (ElementType::Light, AgeStage::Young) => &light::YOUNG_BACK,
        (ElementType::Normal, AgeStage::Young) => &fire::YOUNG_BACK,
        // Adult & Old
        (ElementType::Fire, _) => &fire::ADULT_BACK,
        (ElementType::Water, _) => &water::ADULT_BACK,
        (ElementType::Plant, _) => &plant::ADULT_BACK,
        (ElementType::Electric, _) => &electric::ADULT_BACK,
        (ElementType::Earth, _) => &earth::ADULT_BACK,
        (ElementType::Wind, _) => &wind::ADULT_BACK,
        (ElementType::Shadow, _) => &shadow::ADULT_BACK,
        (ElementType::Light, _) => &light::ADULT_BACK,
        (ElementType::Normal, _) => &fire::ADULT_BACK,
    }
}

// ═══════════════════════════════════════════════════════════════════
//  Overlays secondaires et blending
// ═══════════════════════════════════════════════════════════════════

/// Retourne l'overlay décoratif pour un élément utilisé comme type secondaire.
pub fn get_secondary_overlay(
    element: ElementType,
    age: AgeStage,
) -> &'static overlays::OverlayGrid {
    use overlays::*;
    match (element, age) {
        (ElementType::Fire, AgeStage::Baby) => &FIRE_BABY,
        (ElementType::Fire, AgeStage::Young) => &FIRE_YOUNG,
        (ElementType::Fire, _) => &FIRE_ADULT,

        (ElementType::Water, AgeStage::Baby) => &WATER_BABY,
        (ElementType::Water, AgeStage::Young) => &WATER_YOUNG,
        (ElementType::Water, _) => &WATER_ADULT,

        (ElementType::Plant, AgeStage::Baby) => &PLANT_BABY,
        (ElementType::Plant, AgeStage::Young) => &PLANT_YOUNG,
        (ElementType::Plant, _) => &PLANT_ADULT,

        (ElementType::Electric, AgeStage::Baby) => &ELECTRIC_BABY,
        (ElementType::Electric, AgeStage::Young) => &ELECTRIC_YOUNG,
        (ElementType::Electric, _) => &ELECTRIC_ADULT,

        (ElementType::Earth, AgeStage::Baby) => &EARTH_BABY,
        (ElementType::Earth, AgeStage::Young) => &EARTH_YOUNG,
        (ElementType::Earth, _) => &EARTH_ADULT,

        (ElementType::Wind, AgeStage::Baby) => &WIND_BABY,
        (ElementType::Wind, AgeStage::Young) => &WIND_YOUNG,
        (ElementType::Wind, _) => &WIND_ADULT,

        (ElementType::Shadow, AgeStage::Baby) => &SHADOW_BABY,
        (ElementType::Shadow, AgeStage::Young) => &SHADOW_YOUNG,
        (ElementType::Shadow, _) => &SHADOW_ADULT,

        (ElementType::Light, AgeStage::Baby) => &LIGHT_BABY,
        (ElementType::Light, AgeStage::Young) => &LIGHT_YOUNG,
        (ElementType::Light, _) => &LIGHT_ADULT,

        (ElementType::Normal, _) => &EMPTY,
    }
}

/// Convertit une `PixelGrid` statique en `BlendedGrid` (sans overlay).
pub fn base_to_blended(grid: &PixelGrid) -> BlendedGrid {
    let mut out = [[b'.'; PIXEL_SIZE]; PIXEL_SIZE];
    for (r, row) in grid.iter().enumerate() {
        let bytes = row.as_bytes();
        for (c, byte) in bytes.iter().enumerate().take(PIXEL_SIZE) {
            out[r][c] = *byte;
        }
    }
    out
}

/// Fusionne un sprite de base avec un overlay de type secondaire.
///
/// L'overlay n'écrit que dans les pixels transparents (`.`) du sprite de base.
/// Les caractères minuscules (`m`, `d`, `a`) de l'overlay sont conservés tels
/// quels dans le résultat ; ils seront résolus avec la palette du type secondaire
/// par le moteur de rendu.
pub fn blend_sprite(base: &PixelGrid, overlay: &overlays::OverlayGrid) -> BlendedGrid {
    let mut out = [[b'.'; PIXEL_SIZE]; PIXEL_SIZE];
    for r in 0..PIXEL_SIZE {
        let base_bytes = base[r].as_bytes();
        let over_bytes = overlay[r].as_bytes();
        for c in 0..PIXEL_SIZE {
            let base_ch = if c < base_bytes.len() {
                base_bytes[c]
            } else {
                b'.'
            };
            let over_ch = if c < over_bytes.len() {
                over_bytes[c]
            } else {
                b'.'
            };
            out[r][c] = if base_ch == b'.' && over_ch != b'.' {
                over_ch // overlay visible uniquement sur un pixel transparent
            } else {
                base_ch
            };
        }
    }
    out
}

/// Construit un sprite fusionné prêt au rendu (face avant).
///
/// Si le monstre a un type secondaire différent du primaire, l'overlay
/// correspondant est appliqué sur le sprite de base.
pub fn get_blended_sprite(
    primary: ElementType,
    secondary: Option<ElementType>,
    age: AgeStage,
) -> BlendedGrid {
    let base = get_pixel_sprite(primary, secondary, age);
    match secondary {
        Some(sec) if sec != primary => {
            let overlay = get_secondary_overlay(sec, age);
            blend_sprite(base, overlay)
        }
        _ => base_to_blended(base),
    }
}

/// Construit un sprite fusionné prêt au rendu (face arrière).
pub fn get_blended_back_sprite(
    primary: ElementType,
    secondary: Option<ElementType>,
    age: AgeStage,
) -> BlendedGrid {
    let base = get_pixel_back_sprite(primary, secondary, age);
    match secondary {
        Some(sec) if sec != primary => {
            let overlay = get_secondary_overlay(sec, age);
            blend_sprite(base, overlay)
        }
        _ => base_to_blended(base),
    }
}
