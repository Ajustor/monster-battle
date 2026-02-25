//! Pixel art 16×16 pour les monstres, rendu en demi-blocs dans le terminal.
//!
//! Chaque sprite est défini par une grille 16×16 de caractères de palette :
//! - 'M' = main color
//! - 'D' = dark color
//! - 'A' = accent color
//! - 'W' = white (eyes, highlights)
//! - 'X' = brown (trunk, bone)
//! - '.' = transparent (no pixel)
//!
//! Le rendu utilise le caractère Unicode `▀` (upper half block) avec :
//! - fg = couleur du pixel du haut
//! - bg = couleur du pixel du bas
//! ce qui permet d'afficher 2 rangées de pixels par ligne terminale → 8 lignes pour 16 rangées.

use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

use monster_battle_core::types::ElementType;

/// Dimensions du sprite pixel art.
pub const PIXEL_WIDTH: usize = 16;
pub const PIXEL_HEIGHT: usize = 16;
/// Nombre de lignes terminales (16 pixel rows / 2 = 8).
pub const PIXEL_LINES: usize = PIXEL_HEIGHT / 2;

/// Un sprite pixel art : 16 lignes de 16 caractères de palette.
pub type PixelGrid = [&'static str; PIXEL_HEIGHT];

/// Palette de couleurs RGB par type élémentaire.
struct TypePalette {
    main: Color,
    dark: Color,
    accent: Color,
}

fn type_palette(element: ElementType) -> TypePalette {
    match element {
        ElementType::Fire => TypePalette {
            main: Color::Rgb(244, 67, 54),     // #f44336
            dark: Color::Rgb(198, 40, 40),     // #c62828
            accent: Color::Rgb(255, 138, 101), // #ff8a65
        },
        ElementType::Water => TypePalette {
            main: Color::Rgb(33, 150, 243),    // #2196f3
            dark: Color::Rgb(21, 101, 192),    // #1565c0
            accent: Color::Rgb(100, 181, 246), // #64b5f6
        },
        ElementType::Plant => TypePalette {
            main: Color::Rgb(76, 175, 80),     // #4caf50
            dark: Color::Rgb(27, 94, 32),      // #1b5e20
            accent: Color::Rgb(165, 214, 167), // #a5d6a7
        },
        ElementType::Electric => TypePalette {
            main: Color::Rgb(255, 235, 59),    // #ffeb3b
            dark: Color::Rgb(245, 127, 23),    // #f57f17
            accent: Color::Rgb(255, 245, 157), // #fff59d
        },
        ElementType::Earth => TypePalette {
            main: Color::Rgb(121, 85, 72),     // #795548
            dark: Color::Rgb(62, 39, 35),      // #3e2723
            accent: Color::Rgb(188, 170, 164), // #bcaaa4
        },
        ElementType::Wind => TypePalette {
            main: Color::Rgb(0, 188, 212),     // #00bcd4
            dark: Color::Rgb(0, 96, 100),      // #006064
            accent: Color::Rgb(128, 222, 234), // #80deea
        },
        ElementType::Shadow => TypePalette {
            main: Color::Rgb(156, 39, 176),    // #9c27b0
            dark: Color::Rgb(74, 20, 140),     // #4a148c
            accent: Color::Rgb(206, 147, 216), // #ce93d8
        },
        ElementType::Light => TypePalette {
            main: Color::Rgb(255, 193, 7),     // #ffc107
            dark: Color::Rgb(255, 111, 0),     // #ff6f00
            accent: Color::Rgb(255, 224, 130), // #ffe082
        },
        ElementType::Normal => TypePalette {
            main: Color::Rgb(158, 158, 158),
            dark: Color::Rgb(97, 97, 97),
            accent: Color::Rgb(224, 224, 224),
        },
    }
}

/// Résout un caractère de palette en couleur RGB.
fn resolve_color(ch: u8, palette: &TypePalette) -> Option<Color> {
    match ch {
        b'M' => Some(palette.main),
        b'D' => Some(palette.dark),
        b'A' => Some(palette.accent),
        b'W' => Some(Color::White),
        b'X' => Some(Color::Rgb(139, 90, 43)), // brun (tronc, os)
        b'.' => None,
        _ => None,
    }
}

/// Convertit une grille pixel 16×16 en `Vec<Line>` pour ratatui.
///
/// Utilise le caractère `▀` (upper half block) pour afficher 2 rangées de pixels
/// par ligne de terminal, avec fg = couleur du haut, bg = couleur du bas.
///
/// Produit `PIXEL_LINES` (= 8) lignes.
pub fn render_pixel_sprite(
    grid: &PixelGrid,
    element: ElementType,
    secondary: Option<ElementType>,
) -> Vec<Line<'static>> {
    let palette = type_palette(element);
    // Indicateur de type secondaire : petit carré 3×2 en bas à droite (lignes 12-13, cols 13-15)
    let sec_color = secondary.map(|e| type_palette(e).main);
    let mut lines = Vec::with_capacity(PIXEL_LINES);

    for pair in 0..PIXEL_LINES {
        let top_row = grid[pair * 2].as_bytes();
        let bot_row = grid[pair * 2 + 1].as_bytes();
        let mut spans: Vec<Span<'static>> = Vec::with_capacity(PIXEL_WIDTH);

        for col in 0..PIXEL_WIDTH {
            // Vérifier si ce pixel est dans la zone de l'indicateur secondaire
            let top_pixel_row = pair * 2;
            let bot_pixel_row = pair * 2 + 1;
            let in_sec_top =
                sec_color.is_some() && top_pixel_row >= 12 && top_pixel_row <= 13 && col >= 13;
            let in_sec_bot =
                sec_color.is_some() && bot_pixel_row >= 12 && bot_pixel_row <= 13 && col >= 13;

            let top_color = if in_sec_top {
                sec_color
            } else if col < top_row.len() {
                resolve_color(top_row[col], &palette)
            } else {
                None
            };
            let bot_color = if in_sec_bot {
                sec_color
            } else if col < bot_row.len() {
                resolve_color(bot_row[col], &palette)
            } else {
                None
            };

            let (ch, style) = match (top_color, bot_color) {
                (Some(fg), Some(bg)) => ('▀', Style::default().fg(fg).bg(bg)),
                (Some(fg), None) => ('▀', Style::default().fg(fg)),
                (None, Some(bg)) => ('▄', Style::default().fg(bg)),
                (None, None) => (' ', Style::default()),
            };

            spans.push(Span::styled(String::from(ch), style));
        }

        lines.push(Line::from(spans));
    }

    lines
}

