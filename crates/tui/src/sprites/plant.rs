//! Sprites de type Plante — Treant et ses variantes.

use monster_battle_core::types::ElementType;

pub fn sprite(secondary: Option<ElementType>) -> [&'static str; 5] {
    match secondary {
        // Plante pure — Treant
        None => [
            r"    ,@@@,     ",
            r"   /|o o|\    ",
            r"   || ~ ||    ",
            r"   /|___|\    ",
            r"   |_| |_|    ",
        ],
        // Plante + Feu — Arbre ardent
        Some(ElementType::Fire) => [
            r"   ^,@@@,^    ",
            r"   /|o o|\    ",
            r"   |^ ~ ^|    ",
            r"   /|^^^|\    ",
            r"   |_|^|_|    ",
        ],
        // Plante + Eau — Lotus vivant
        Some(ElementType::Water) => [
            r"   ~,@@@,~    ",
            r"   /|o o|\    ",
            r"   |~ ~ ~|    ",
            r"   /|___|~    ",
            r"   ~~| |~~    ",
        ],
        // Plante + Électrique — Cactus statique
        Some(ElementType::Electric) => [
            r"   Z,@@@,Z    ",
            r"   /|o o|\    ",
            r"   |Z ~ Z|    ",
            r"   /|___|\    ",
            r"   Z_| |_Z    ",
        ],
        // Plante + Terre — Golem de mousse
        Some(ElementType::Earth) => [
            r"   [,@@@,]    ",
            r"   [|o o|]    ",
            r"   || ~ ||    ",
            r"   [|___|]    ",
            r"   [_] [_]    ",
        ],
        // Plante + Vent — Pissenlit esprit
        Some(ElementType::Wind) => [
            r"  =-,@@@,-=   ",
            r"   /|o o|\    ",
            r"  =| ~ ~ |=  ",
            r"   /|___|\    ",
            r"  =-|_|_|-=   ",
        ],
        // Plante + Ombre — Ronce maudite
        Some(ElementType::Shadow) => [
            r"   .,@@@,.    ",
            r"   /|. .|.    ",
            r"   || . ||    ",
            r"   /|.__|.    ",
            r"   ..|..|..   ",
        ],
        // Plante + Lumière — Floraison sacrée
        Some(ElementType::Light) => [
            r"   *,@@@,*    ",
            r"   /|o o|\    ",
            r"   |* ~ *|    ",
            r"   /|___|\    ",
            r"   *_| |_*    ",
        ],
        _ => sprite(None),
    }
}
