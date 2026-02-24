//! Sprites de type Feu — Dragon et ses variantes.

use monster_battle_core::types::ElementType;

pub fn sprite(secondary: Option<ElementType>) -> [&'static str; 5] {
    match secondary {
        // Feu pur — Dragon de flammes
        None => [
            r"    /\_/\     ",
            r"   ( o.o )    ",
            r"    )^^^(     ",
            r"   /|   |\    ",
            r"  ^ '   ' ^  ",
        ],
        // Feu + Eau — Wyrm de vapeur
        Some(ElementType::Water) => [
            r"    /\_/\     ",
            r"   (~o.o~)    ",
            r"   ~)~~~(~    ",
            r"   /|   |\    ",
            r"  ~~~   ~~~   ",
        ],
        // Feu + Plante — Dragon floral
        Some(ElementType::Plant) => [
            r"   @/\_/\@    ",
            r"   ( o.o )    ",
            r"   @)^^^(@    ",
            r"  @/|   |\@   ",
            r"  @@ ' ' @@  ",
        ],
        // Feu + Électrique — Drake de foudre
        Some(ElementType::Electric) => [
            r"   //\_/\\    ",
            r"   ( o.o )    ",
            r"   Z)^^^(Z    ",
            r"   /|   |\    ",
            r"  Z ' Z ' Z  ",
        ],
        // Feu + Terre — Dragon de lave
        Some(ElementType::Earth) => [
            r"   [/\_/\]    ",
            r"   [ o.o ]    ",
            r"    ]^^^[     ",
            r"   [|   |]    ",
            r"  ## ' ' ##   ",
        ],
        // Feu + Vent — Phoenix
        Some(ElementType::Wind) => [
            r"  =-/\_/\-=   ",
            r"   ( o.o )    ",
            r"  =>)^^^(<=   ",
            r"   /|   |\    ",
            r"  -=' ' '=-  ",
        ],
        // Feu + Ombre — Drake infernal
        Some(ElementType::Shadow) => [
            r"   ./\_/\.    ",
            r"   ( .  . )   ",
            r"    )....(    ",
            r"   /|   |\    ",
            r"  ..' ' '..  ",
        ],
        // Feu + Lumière — Dragon solaire
        Some(ElementType::Light) => [
            r"   */\_/\*    ",
            r"   ( o.o )    ",
            r"   *)^^^(*    ",
            r"   /| * |\    ",
            r"  *' ' ' '*  ",
        ],
        // Fallback Normal
        _ => sprite(None),
    }
}