/// Rend un sprite pixel art avec un effet "hit" (flash rouge).
pub fn render_pixel_sprite_hit(grid: &PixelGrid) -> Vec<Line<'static>> {
    let red_palette = TypePalette {
        main: Color::Rgb(255, 60, 60),
        dark: Color::Rgb(180, 30, 30),
        accent: Color::Rgb(255, 120, 120),
    };
    let mut lines = Vec::with_capacity(PIXEL_LINES);

    for pair in 0..PIXEL_LINES {
        let top_row = grid[pair * 2].as_bytes();
        let bot_row = grid[pair * 2 + 1].as_bytes();
        let mut spans: Vec<Span<'static>> = Vec::with_capacity(PIXEL_WIDTH);

        for col in 0..PIXEL_WIDTH {
            let top_color = if col < top_row.len() {
                resolve_color(top_row[col], &red_palette)
            } else {
                None
            };
            let bot_color = if col < bot_row.len() {
                resolve_color(bot_row[col], &red_palette)
            } else {
                None
            };

            let (ch, style) = match (top_color, bot_color) {
                (Some(fg), Some(bg)) => ('▀', Style::default().fg(fg).bg(bg)),
                (Some(fg), None) => ('▀', Style::default().fg(fg)),
                (None, Some(bg)) => ('▄', Style::default().fg(bg)),
                (None, None) => (' ', Style::default()),
            };

            spans.push(Span::styled(String::from(ch), style));
        }

        lines.push(Line::from(spans));
    }

    lines
}

// ═══════════════════════════════════════════════════════════════════
//  Données des sprites — grilles 16×16 pour chaque type
// ═══════════════════════════════════════════════════════════════════

// ── Feu — Dragon ────────────────────────────────────────────────

pub const FIRE_FRONT: PixelGrid = [
    "................",
    "....A......A....",
    "....AMMMMMMA....",
    ".....MWMMWM.....",
    "....AMMMMMM.....",
    ".....MMDDMM.....",
    "...DD.MMMM.DD...",
    "...DDDMAAMDDD...",
    "...DDDMAAMDDDA..",
    "......MMMM.MM...",
    "......M..M......",
    "......M..M......",
    ".....DD..DD.....",
    "................",
    "................",
    "................",
];

pub const FIRE_BACK: PixelGrid = [
    "................",
    "....A......A....",
    "....AMMMMMMA....",
    ".....MMMMMM.....",
    ".....MMMMMM.....",
    ".....MMAAMM.....",
    "...DD.MMMM.DD...",
    "...DDDMAAMDDD...",
    "...DDDMAAMDDDA..",
    "......MMMM.MM...",
    "......M..M......",
    "......M..M......",
    ".....DD..DD.....",
    "................",
    "................",
    "................",
];

