//! Sprites de type Ombre — Spectre et ses variantes.

use monster_battle_core::types::ElementType;

pub fn sprite(secondary: Option<ElementType>) -> [&'static str; 5] {
    match secondary {
        // Ombre pure — Spectre
        None => [
            r"    .~~~.     ",
            r"   / o o \    ",
            r"   | ~~~ |    ",
            r"    \   /     ",
            r"     ~^~      ",
        ],
        // Ombre + Feu — Spectre infernal
        Some(ElementType::Fire) => [
            r"   ^.~~~.^    ",
            r"   / o o \    ",
            r"   |^~~~^|    ",
            r"    \^^^/     ",
            r"    ^^~^^     ",
        ],
        // Ombre + Eau — Esprit noyé
        Some(ElementType::Water) => [
            r"   ~.~~~.~    ",
            r"   / o o \    ",
            r"   |~~~~~ |   ",
            r"    \~~~/     ",
            r"    ~~^~~     ",
        ],
        // Ombre + Plante — Ronce morte
        Some(ElementType::Plant) => [
            r"   @.~~~.@    ",
            r"   /@o o@\    ",
            r"   |@~~~@|    ",
            r"    \@@@/     ",
            r"    @~^~@     ",
        ],
        // Ombre + Électrique — Poltergeist
        Some(ElementType::Electric) => [
            r"   Z.~~~.Z    ",
            r"   / o o \    ",
            r"   |Z~~~Z|    ",
            r"    \ZZZ/     ",
            r"    Z~^~Z     ",
        ],
        // Ombre + Terre — Golem de tombe
        Some(ElementType::Earth) => [
            r"   [.~~~.]    ",
            r"   [ o o ]    ",
            r"   |[~~~]|    ",
            r"    [   ]     ",
            r"    #~^~#     ",
        ],
        // Ombre + Vent — Fantôme éolien
        Some(ElementType::Wind) => [
            r"  =-.~~~.-=   ",
            r"   / o o \    ",
            r"  =| ~~~ |=  ",
            r"    \===/     ",
            r"   =-~^~-=    ",
        ],
        // Ombre + Lumière — Esprit crépusculaire
        Some(ElementType::Light) => [
            r"   *.~~~.*    ",
            r"   / o o \    ",
            r"   |*~~~*|    ",
            r"    \ * /     ",
            r"    *~^~*     ",
        ],
        _ => sprite(None),
    }
}
