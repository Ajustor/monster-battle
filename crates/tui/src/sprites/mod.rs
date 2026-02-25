//! Sprites ASCII art pour chaque combinaison de types élémentaires.
//!
//! Chaque monstre possède un type primaire et optionnellement un type secondaire.
//! 8 types × (1 pur + 7 duaux) = 64 sprites uniques.

mod earth;
mod electric;
mod fire;
mod light;
mod plant;
mod shadow;
mod water;
mod wind;

use monster_battle_core::types::ElementType;

/// Nombre de lignes par sprite.
pub const SPRITE_HEIGHT: usize = 5;

/// Retourne un sprite ASCII art de 5 lignes pour un monstre selon ses types.
pub fn get_sprite(
    primary: ElementType,
    secondary: Option<ElementType>,
) -> [&'static str; SPRITE_HEIGHT] {
    match primary {
        ElementType::Fire => fire::sprite(secondary),
        ElementType::Water => water::sprite(secondary),
        ElementType::Plant => plant::sprite(secondary),
        ElementType::Electric => electric::sprite(secondary),
        ElementType::Earth => earth::sprite(secondary),
        ElementType::Wind => wind::sprite(secondary),
        ElementType::Shadow => shadow::sprite(secondary),
        ElementType::Light => light::sprite(secondary),
        ElementType::Normal => fire::sprite(secondary), // fallback
    }
}

/// Retourne un sprite ASCII art de dos (5 lignes) pour un monstre vu de derrière.
/// Utilisé côté joueur en combat, comme dans Pokémon.
pub fn get_back_sprite(
    primary: ElementType,
    secondary: Option<ElementType>,
) -> [&'static str; SPRITE_HEIGHT] {
    match primary {
        ElementType::Fire => fire::back_sprite(secondary),
        ElementType::Water => water::back_sprite(secondary),
        ElementType::Plant => plant::back_sprite(secondary),
        ElementType::Electric => electric::back_sprite(secondary),
        ElementType::Earth => earth::back_sprite(secondary),
        ElementType::Wind => wind::back_sprite(secondary),
        ElementType::Shadow => shadow::back_sprite(secondary),
        ElementType::Light => light::back_sprite(secondary),
        ElementType::Normal => fire::back_sprite(secondary), // fallback
    }
}