// ── Eau — Serpent ───────────────────────────────────────────────

pub const WATER_FRONT: PixelGrid = [
    "................",
    "................",
    "....AA..........",
    "....MMMM........",
    "....MWMD....A...",
    "..A.MMMMMM......",
    ".......MAA......",
    ".....MMMM.......",
    ".....MAAM.......",
    "........MMM.A...",
    "...A....MMMDD...",
    "................",
    "...DD...........",
    ".......DDD......",
    "................",
    "................",
];

pub const WATER_BACK: PixelGrid = [
    "................",
    "................",
    "....AA..........",
    "....MMMM........",
    "....MMMM........",
    "....MMMMMM......",
    ".......MAA......",
    ".....MMMM.......",
    ".....MAAM.......",
    "........MMM.A...",
    "...A....MMMDD...",
    "................",
    "...DD...........",
    ".......DDD......",
    "................",
    "................",
];

// ── Plante — Tréant ────────────────────────────────────────────

pub const PLANT_FRONT: PixelGrid = [
    ".....MMMMMM.....",
    "...AMAAMMMMA....",
    "...DMMMMMAAMD...",
    "...DMMMMMMMMD...",
    "......XXXX......",
    "......WXXW......",
    "......XDDX......",
    ".......XX.......",
    "......DXXD......",
    "......DXXD......",
    ".......XX.......",
    ".....DD..DD.....",
    "....DD....DD....",
    "................",
    "................",
    "................",
];

pub const PLANT_BACK: PixelGrid = [
    ".....MMMMMM.....",
    "...AMAAMMMMA....",
    "...DMMMMMAAMD...",
    "...DMMMMMMMMD...",
    "......XXXX......",
    "......XXXX......",
    "......XDDX......",
    ".......XX.......",
    "......DXXD......",
    "......DXXD......",
    ".......XX.......",
    ".....DD..DD.....",
    "....DD....DD....",
    "................",
    "................",
    "................",
];

// ── Électrique — Loup ──────────────────────────────────────────

pub const ELECTRIC_FRONT: PixelGrid = [
    "................",
    "....MM....MM....",
    "....MM....MM.A..",
    "..A..MMMMMM.....",
    "....AMWMMWMA....",
    "....AMMDDMMA.A..",
    "....MMMMMMMMMA..",
    "...AMAAAAAAMA...",
    "....MMMMMMMM....",
    "....MM....MM....",
    "....MM....MM....",
    "....MM....MM....",
    ".....D....D.....",
    "................",
    "................",
    "................",
];

pub const ELECTRIC_BACK: PixelGrid = [
    "................",
    "....MM....MM....",
    "....MM....MM....",
    ".....MMMMMM.....",
    "....AMMMMMMA....",
    "....AMMAAMMA.A..",
    "....MMMMMMMMMA..",
    "...AMAAAAAAMA...",
    "....MMMMMMMM....",
    "....MM....MM....",
    "....MM....MM....",
    "....MM....MM....",
    ".....D....D.....",
    "................",
    "................",
    "................",
];

// ── Terre — Golem ──────────────────────────────────────────────

pub const EARTH_FRONT: PixelGrid = [
    "................",
    "................",
    ".....MMMMMM.....",
    "....DMWMMWMD....",
    "....DAMMMMAD....",
    ".....MMDDMM.....",
    "..MMMMMMMMMMMM..",
    "..MMMAAAAAAMMM..",
    ".DMMMMDDDDMMMMD.",
    ".D..MMMMMMMM..D.",
    ".....MM..MM.....",
    ".....MM..MM.....",
    "....DDD..DDD....",
    "................",
    "................",
    "................",
];

pub const EARTH_BACK: PixelGrid = [
    "................",
    "................",
    ".....MMMMMM.....",
    "....DMMMMMMD....",
    "....DAMAAMAD....",
    ".....MMDDMM.....",
    "..MMMMMMMMMMMM..",
    "..MMMAAAAAAMMM..",
    ".DMMMMDDDDMMMMD.",
    ".D..MMMMMMMM..D.",
    ".....MM..MM.....",
    ".....MM..MM.....",
    "....DDD..DDD....",
    "................",
    "................",
    "................",
];

// ── Vent — Aigle ───────────────────────────────────────────────

