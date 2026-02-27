//! Composants UI communs — en-tête, pied de page, styles partagés.

use bevy::prelude::*;
use bevy::text::Font;

use crate::game::ScreenEntity;

// ═══════════════════════════════════════════════════════════════════
//  Police personnalisée (DejaVu Sans — support Latin complet)
// ═══════════════════════════════════════════════════════════════════

/// Données de la police DejaVu Sans embarquée dans le binaire.
const FONT_DATA: &[u8] = include_bytes!("../../assets/fonts/DejaVuSans.ttf");

/// Système de démarrage : remplace la police par défaut de Bevy
/// par DejaVu Sans (support complet des accents français et symboles).
pub fn setup_custom_font(mut fonts: ResMut<Assets<Font>>) {
    let font = Font::try_from_bytes(FONT_DATA.to_vec())
        .expect("Impossible de charger la police DejaVu Sans");
    // Remplacer la police à l'ID par défaut → tous les TextFont::default() l'utiliseront
    fonts.insert(Handle::<Font>::default().id(), font);
    log::info!("Police DejaVu Sans chargee avec succes");
}

// ═══════════════════════════════════════════════════════════════════
//  Marge de sécurité pour l'encoche caméra (safe area)
// ═══════════════════════════════════════════════════════════════════

/// Marge haute pour éviter l'encoche caméra sur Android.
#[cfg(target_os = "android")]
pub const SAFE_TOP: f32 = 48.0;
#[cfg(not(target_os = "android"))]
pub const SAFE_TOP: f32 = 16.0;

// ═══════════════════════════════════════════════════════════════════
//  Constantes de style
// ═══════════════════════════════════════════════════════════════════

/// Couleurs du jeu.
pub mod colors {
    use bevy::prelude::*;

    pub const BACKGROUND: Color = Color::srgb(0.08, 0.08, 0.12);
    pub const PANEL: Color = Color::srgb(0.12, 0.12, 0.18);
    pub const BORDER: Color = Color::srgb(0.25, 0.25, 0.35);
    pub const TEXT_PRIMARY: Color = Color::WHITE;
    pub const TEXT_SECONDARY: Color = Color::srgb(0.6, 0.6, 0.7);
    pub const ACCENT_YELLOW: Color = Color::srgb(1.0, 0.84, 0.0);
    pub const ACCENT_RED: Color = Color::srgb(0.96, 0.26, 0.21);
    pub const ACCENT_GREEN: Color = Color::srgb(0.30, 0.69, 0.31);
    pub const ACCENT_BLUE: Color = Color::srgb(0.13, 0.59, 0.95);
    pub const ACCENT_MAGENTA: Color = Color::srgb(0.61, 0.15, 0.69);

    pub const HP_HIGH: Color = Color::srgb(0.30, 0.69, 0.31);
    pub const HP_MID: Color = Color::srgb(1.0, 0.84, 0.0);
    pub const HP_LOW: Color = Color::srgb(0.96, 0.26, 0.21);
}

/// Tailles de police.
pub mod fonts {
    pub const TITLE: f32 = 28.0;
    pub const HEADING: f32 = 22.0;
    pub const BODY: f32 = 18.0;
    pub const SMALL: f32 = 14.0;
}

// ═══════════════════════════════════════════════════════════════════
//  Composants helper
// ═══════════════════════════════════════════════════════════════════

/// Crée un nœud racine plein écran pour un écran.
/// Inclut une marge haute pour éviter l'encoche caméra sur Android.
pub fn screen_root() -> Node {
    Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::FlexStart,
        padding: UiRect::new(
            Val::Px(16.0),
            Val::Px(16.0),
            Val::Px(SAFE_TOP),
            Val::Px(16.0),
        ),
        ..default()
    }
}

/// Crée un en-tête « Monster Battle ».
pub fn spawn_header(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            Text::new("~ Monster Battle ~"),
            TextFont {
                font_size: fonts::TITLE,
                ..default()
            },
            TextColor(colors::ACCENT_YELLOW),
            Node {
                margin: UiRect::bottom(Val::Px(24.0)),
                ..default()
            },
            ScreenEntity,
        ))
        .id()
}

/// Crée un bouton de menu avec texte.
pub fn spawn_menu_button(commands: &mut Commands, text: &str, selected: bool) -> Entity {
    let bg_color = if selected {
        colors::ACCENT_YELLOW
    } else {
        colors::PANEL
    };
    let text_color = if selected {
        Color::BLACK
    } else {
        colors::TEXT_PRIMARY
    };

    commands
        .spawn((
            Node {
                width: Val::Percent(90.0),
                padding: UiRect::axes(Val::Px(20.0), Val::Px(14.0)),
                margin: UiRect::bottom(Val::Px(8.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(bg_color),
            BorderRadius::all(Val::Px(8.0)),
            ScreenEntity,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(text.to_string()),
                TextFont {
                    font_size: fonts::BODY,
                    ..default()
                },
                TextColor(text_color),
            ));
        })
        .id()
}

/// Crée un pied de page avec texte d'aide.
pub fn spawn_footer(commands: &mut Commands, text: &str) -> Entity {
    commands
        .spawn((
            Text::new(text.to_string()),
            TextFont {
                font_size: fonts::SMALL,
                ..default()
            },
            TextColor(colors::TEXT_SECONDARY),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(16.0),
                ..default()
            },
            ScreenEntity,
        ))
        .id()
}

/// Retourne la couleur de la barre de PV selon le pourcentage.
pub fn hp_color(current: u32, max: u32) -> Color {
    if max == 0 {
        return colors::HP_LOW;
    }
    let ratio = current as f32 / max as f32;
    if ratio > 0.5 {
        colors::HP_HIGH
    } else if ratio > 0.2 {
        colors::HP_MID
    } else {
        colors::HP_LOW
    }
}
