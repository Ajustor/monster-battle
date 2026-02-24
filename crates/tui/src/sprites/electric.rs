//! Sprites de type Électrique — Loup de foudre et ses variantes.

use monster_battle_core::types::ElementType;

pub fn sprite(secondary: Option<ElementType>) -> [&'static str; 5] {
    match secondary {
        // Électrique pur — Loup de foudre
        None => [
            r"   /\   /\    ",
            r"  / o   o \   ",
            r"  | \===/ |   ",
            r"   \  ^  /    ",
            r"   /\   /\    ",
        ],
        // Électrique + Feu — Loup plasma
        Some(ElementType::Fire) => [
            r"  ^/\   /\^   ",
            r"  / o   o \   ",
            r"  |^^===^^|   ",
            r"   \  ^  /    ",
            r"  ^/\ ^ /\^  ",
        ],
        // Électrique + Eau — Requin de foudre
        Some(ElementType::Water) => [
            r"  ~/\   /\~   ",
            r"  / o   o \   ",
            r"  |~\===/ |   ",
            r"   \  ~  /    ",
            r"  ~/\~~~/ \~  ",
        ],
        // Électrique + Plante — Scarabée étincelle
        Some(ElementType::Plant) => [
            r"  @/\   /\@   ",
            r"  / o   o \   ",
            r"  |@\===/@|   ",
            r"   \  @  /    ",
            r"  @/\ @ /\@  ",
        ],
        // Électrique + Terre — Golem magnétique
        Some(ElementType::Earth) => [
            r"  [/\   /\]   ",
            r"  [ o   o ]   ",
            r"  |[\===/ |   ",
            r"   [  ^  ]    ",
            r"  [/\   /\]   ",
        ],
        // Électrique + Vent — Oiseau-tonnerre
        Some(ElementType::Wind) => [
            r"  =/\   /\=   ",
            r"  / o   o \   ",
            r" =| \===/ |=  ",
            r"   \  =  /    ",
            r"  =/\   /\=   ",
        ],
        // Électrique + Ombre — Pulsion sombre
        Some(ElementType::Shadow) => [
            r"  ./\   /\.   ",
            r"  / .   . \   ",
            r"  |.\===/ |   ",
            r"   \ ... /    ",
            r"  ./\   /\.   ",
        ],
        // Électrique + Lumière — Arc céleste
        Some(ElementType::Light) => [
            r"  */\   /\*   ",
            r"  / o   o \   ",
            r"  |*\===/*|   ",
            r"   \  *  /    ",
            r"  */\ * /\*  ",
        ],
        _ => sprite(None),
    }
}