pub const WIND_FRONT: PixelGrid = [
    "................",
    "......AA........",
    "......MMMM......",
    "DD....MWMDAA..DD",
    ".MMMM.MMMM.MMMM",
    ".MAAMMMMMMMMAAMM",
    ".....MAAAAM...AA",
    "AA...MAAAAM.....",
    ".....MMMMMM.....",
    "......DDDD......",
    "......DDDD......",
    ".....DDMMDD.....",
    "................",
    "................",
    "................",
    "................",
];

pub const WIND_BACK: PixelGrid = [
    "................",
    "......AA........",
    "......MMMM......",
    "DD....MMMMAA..DD",
    ".MMMM.MAAM.MMMM",
    ".MAAMMMMMMMMAAMM",
    ".....MAAAAM.....",
    ".....MAAAAM.....",
    ".....MMMMMM.....",
    "......DDDD......",
    "......DDDD......",
    ".....DDMMDD.....",
    "................",
    "................",
    "................",
    "................",
];

// ── Ombre — Spectre ────────────────────────────────────────────

pub const SHADOW_FRONT: PixelGrid = [
    "................",
    "....DDDDDDDD....",
    "...DDMMMMMMDD...",
    "....DMWMMWMD....",
    "..A.DMAMMAMD....",
    ".....MMDDMM..A..",
    ".....MMMMMM.....",
    "....DMMMMMMD....",
    "....DMMMMMMD....",
    "...A.MMMMMM.....",
    ".....MM..MM.....",
    "....D..MM..D....",
    "......M..M......",
    "................",
    "................",
    "................",
];

pub const SHADOW_BACK: PixelGrid = [
    "................",
    "....DDDDDDDD....",
    "...DDMMMMMMDD...",
    "....DMMMMMMD....",
    "....DMMMMMMD....",
    ".....MMDDMM.....",
    ".....MMMMMM.....",
    "....DMMMMMMD....",
    "....DMMMMMMD....",
    "...A.MMMMMM.....",
    ".....MM..MM.....",
    "....D..MM..D....",
    "......M..M......",
    "................",
    "................",
    "................",
];

// ── Lumière — Cerf ─────────────────────────────────────────────

pub const LIGHT_FRONT: PixelGrid = [
    "...AM......MA...",
    "....MA....AM....",
    "..A.M......M....",
    ".....MMMMMM...A.",
    ".....MWMMWM.....",
    ".....MMDDMM.....",
    "....MMMMMMMM....",
    "....MAAWWAAMAW..",
    ".A..MAAAAAAMA...",
    "....MMMMMMMM..A.",
    "....MM....MM....",
    "....MM....MM....",
    "....MD....DM....",
    "................",
    "................",
    "................",
];

pub const LIGHT_BACK: PixelGrid = [
    "...AM......MA...",
    "....MA....AM....",
    "..A.M......M....",
    ".....MMMMMM...A.",
    ".....MMMMMM.....",
    ".....MMDDMM.....",
    "....MMMMMMMM....",
    "....MAAWWAAMAW..",
    ".A..MAAAAAAMA...",
    "....MMMMMMMM..A.",
    "....MM....MM....",
    "....MM....MM....",
    "....MD....DM....",
    "................",
    "................",
    "................",
];

// ═══════════════════════════════════════════════════════════════════
//  Accesseurs par type
// ═══════════════════════════════════════════════════════════════════

/// Retourne le sprite pixel art de face pour le type primaire donné.
pub fn get_pixel_sprite(
    primary: ElementType,
    _secondary: Option<ElementType>,
) -> &'static PixelGrid {
    match primary {
        ElementType::Fire => &FIRE_FRONT,
        ElementType::Water => &WATER_FRONT,
        ElementType::Plant => &PLANT_FRONT,
        ElementType::Electric => &ELECTRIC_FRONT,
        ElementType::Earth => &EARTH_FRONT,
        ElementType::Wind => &WIND_FRONT,
        ElementType::Shadow => &SHADOW_FRONT,
        ElementType::Light => &LIGHT_FRONT,
        ElementType::Normal => &FIRE_FRONT,
    }
}

/// Retourne le sprite pixel art de dos pour le type primaire donné.
pub fn get_pixel_back_sprite(
    primary: ElementType,
    _secondary: Option<ElementType>,
) -> &'static PixelGrid {
    match primary {
        ElementType::Fire => &FIRE_BACK,
        ElementType::Water => &WATER_BACK,
        ElementType::Plant => &PLANT_BACK,
        ElementType::Electric => &ELECTRIC_BACK,
        ElementType::Earth => &EARTH_BACK,
        ElementType::Wind => &WIND_BACK,
        ElementType::Shadow => &SHADOW_BACK,
        ElementType::Light => &LIGHT_BACK,
        ElementType::Normal => &FIRE_BACK,
    }
}
