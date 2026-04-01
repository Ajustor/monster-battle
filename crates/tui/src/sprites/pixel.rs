//! Rendu pixel-art 16×16 en demi-blocs pour le terminal.
//!
//! Les données de grille (sprites) et les palettes de couleurs sont définies dans
//! la crate partagée `monster_battle_sprites`. Ce module ne contient que le code
//! de rendu spécifique à ratatui.
//!
//! Le rendu utilise le caractère Unicode `▀` (upper half block) avec :
//! - fg = couleur du pixel du haut
//! - bg = couleur du pixel du bas
//!
//! ce qui permet d'afficher 2 rangées de pixels par ligne terminale → 8 lignes pour 16 rangées.

use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

use monster_battle_core::AgeStage;
use monster_battle_core::types::ElementType;

// Ré-exports de la crate sprites partagée pour garder la compatibilité des imports.
pub use monster_battle_sprites::{BlendedGrid, get_blended_back_sprite, get_blended_sprite};

/// Nombre de lignes terminales (16 pixel rows / 2 = 8).
pub const PIXEL_LINES: usize = monster_battle_sprites::PIXEL_SIZE / 2;

// ═══════════════════════════════════════════════════════════════════
//  Palette TUI (Color RGB ratatui)
// ═══════════════════════════════════════════════════════════════════

struct TuiPalette {
    main: Color,
    dark: Color,
    accent: Color,
}

fn tui_palette(element: ElementType) -> TuiPalette {
    let p = monster_battle_sprites::type_palette(element);
    TuiPalette {
        main: Color::Rgb(p.main[0], p.main[1], p.main[2]),
        dark: Color::Rgb(p.dark[0], p.dark[1], p.dark[2]),
        accent: Color::Rgb(p.accent[0], p.accent[1], p.accent[2]),
    }
}

/// Résout un caractère de palette en couleur RGB.
/// Gère les majuscules (palette primaire) et les minuscules (palette secondaire).
fn resolve_color(ch: u8, primary: &TuiPalette, secondary: &TuiPalette) -> Option<Color> {
    match ch {
        b'M' => Some(primary.main),
        b'D' => Some(primary.dark),
        b'A' => Some(primary.accent),
        b'm' => Some(secondary.main),
        b'd' => Some(secondary.dark),
        b'a' => Some(secondary.accent),
        b'W' => Some(Color::White),
        b'X' => Some(Color::Rgb(139, 90, 43)),
        b'.' => None,
        _ => None,
    }
}

/// Résout un caractère pour un monstre âgé (palettes assombries).
fn resolve_color_old(ch: u8, primary: &TuiPalette, secondary: &TuiPalette) -> Option<Color> {
    match ch {
        b'M' => Some(darken(primary.main, 0.70)),
        b'D' => Some(primary.dark),
        b'A' => Some(darken(primary.accent, 0.50)),
        b'm' => Some(darken(secondary.main, 0.70)),
        b'd' => Some(secondary.dark),
        b'a' => Some(darken(secondary.accent, 0.50)),
        b'W' => Some(Color::Rgb(200, 200, 200)),
        b'X' => Some(Color::Rgb(100, 65, 30)),
        b'.' => None,
        _ => None,
    }
}

/// Assombrit une couleur RGB.
fn darken(color: Color, factor: f64) -> Color {
    match color {
        Color::Rgb(r, g, b) => Color::Rgb(
            (r as f64 * factor) as u8,
            (g as f64 * factor) as u8,
            (b as f64 * factor) as u8,
        ),
        other => other,
    }
}

// ═══════════════════════════════════════════════════════════════════
//  Rendu ratatui
// ═══════════════════════════════════════════════════════════════════

/// Convertit une grille blendée 16×16 en `Vec<Line>` pour ratatui.
///
/// Utilise le caractère `▀` (upper half block) pour afficher 2 rangées de pixels
/// par ligne de terminal, avec fg = couleur du haut, bg = couleur du bas.
///
/// Les caractères majuscules utilisent la palette du type primaire, les
/// minuscules celle du type secondaire (overlay décoratif).
///
/// Produit `PIXEL_LINES` (= 8) lignes.
pub fn render_pixel_sprite(
    grid: &BlendedGrid,
    element: ElementType,
    secondary: Option<ElementType>,
    age: AgeStage,
) -> Vec<Line<'static>> {
    let primary_pal = tui_palette(element);
    let secondary_pal = secondary
        .map(tui_palette)
        .unwrap_or_else(|| tui_palette(element));
    let resolver: fn(u8, &TuiPalette, &TuiPalette) -> Option<Color> = match age {
        AgeStage::Old => resolve_color_old,
        _ => resolve_color,
    };
    let width = monster_battle_sprites::PIXEL_SIZE;
    let mut lines = Vec::with_capacity(PIXEL_LINES);

    for pair in 0..PIXEL_LINES {
        let top_row = &grid[pair * 2];
        let bot_row = &grid[pair * 2 + 1];
        let mut spans: Vec<Span<'static>> = Vec::with_capacity(width);

        for col in 0..width {
            let top_ch = top_row[col];
            let bot_ch = bot_row[col];

            let top_color = resolver(top_ch, &primary_pal, &secondary_pal);
            let bot_color = resolver(bot_ch, &primary_pal, &secondary_pal);

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
pub fn render_pixel_sprite_hit(grid: &BlendedGrid) -> Vec<Line<'static>> {
    let red_palette = TuiPalette {
        main: Color::Rgb(255, 60, 60),
        dark: Color::Rgb(180, 30, 30),
        accent: Color::Rgb(255, 120, 120),
    };
    let width = monster_battle_sprites::PIXEL_SIZE;
    let mut lines = Vec::with_capacity(PIXEL_LINES);

    for pair in 0..PIXEL_LINES {
        let top_row = &grid[pair * 2];
        let bot_row = &grid[pair * 2 + 1];
        let mut spans: Vec<Span<'static>> = Vec::with_capacity(width);

        for col in 0..width {
            let top_color = resolve_color(top_row[col], &red_palette, &red_palette);
            let bot_color = resolve_color(bot_row[col], &red_palette, &red_palette);

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
