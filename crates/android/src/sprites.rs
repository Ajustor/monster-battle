//! Conversion des sprites pixel-art 16×16 en textures Bevy.
//!
//! Les données de grille utilisent le même format que la TUI :
//! - `'M'` = couleur principale du type
//! - `'D'` = couleur foncée
//! - `'A'` = couleur accent
//! - `'W'` = blanc (yeux, reflets)
//! - `'X'` = brun (tronc, os)
//! - `'.'` = transparent
//!
//! Les palettes de couleurs sont les mêmes que dans `crates/tui/src/sprites/pixel.rs`.

use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use monster_battle_core::types::ElementType;

/// Plugin de gestion des sprites.
pub struct SpritePlugin;

impl Plugin for SpritePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MonsterSpriteAtlas>();
    }
}

// ═══════════════════════════════════════════════════════════════════
//  Dimensions
// ═══════════════════════════════════════════════════════════════════

/// Largeur et hauteur d'un sprite pixel art (en pixels).
pub const PIXEL_SIZE: usize = 16;

/// Facteur d'échelle pour le rendu mobile (16×16 → 64×64 par exemple).
pub const SCALE_FACTOR: usize = 4;

// ═══════════════════════════════════════════════════════════════════
//  Type PixelGrid — même format que la TUI
// ═══════════════════════════════════════════════════════════════════

