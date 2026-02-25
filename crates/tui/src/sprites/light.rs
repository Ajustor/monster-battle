//! Sprites de type Lumière — Cerf céleste et ses variantes.

use monster_battle_core::types::ElementType;

pub fn sprite(secondary: Option<ElementType>) -> [&'static str; 5] {
    match secondary {
        // Lumière pure — Cerf céleste
        None => [
            r"   \*|  |*/   ",
            r"    (o  o)    ",
            r"    | ** |    ",
            r"    |/  \|    ",
            r"    *    *    ",
        ],
        // Lumière + Feu — Phoenix solaire
        Some(ElementType::Fire) => [
            r"  ^\*| |*/^   ",
            r"    (o  o)    ",
            r"    |^**^|    ",
            r"    |/^^\ |   ",
            r"   ^*    *^   ",
        ],
        // Lumière + Eau — Poisson lunaire
        Some(ElementType::Water) => [
            r"  ~\*|  |*/~  ",
            r"    (o  o)    ",
            r"    |~**~|    ",
            r"    |/~~\|    ",
            r"   ~*    *~   ",
        ],
        // Lumière + Plante — Cerf sacré
        Some(ElementType::Plant) => [
            r"  @\*|  |*/@  ",
            r"    (o  o)    ",
            r"    |@**@|    ",
            r"    |/@@\|    ",
            r"   @*    *@   ",
        ],
        // Lumière + Électrique — Étincelle stellaire
        Some(ElementType::Electric) => [
            r"  Z\*|  |*/Z  ",
            r"    (o  o)    ",
            r"    |Z**Z|    ",
            r"    |/ZZ\|    ",
            r"   Z*    *Z   ",
        ],
        // Lumière + Terre — Bête de diamant
        Some(ElementType::Earth) => [
            r"  [\*|  |*/]  ",
            r"    [o  o]    ",
            r"    |[**]|    ",
            r"    |/##\|    ",
            r"   [*    *]   ",
        ],
        // Lumière + Vent — Oiseau angélique
        Some(ElementType::Wind) => [
            r"  =\*|  |*/=  ",
            r"    (o  o)    ",
            r"   =| ** |=   ",
            r"    |/==\|    ",
            r"   =*    *=   ",
        ],
        // Lumière + Ombre — Sphinx du crépuscule
        Some(ElementType::Shadow) => [
            r"  .\*|  |*/.  ",
            r"    (.  .)    ",
            r"    |.**.|    ",
            r"    |/..\|    ",
            r"   .*    *.   ",
        ],
        _ => sprite(None),
    }
}

pub fn back_sprite(secondary: Option<ElementType>) -> [&'static str; 5] {
    match secondary {
        // Lumière pure (dos)
        None => [
            r"   \*|  |*/   ",
            r"    (^^^^)    ",
            r"    | ** |    ",
            r"    |/  \|    ",
            r"    * ww *    ",
        ],
        // Lumière + Feu (dos)
        Some(ElementType::Fire) => [
            r"  ^\*| |*/^   ",
            r"    (^^^^)    ",
            r"    |^**^|    ",
            r"    |/^^\|    ",
            r"   ^* ww *^   ",
        ],
        // Lumière + Eau (dos)
        Some(ElementType::Water) => [
            r"  ~\*|  |*/~  ",
            r"    (~~~~)    ",
            r"    |~**~|    ",
            r"    |/~~\|    ",
            r"   ~* ww *~   ",
        ],
        // Lumière + Plante (dos)
        Some(ElementType::Plant) => [
            r"  @\*|  |*/@  ",
            r"    (@@@@)    ",
            r"    |@**@|    ",
            r"    |/@@\|    ",
            r"   @* ww *@   ",
        ],
        // Lumière + Électrique (dos)
        Some(ElementType::Electric) => [
            r"  Z\*|  |*/Z  ",
            r"    (ZZZZ)    ",
            r"    |Z**Z|    ",
            r"    |/ZZ\|    ",
            r"   Z* ww *Z   ",
        ],
        // Lumière + Terre (dos)
        Some(ElementType::Earth) => [
            r"  [\*|  |*/]  ",
            r"    [^^^^]    ",
            r"    |[**]|    ",
            r"    |/##\|    ",
            r"   [* ww *]   ",
        ],
        // Lumière + Vent (dos)
        Some(ElementType::Wind) => [
            r"  =\*|  |*/=  ",
            r"    (====)    ",
            r"   =| ** |=   ",
            r"    |/==\|    ",
            r"   =* ww *=   ",
        ],
        // Lumière + Ombre (dos)
        Some(ElementType::Shadow) => [
            r"  .\*|  |*/.  ",
            r"    (....)    ",
            r"    |.**.|    ",
            r"    |/..\|    ",
            r"   .* ww *.   ",
        ],
        _ => back_sprite(None),
    }
}
