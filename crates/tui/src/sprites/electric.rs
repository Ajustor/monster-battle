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

pub fn back_sprite(secondary: Option<ElementType>) -> [&'static str; 5] {
    match secondary {
        // Électrique pur (dos)
        None => [
            r"   /\   /\    ",
            r"  / ^^^^^ \   ",
            r"  |  ===  |   ",
            r"   \ vvv /    ",
            r"    | w |     ",
        ],
        // Électrique + Feu (dos)
        Some(ElementType::Fire) => [
            r"  ^/\   /\^   ",
            r"  / ^^^^^ \   ",
            r"  |^^===^^|   ",
            r"   \ vvv /    ",
            r"   ^| w |^    ",
        ],
        // Électrique + Eau (dos)
        Some(ElementType::Water) => [
            r"  ~/\   /\~   ",
            r"  / ~~~~~ \   ",
            r"  |~ === ~|   ",
            r"   \ ~~~ /    ",
            r"   ~| w |~    ",
        ],
        // Électrique + Plante (dos)
        Some(ElementType::Plant) => [
            r"  @/\   /\@   ",
            r"  / @@@@@ \   ",
            r"  |@ === @|   ",
            r"   \ @@@ /    ",
            r"   @| w |@    ",
        ],
        // Électrique + Terre (dos)
        Some(ElementType::Earth) => [
            r"  [/\   /\]   ",
            r"  [ ^^^^^ ]   ",
            r"  |[ === ]|   ",
            r"   [ vvv ]    ",
            r"   [| w |]    ",
        ],
        // Électrique + Vent (dos)
        Some(ElementType::Wind) => [
            r"  =/\   /\=   ",
            r"  / ===== \   ",
            r" =|  ===  |=  ",
            r"   \ === /    ",
            r"   =| w |=    ",
        ],
        // Électrique + Ombre (dos)
        Some(ElementType::Shadow) => [
            r"  ./\   /\.   ",
            r"  / ..... \   ",
            r"  |. === .|   ",
            r"   \ ... /    ",
            r"   .| w |.    ",
        ],
        // Électrique + Lumière (dos)
        Some(ElementType::Light) => [
            r"  */\   /\*   ",
            r"  / ***** \   ",
            r"  |* === *|   ",
            r"   \ *** /    ",
            r"   *| w |*    ",
        ],
        _ => back_sprite(None),
    }
}
