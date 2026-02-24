//! Sprites de type Terre — Golem et ses variantes.

use monster_battle_core::types::ElementType;

pub fn sprite(secondary: Option<ElementType>) -> [&'static str; 5] {
    match secondary {
        // Terre pure — Golem de pierre
        None => [
            r"   [=====]    ",
            r"   | o o |    ",
            r"   |[===]|    ",
            r"   | | | |    ",
            r"   [_] [_]    ",
        ],
        // Terre + Feu — Golem de magma
        Some(ElementType::Fire) => [
            r"  ^[=====]^   ",
            r"   | o o |    ",
            r"   |[^^^]|    ",
            r"   |^| |^|    ",
            r"  ^[_]^[_]^   ",
        ],
        // Terre + Eau — Titan de boue
        Some(ElementType::Water) => [
            r"  ~[=====]~   ",
            r"   | o o |    ",
            r"   |[~~~]|    ",
            r"   |~| |~|    ",
            r"  ~[_] [_]~   ",
        ],
        // Terre + Plante — Colosse de mousse
        Some(ElementType::Plant) => [
            r"  @[=====]@   ",
            r"   |@o o@|    ",
            r"   |[@@@]|    ",
            r"   |@| |@|    ",
            r"  @[_]@[_]@   ",
        ],
        // Terre + Électrique — Mech de cristal
        Some(ElementType::Electric) => [
            r"  Z[=====]Z   ",
            r"   | o o |    ",
            r"   |[ZZZ]|    ",
            r"   |Z| |Z|    ",
            r"  Z[_] [_]Z   ",
        ],
        // Terre + Vent — Djinn de sable
        Some(ElementType::Wind) => [
            r"  =[=====]=   ",
            r"   | o o |    ",
            r"  =|[===]|=   ",
            r"   |=| |=|    ",
            r"  =[_]=[_]=   ",
        ],
        // Terre + Ombre — Chevalier d'obsidienne
        Some(ElementType::Shadow) => [
            r"  .[=====].   ",
            r"   | . . |    ",
            r"   |[...]|    ",
            r"   |.| |.|    ",
            r"  .[_].[_].   ",
        ],
        // Terre + Lumière — Gardien de diamant
        Some(ElementType::Light) => [
            r"  *[=====]*   ",
            r"   | o o |    ",
            r"   |[***]|    ",
            r"   |*| |*|    ",
            r"  *[_]*[_]*   ",
        ],
        _ => sprite(None),
    }
}
