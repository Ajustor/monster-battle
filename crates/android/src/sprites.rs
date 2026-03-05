//! Conversion des sprites pixel-art 16×16 en textures Bevy.
//!
//! Les données de grille (sprites) et les palettes de couleurs sont définies dans
//! la crate partagée `monster_battle_sprites`. Ce module ne contient que le code
//! de rendu spécifique à Bevy (conversion en `Image`, atlas de cache, etc.).

use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use monster_battle_core::AgeStage;
use monster_battle_core::types::ElementType;

// Ré-exports de la crate sprites partagée pour garder la compatibilité des imports.
pub use monster_battle_sprites::{
    BlendedGrid, PIXEL_SIZE, PixelGrid, TypePalette, get_blended_back_sprite, get_blended_sprite,
    get_pixel_back_sprite, get_pixel_sprite, type_palette,
};

/// Plugin de gestion des sprites.
pub struct SpritePlugin;

impl Plugin for SpritePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MonsterSpriteAtlas>();
    }
}

/// Facteur d'échelle pour le rendu mobile (16×16 → 64×64 par exemple).
pub const SCALE_FACTOR: usize = 4;

/// Résout un caractère de palette en [R, G, B, A].
/// Les majuscules utilisent la palette primaire, les minuscules la palette secondaire.
fn resolve_pixel(ch: u8, primary: &TypePalette, secondary: &TypePalette) -> [u8; 4] {
    match ch {
        b'M' => [primary.main[0], primary.main[1], primary.main[2], 255],
        b'D' => [primary.dark[0], primary.dark[1], primary.dark[2], 255],
        b'A' => [primary.accent[0], primary.accent[1], primary.accent[2], 255],
        b'm' => [secondary.main[0], secondary.main[1], secondary.main[2], 255],
        b'd' => [secondary.dark[0], secondary.dark[1], secondary.dark[2], 255],
        b'a' => [
            secondary.accent[0],
            secondary.accent[1],
            secondary.accent[2],
            255,
        ],
        b'W' => [255, 255, 255, 255],
        b'X' => [139, 90, 43, 255],
        b'.' => [0, 0, 0, 0],
        _ => [0, 0, 0, 0],
    }
}

/// Assombrit une composante couleur par un facteur (0.0 – 1.0).
fn darken_u8(v: u8, factor: f64) -> u8 {
    (v as f64 * factor) as u8
}

/// Résout un caractère de palette avec assombrissement pour les vieux monstres.
fn resolve_pixel_old(ch: u8, primary: &TypePalette, secondary: &TypePalette) -> [u8; 4] {
    match ch {
        b'M' => {
            let [r, g, b] = primary.main;
            [darken_u8(r, 0.7), darken_u8(g, 0.7), darken_u8(b, 0.7), 255]
        }
        b'D' => {
            let [r, g, b] = primary.dark;
            [darken_u8(r, 0.7), darken_u8(g, 0.7), darken_u8(b, 0.7), 255]
        }
        b'A' => {
            let [r, g, b] = primary.accent;
            [darken_u8(r, 0.5), darken_u8(g, 0.5), darken_u8(b, 0.5), 255]
        }
        b'm' => {
            let [r, g, b] = secondary.main;
            [darken_u8(r, 0.7), darken_u8(g, 0.7), darken_u8(b, 0.7), 255]
        }
        b'd' => {
            let [r, g, b] = secondary.dark;
            [darken_u8(r, 0.7), darken_u8(g, 0.7), darken_u8(b, 0.7), 255]
        }
        b'a' => {
            let [r, g, b] = secondary.accent;
            [darken_u8(r, 0.5), darken_u8(g, 0.5), darken_u8(b, 0.5), 255]
        }
        b'W' => [200, 200, 200, 255],
        b'X' => [100, 65, 30, 255],
        b'.' => [0, 0, 0, 0],
        _ => [0, 0, 0, 0],
    }
}

// ═══════════════════════════════════════════════════════════════════
//  Conversion PixelGrid → Image Bevy
// ═══════════════════════════════════════════════════════════════════

/// Convertit une grille blendée 16×16 en `Image` Bevy RGBA8.
/// Pour `AgeStage::Old`, les couleurs sont automatiquement assombries.
/// Les caractères minuscules (overlay secondaire) utilisent `sec_palette`.
pub fn pixel_grid_to_image(
    grid: &BlendedGrid,
    element: ElementType,
    secondary: Option<ElementType>,
    age: AgeStage,
) -> Image {
    let palette = type_palette(element);
    let sec_palette = secondary
        .map(|e| type_palette(e))
        .unwrap_or_else(|| type_palette(element));
    let resolver: fn(u8, &TypePalette, &TypePalette) -> [u8; 4] = if matches!(age, AgeStage::Old) {
        resolve_pixel_old
    } else {
        resolve_pixel
    };
    let size = PIXEL_SIZE;
    let mut data = Vec::with_capacity(size * size * 4);

    for row in grid.iter() {
        for col in 0..size {
            let ch = row[col];
            let rgba = resolver(ch, &palette, &sec_palette);
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

/// Convertit une grille blendée en `Image` Bevy avec flash rouge (hit).
pub fn pixel_grid_to_image_hit(grid: &BlendedGrid) -> Image {
    let red_palette = TypePalette {
        main: [255, 60, 60],
        dark: [180, 30, 30],
        accent: [255, 120, 120],
    };
    let size = PIXEL_SIZE;
    let mut data = Vec::with_capacity(size * size * 4);

    for row in grid.iter() {
        for col in 0..size {
            let ch = row[col];
            let rgba = resolve_pixel(ch, &red_palette, &red_palette);
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
    /// Sprites de face par type : `(primary, secondary, age) → Handle<Image>`
    pub front_sprites: Vec<(ElementType, Option<ElementType>, AgeStage, Handle<Image>)>,
    /// Sprites de dos par type.
    pub back_sprites: Vec<(ElementType, Option<ElementType>, AgeStage, Handle<Image>)>,
}

impl MonsterSpriteAtlas {
    /// Récupère (ou crée) le handle de texture pour un monstre de face.
    pub fn get_or_create_front(
        &mut self,
        primary: ElementType,
        secondary: Option<ElementType>,
        age: AgeStage,
        grid: &BlendedGrid,
        images: &mut Assets<Image>,
    ) -> Handle<Image> {
        // Chercher dans le cache
        if let Some((_, _, _, handle)) = self
            .front_sprites
            .iter()
            .find(|(p, s, a, _)| *p == primary && *s == secondary && *a == age)
        {
            return handle.clone();
        }

        // Créer la texture
        let image = pixel_grid_to_image(grid, primary, secondary, age);
        let handle = images.add(image);
        self.front_sprites
            .push((primary, secondary, age, handle.clone()));
        handle
    }

    /// Récupère (ou crée) le handle de texture pour un monstre de dos.
    pub fn get_or_create_back(
        &mut self,
        primary: ElementType,
        secondary: Option<ElementType>,
        age: AgeStage,
        grid: &BlendedGrid,
        images: &mut Assets<Image>,
    ) -> Handle<Image> {
        if let Some((_, _, _, handle)) = self
            .back_sprites
            .iter()
            .find(|(p, s, a, _)| *p == primary && *s == secondary && *a == age)
        {
            return handle.clone();
        }

        let image = pixel_grid_to_image(grid, primary, secondary, age);
        let handle = images.add(image);
        self.back_sprites
            .push((primary, secondary, age, handle.clone()));
        handle
    }
}