/// Grille 16×16 de caractères de palette.
pub type PixelGrid = [&'static str; PIXEL_SIZE];

// ═══════════════════════════════════════════════════════════════════
//  Palettes de couleurs par type élémentaire
// ═══════════════════════════════════════════════════════════════════

/// Palette RGB pour un type élémentaire.
pub struct TypePalette {
    pub main: [u8; 3],
    pub dark: [u8; 3],
    pub accent: [u8; 3],
}

/// Retourne la palette de couleurs pour un type élémentaire.
/// Valeurs identiques à celles de la TUI.
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

/// Résout un caractère de palette en [R, G, B, A].
fn resolve_pixel(ch: u8, palette: &TypePalette) -> [u8; 4] {
    match ch {
        b'M' => [palette.main[0], palette.main[1], palette.main[2], 255],
        b'D' => [palette.dark[0], palette.dark[1], palette.dark[2], 255],
        b'A' => [palette.accent[0], palette.accent[1], palette.accent[2], 255],
        b'W' => [255, 255, 255, 255],
        b'X' => [139, 90, 43, 255], // brun (tronc, os)
        b'.' => [0, 0, 0, 0],       // transparent
        _ => [0, 0, 0, 0],
    }
}

// ═══════════════════════════════════════════════════════════════════
//  Conversion PixelGrid → Image Bevy
// ═══════════════════════════════════════════════════════════════════

/// Convertit une grille pixel-art 16×16 en `Image` Bevy RGBA8.
pub fn pixel_grid_to_image(grid: &PixelGrid, element: ElementType) -> Image {
    let palette = type_palette(element);
    let size = PIXEL_SIZE;
    let mut data = Vec::with_capacity(size * size * 4);

    for row in grid.iter() {
        let bytes = row.as_bytes();
        for col in 0..size {
            let ch = if col < bytes.len() { bytes[col] } else { b'.' };
            let rgba = resolve_pixel(ch, &palette);
            data.extend_from_slice(&rgba);
        }
    }

    Image::new(
        Extent3d {
            width: size as u32,
            height: size as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        default(),
    )
}

/// Convertit une grille pixel-art en `Image` Bevy avec un indicateur de type secondaire.
///
/// Le type secondaire est affiché comme un carré 3×2 en bas à droite (lignes 12-13, cols 13-15).
pub fn pixel_grid_to_image_dual(
    grid: &PixelGrid,
    primary: ElementType,
    secondary: ElementType,
) -> Image {
    let palette = type_palette(primary);
    let sec_palette = type_palette(secondary);
    let size = PIXEL_SIZE;
    let mut data = Vec::with_capacity(size * size * 4);

    for (row_idx, row) in grid.iter().enumerate() {
        let bytes = row.as_bytes();
        for col in 0..size {
            // Zone indicateur secondaire : lignes 12-13, colonnes 13-15
            let in_secondary_zone = row_idx >= 12 && row_idx <= 13 && col >= 13;

            let ch = if col < bytes.len() { bytes[col] } else { b'.' };
            let rgba = if in_secondary_zone {
                [
                    sec_palette.main[0],
                    sec_palette.main[1],
                    sec_palette.main[2],
                    255,
                ]
            } else {
                resolve_pixel(ch, &palette)
            };
            data.extend_from_slice(&rgba);
        }
    }

    Image::new(
        Extent3d {
            width: size as u32,
            height: size as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        default(),
    )
}

/// Convertit une grille pixel-art en `Image` Bevy avec flash rouge (hit).
pub fn pixel_grid_to_image_hit(grid: &PixelGrid) -> Image {
    let red_palette = TypePalette {
        main: [255, 60, 60],
        dark: [180, 30, 30],
        accent: [255, 120, 120],
    };
    let size = PIXEL_SIZE;
    let mut data = Vec::with_capacity(size * size * 4);

    for row in grid.iter() {
        let bytes = row.as_bytes();
        for col in 0..size {
            let ch = if col < bytes.len() { bytes[col] } else { b'.' };
            let rgba = resolve_pixel(ch, &red_palette);
            data.extend_from_slice(&rgba);
        }
    }

    Image::new(
        Extent3d {
            width: size as u32,
            height: size as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        default(),
    )
}

// ═══════════════════════════════════════════════════════════════════
//  Atlas de sprites — cache des textures générées
// ═══════════════════════════════════════════════════════════════════

/// Cache des handles de textures pour chaque type de monstre.
/// Évite de recréer les textures à chaque frame.
#[derive(Resource, Default)]
pub struct MonsterSpriteAtlas {
    /// Sprites de face par type : `(primary, secondary) → Handle<Image>`
    pub front_sprites: Vec<(ElementType, Option<ElementType>, Handle<Image>)>,
    /// Sprites de dos par type.
    pub back_sprites: Vec<(ElementType, Option<ElementType>, Handle<Image>)>,
}

impl MonsterSpriteAtlas {
    /// Récupère (ou crée) le handle de texture pour un monstre de face.
    pub fn get_or_create_front(
        &mut self,
        primary: ElementType,
        secondary: Option<ElementType>,
        grid: &PixelGrid,
        images: &mut Assets<Image>,
    ) -> Handle<Image> {
        // Chercher dans le cache
        if let Some((_, _, handle)) = self
            .front_sprites
            .iter()
            .find(|(p, s, _)| *p == primary && *s == secondary)
        {
            return handle.clone();
        }

        // Créer la texture
        let image = match secondary {
            Some(sec) => pixel_grid_to_image_dual(grid, primary, sec),
            None => pixel_grid_to_image(grid, primary),
        };
        let handle = images.add(image);
        self.front_sprites
            .push((primary, secondary, handle.clone()));
        handle
    }

    /// Récupère (ou crée) le handle de texture pour un monstre de dos.
    pub fn get_or_create_back(
        &mut self,
        primary: ElementType,
        secondary: Option<ElementType>,
        grid: &PixelGrid,
        images: &mut Assets<Image>,
    ) -> Handle<Image> {
        if let Some((_, _, handle)) = self
            .back_sprites
            .iter()
            .find(|(p, s, _)| *p == primary && *s == secondary)
        {
            return handle.clone();
        }

        let image = match secondary {
            Some(sec) => pixel_grid_to_image_dual(grid, primary, sec),
            None => pixel_grid_to_image(grid, primary),
        };
        let handle = images.add(image);
        self.back_sprites.push((primary, secondary, handle.clone()));
        handle
    }
}

// ═══════════════════════════════════════════════════════════════════
//  Données des sprites — grilles 16×16 identiques à la TUI
// ═══════════════════════════════════════════════════════════════════

// ── Feu — Dragon ────────────────────────────────────────────────

pub const FIRE_FRONT: PixelGrid = [
    "................",
    "....A......A....",
    "....AMMMMMMA....",
    ".....MWMMWM.....",
    "....AMMMMMM.....",
    ".....MMDDMM.....",
    "...DD.MMMM.DD...",
    "...DDDMAAMDDD...",
    "...DDDMAAMDDDA..",
    "......MMMM.MM...",
    "......M..M......",
    "......M..M......",
    ".....DD..DD.....",
    "................",
    "................",
    "................",
];

pub const FIRE_BACK: PixelGrid = [
    "................",
    "....A......A....",
    "....AMMMMMMA....",
    ".....MMMMMM.....",
    ".....MMMMMM.....",
    ".....MMAAMM.....",
    "...DD.MMMM.DD...",
    "...DDDMAAMDDD...",
    "...DDDMAAMDDDA..",
    "......MMMM.MM...",
    "......M..M......",
    "......M..M......",
    ".....DD..DD.....",
    "................",
    "................",
    "................",
];

// ── Eau — Serpent ───────────────────────────────────────────────

pub const WATER_FRONT: PixelGrid = [
    "................",
    "................",
    "....AA..........",
    "....MMMM........",
    "....MWMD....A...",
    "..A.MMMMMM......",
    ".......MAA......",
    ".....MMMM.......",
    ".....MAAM.......",
    "........MMM.A...",
    "...A....MMMDD...",
    "................",
    "...DD...........",
    ".......DDD......",
    "................",
    "................",
];

pub const WATER_BACK: PixelGrid = [
    "................",
    "................",
    "....AA..........",
    "....MMMM........",
    "....MMMM........",
    "....MMMMMM......",
    ".......MAA......",
    ".....MMMM.......",
    ".....MAAM.......",
    "........MMM.A...",
    "...A....MMMDD...",
    "................",
    "...DD...........",
    ".......DDD......",
    "................",
    "................",
];

// ── Plante — Arbre ──────────────────────────────────────────────

pub const PLANT_FRONT: PixelGrid = [
    "................",
    "......AAA.......",
    "....AAMMMMAA....",
    "...AMMWMMWMMA...",
    "...AMMMMMMMA....",
    "....AMMDDMMA....",
    "....AAMMMMAA....",
    "......XMMX......",
    "......XMMX......",
    ".....XXMMXX.....",
    "......XMMX......",
    "......X..X......",
    "....DDD..DDD....",
    "................",
    "................",
    "................",
];

pub const PLANT_BACK: PixelGrid = [
    "................",
    "......AAA.......",
    "....AAMMMMAA....",
    "...AMMMMMMMMA...",
    "...AMMMMMMMMA...",
    "....AMMMMMMA....",
    "....AAMMMMAA....",
    "......XMMX......",
    "......XMMX......",
    ".....XXMMXX.....",
    "......XMMX......",
    "......X..X......",
    "....DDD..DDD....",
    "................",
    "................",
    "................",
];

// ── Électrique — Lézard ─────────────────────────────────────────

pub const ELECTRIC_FRONT: PixelGrid = [
    "................",
    "...AA....AA.....",
    "...AMMMMMMA.....",
    "....MWMMWM......",
    "...AMMMMMMA.....",
    "....MMDDMM......",
    "....MMMMMMAA....",
    "..DDMAAMD.......",
    "....MAAMD.......",
    "....MMMM........",
    "....M..M...AA...",
    "....M..M........",
    "...DD..DD.......",
    "................",
    "................",
    "................",
];

pub const ELECTRIC_BACK: PixelGrid = [
    "................",
    "...AA....AA.....",
    "...AMMMMMMA.....",
    "....MMMMMM......",
    "...AMMMMMMA.....",
    "....MMMMMM......",
    "....MMMMMMAA....",
    "..DDMAAMD.......",
    "....MAAMD.......",
    "....MMMM........",
    "....M..M...AA...",
    "....M..M........",
    "...DD..DD.......",
    "................",
    "................",
    "................",
];

// ── Terre — Golem ───────────────────────────────────────────────

pub const EARTH_FRONT: PixelGrid = [
    "................",
    "....DDDDDD......",
    "...DMMMMMMD.....",
    "...DMWMMWMD.....",
    "...DMMMMMMD.....",
    "...DMMDDMMD.....",
    "....DMMMMD......",
    "..MMMMAAMMM.....",
    "..MMMMAAMMM.....",
    "..MMMMMMMMM.....",
    "....MMMMMM......",
    "....MM..MM......",
    "...DDD..DDD.....",
    "................",
    "................",
    "................",
];

pub const EARTH_BACK: PixelGrid = [
    "................",
    "....DDDDDD......",
    "...DMMMMMMD.....",
    "...DMMMMMMD.....",
    "...DMMMMMMD.....",
    "...DMMMMMMD.....",
    "....DMMMMD......",
    "..MMMMAAMMM.....",
    "..MMMMAAMMM.....",
    "..MMMMMMMMM.....",
    "....MMMMMM......",
    "....MM..MM......",
    "...DDD..DDD.....",
    "................",
    "................",
    "................",
];

// ── Vent — Oiseau ───────────────────────────────────────────────

pub const WIND_FRONT: PixelGrid = [
    "................",
    "......MMM.......",
    ".....MMMMM......",
    ".....MWMWM......",
    "......MMMM......",
    "......MDDM......",
    "..AAAMMMMMAA....",
    ".AAAMMAAMMMAAA..",
    "..AAAMMMMMAAA...",
    "......MAAM......",
    "......MMMM......",
    "......M..M......",
    ".....DD..DD.....",
    "................",
    "................",
    "................",
];

pub const WIND_BACK: PixelGrid = [
    "................",
    "......MMM.......",
    ".....MMMMM......",
    ".....MMMMM......",
    "......MMMM......",
    "......MMMM......",
    "..AAAMMMMMAA....",
    ".AAAMMAAMMMAAA..",
    "..AAAMMMMMAAA...",
    "......MAAM......",
    "......MMMM......",
    "......M..M......",
    ".....DD..DD.....",
    "................",
    "................",
    "................",
];

// ── Ombre — Spectre ─────────────────────────────────────────────

pub const SHADOW_FRONT: PixelGrid = [
    "................",
    ".....AAAA.......",
    "....AMMMMMA.....",
    "...AMWMMWMMA....",
    "...AMMMMMMA.....",
    "....AMDDMA......",
    "....AMMMMMA.....",
    "...AAAMMMAAA....",
    "..AA..MMM..AA...",
    "......MMM.......",
    ".....DMMMD......",
    "....DD.M.DD.....",
    "...DD..M..DD....",
    "................",
    "................",
    "................",
];

pub const SHADOW_BACK: PixelGrid = [
    "................",
    ".....AAAA.......",
    "....AMMMMMA.....",
    "...AMMMMMMA.....",
    "...AMMMMMMA.....",
    "....AMMMMMA.....",
    "....AMMMMMA.....",
    "...AAAMMMAAA....",
    "..AA..MMM..AA...",
    "......MMM.......",
    ".....DMMMD......",
    "....DD.M.DD.....",
    "...DD..M..DD....",
    "................",
    "................",
    "................",
];

// ── Lumière — Ange ──────────────────────────────────────────────

pub const LIGHT_FRONT: PixelGrid = [
    "......AAA.......",
    ".....A.A.A......",
    "......AAA.......",
    ".....MMMMM......",
    "....MWMMMWM.....",
    "....MMMMMMM.....",
    ".AA.MMDDMMM.AA..",
    ".AAAMMAAMMAAA...",
    "..AAMMAAMMAAA...",
    ".....MMMMMM.....",
    ".....MMMM.......",
    "......M..M......",
    ".....DD..DD.....",
    "................",
    "................",
    "................",
];

pub const LIGHT_BACK: PixelGrid = [
    "......AAA.......",
    ".....A.A.A......",
    "......AAA.......",
    ".....MMMMM......",
    "....MMMMMMM.....",
    "....MMMMMMM.....",
    ".AA.MMMMMMM.AA..",
    ".AAAMMAAMMAAA...",
    "..AAMMAAMMAAA...",
    ".....MMMMMM.....",
    ".....MMMM.......",
    "......M..M......",
    ".....DD..DD.....",
    "................",
    "................",
    "................",
];

// ═══════════════════════════════════════════════════════════════════
//  Accesseurs par type
// ═══════════════════════════════════════════════════════════════════

/// Retourne le sprite pixel art de face pour un type élémentaire.
pub fn get_pixel_sprite(
    primary: ElementType,
    _secondary: Option<ElementType>,
) -> &'static PixelGrid {
    match primary {
        ElementType::Fire => &FIRE_FRONT,
        ElementType::Water => &WATER_FRONT,
        ElementType::Plant => &PLANT_FRONT,
        ElementType::Electric => &ELECTRIC_FRONT,
        ElementType::Earth => &EARTH_FRONT,
        ElementType::Wind => &WIND_FRONT,
        ElementType::Shadow => &SHADOW_FRONT,
        ElementType::Light => &LIGHT_FRONT,
        ElementType::Normal => &FIRE_FRONT,
    }
}

/// Retourne le sprite pixel art de dos pour un type élémentaire.
pub fn get_pixel_back_sprite(
    primary: ElementType,
    _secondary: Option<ElementType>,
) -> &'static PixelGrid {
    match primary {
        ElementType::Fire => &FIRE_BACK,
        ElementType::Water => &WATER_BACK,
        ElementType::Plant => &PLANT_BACK,
        ElementType::Electric => &ELECTRIC_BACK,
        ElementType::Earth => &EARTH_BACK,
        ElementType::Wind => &WIND_BACK,
        ElementType::Shadow => &SHADOW_BACK,
        ElementType::Light => &LIGHT_BACK,
        ElementType::Normal => &FIRE_BACK,
    }
}
