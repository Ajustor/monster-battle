//! Sprites de type Vent — Aigle et ses variantes.

use monster_battle_core::types::ElementType;

pub fn sprite(secondary: Option<ElementType>) -> [&'static str; 5] {
    match secondary {
        // Vent pur — Aigle céleste
        None => [
            r"  __/\  /\__  ",
            r" /  (o  o)  \ ",
            r" |   \__/   | ",
            r"  \__/  \__/  ",
            r"     \  /     ",
        ],
        // Vent + Feu — Phoenix
        Some(ElementType::Fire) => [
            r" ^__/\  /\__^ ",
            r" /  (o  o)  \ ",
            r" |  ^\/\/^  | ",
            r"  \__/^^\_^/  ",
            r"    ^\^^/^    ",
        ],
        // Vent + Eau — Albatros des tempêtes
        Some(ElementType::Water) => [
            r" ~__/\  /\__~ ",
            r" /  (o  o)  \ ",
            r" | ~~\__/~~ | ",
            r"  \__/  \__/  ",
            r"    ~\~~/~    ",
        ],
        // Vent + Plante — Papillon de feuilles
        Some(ElementType::Plant) => [
            r" @__/\  /\__@ ",
            r" /  (o  o)  \ ",
            r" | @@\__/@@ | ",
            r"  \__/@_\__/  ",
            r"    @\@@/@    ",
        ],
        // Vent + Électrique — Faucon-éclair
        Some(ElementType::Electric) => [
            r" Z__/\  /\__Z ",
            r" /  (o  o)  \ ",
            r" | Z \__/ Z | ",
            r"  \__/ZZ\__/  ",
            r"    Z\ZZ/Z    ",
        ],
        // Vent + Terre — Faucon de sable
        Some(ElementType::Earth) => [
            r" #__/\  /\__# ",
            r" /  (o  o)  \ ",
            r" | # \__/ # | ",
            r"  [__/  \__]  ",
            r"    #\##/#    ",
        ],
        // Vent + Ombre — Corbeau fantôme
        Some(ElementType::Shadow) => [
            r" .__/\  /\__. ",
            r" /  (.  .)  \ ",
            r" | . \__/ . | ",
            r"  \__/..\__/  ",
            r"    .\.../    ",
        ],
        // Vent + Lumière — Colombe angélique
        Some(ElementType::Light) => [
            r" *__/\  /\__* ",
            r" /  (o  o)  \ ",
            r" | * \__/ * | ",
            r"  \__/ *\__/  ",
            r"    *\**/     ",
        ],
        _ => sprite(None),
    }
}
