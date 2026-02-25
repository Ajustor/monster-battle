//! Sprites de type Eau — Serpent marin et ses variantes.

use monster_battle_core::types::ElementType;

pub fn sprite(secondary: Option<ElementType>) -> [&'static str; 5] {
    match secondary {
        // Eau pure — Serpent marin
        None => [
            r"      ___     ",
            r"    /o   \    ",
            r"   | ~~~~ >   ",
            r"    \o___/    ",
            r"   ~~    ~~   ",
        ],
        // Eau + Feu — Crabe geyser
        Some(ElementType::Fire) => [
            r"     ^___^    ",
            r"    /o   \    ",
            r"   |^^~~^^>   ",
            r"    \o___/    ",
            r"   ^~    ~^   ",
        ],
        // Eau + Plante — Grenouille nénuphar
        Some(ElementType::Plant) => [
            r"    @_____@   ",
            r"   / o  o  \  ",
            r"   | @~~@  |  ",
            r"   \_@__@_/   ",
            r"   @~    ~@   ",
        ],
        // Eau + Électrique — Anguille électrique
        Some(ElementType::Electric) => [
            r"    Z/===\Z   ",
            r"   / o    \   ",
            r"   |Z~~~~Z>   ",
            r"    \_Z__/    ",
            r"   Z~    ~Z   ",
        ],
        // Eau + Terre — Tortue de boue
        Some(ElementType::Earth) => [
            r"    [_____]   ",
            r"   /[o   ]    ",
            r"   |[~~~~]>   ",
            r"    [o___]    ",
            r"   ##    ##   ",
        ],
        // Eau + Vent — Serpent tempête
        Some(ElementType::Wind) => [
            r"   =-_____-=  ",
            r"    /o   \    ",
            r"  =| ~~~~ >=  ",
            r"    \o___/    ",
            r"  =-~    ~-=  ",
        ],
        // Eau + Ombre — Poisson abyssal
        Some(ElementType::Shadow) => [
            r"    .___.     ",
            r"    /.   \    ",
            r"   |..~~..|   ",
            r"    \.___./   ",
            r"   ..    ..   ",
        ],
        // Eau + Lumière — Serpent cristallin
        Some(ElementType::Light) => [
            r"    *___*     ",
            r"    /o   \    ",
            r"   |*~~~~*>   ",
            r"    \o___/    ",
            r"   *~  * ~*   ",
        ],
        _ => sprite(None),
    }
}

pub fn back_sprite(secondary: Option<ElementType>) -> [&'static str; 5] {
    match secondary {
        // Eau pure (dos)
        None => [
            r"      ___     ",
            r"    / ~~~ \   ",
            r"   |  ^^  |   ",
            r"    \____/    ",
            r"   ~~    ~~   ",
        ],
        // Eau + Feu (dos)
        Some(ElementType::Fire) => [
            r"     ^___^    ",
            r"    / ~~~ \   ",
            r"   |^^~~^^|   ",
            r"    \^^^^/    ",
            r"   ^~    ~^   ",
        ],
        // Eau + Plante (dos)
        Some(ElementType::Plant) => [
            r"    @_____@   ",
            r"   /  @@@  \  ",
            r"   | @~~@  |  ",
            r"   \_@__@_/   ",
            r"   @~    ~@   ",
        ],
        // Eau + Électrique (dos)
        Some(ElementType::Electric) => [
            r"    Z/===\Z   ",
            r"   /  ~~~  \  ",
            r"   |Z~~~~Z|   ",
            r"    \_Z__/    ",
            r"   Z~    ~Z   ",
        ],
        // Eau + Terre (dos)
        Some(ElementType::Earth) => [
            r"    [_____]   ",
            r"   /[ ~~~ ]   ",
            r"   |[~~~~]|   ",
            r"    [____]    ",
            r"   ##    ##   ",
        ],
        // Eau + Vent (dos)
        Some(ElementType::Wind) => [
            r"   =-_____-=  ",
            r"    / ~~~ \   ",
            r"  =|  ^^  |=  ",
            r"    \____/    ",
            r"  =-~    ~-=  ",
        ],
        // Eau + Ombre (dos)
        Some(ElementType::Shadow) => [
            r"    .___.     ",
            r"    / ... \   ",
            r"   |..~~..|   ",
            r"    \.___.    ",
            r"   ..    ..   ",
        ],
        // Eau + Lumière (dos)
        Some(ElementType::Light) => [
            r"    *___*     ",
            r"    / ~~~ \   ",
            r"   |*~~~~*|   ",
            r"    \*__*/    ",
            r"   *~  * ~*   ",
        ],
        _ => back_sprite(None),
    }
}
