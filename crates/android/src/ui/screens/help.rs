//! Écran d'aide — tutoriel / comment jouer.

use bevy::prelude::*;
use bevy::state::state::NextState;

use crate::game::{GameData, GameScreen, ScreenEntity};
use crate::ui::common::{colors, fonts};

/// Construit l'UI d'aide.
pub(crate) fn spawn_help(mut commands: Commands, data: Res<GameData>) {
    let _ = &data; // on utilise data pour scroll_offset au rebuild

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                overflow: Overflow::clip_y(),
                ..default()
            },
            BackgroundColor(colors::BACKGROUND),
            ScreenEntity,
        ))
        .with_children(|parent| {
            // Titre
            parent.spawn((
                Text::new("❓ Aide — Comment jouer"),
                TextFont {
                    font_size: fonts::HEADING,
                    ..default()
                },
                TextColor(colors::ACCENT_BLUE),
                Node {
                    margin: UiRect::bottom(Val::Px(16.0)),
                    ..default()
                },
            ));

            // Contenu d'aide (sections)
            let sections: &[(&str, Color, &[&str])] = &[
                (
                    "🐉 Bienvenue dans Monster Battle !",
                    colors::ACCENT_YELLOW,
                    &[],
                ),
                (
                    "── But du jeu ──",
                    colors::ACCENT_GREEN,
                    &[
                        "Élevez un monstre unique, nourrissez-le, entraînez-le",
                        "et affrontez d'autres joueurs en combat PvP !",
                        "Votre monstre est mortel : il vieillit et peut mourir",
                        "de vieillesse, de faim ou au combat.",
                        "Reproduisez-le pour créer une lignée plus puissante.",
                    ],
                ),
                (
                    "── Cycle de vie ──",
                    colors::ACCENT_GREEN,
                    &[
                        "💿 Bébé (0-15%)    → Stats ×80%",
                        "🌱 Jeune (15-40%)  → Stats ×95%",
                        "💪 Adulte (40-75%) → Stats ×110%  ← pic",
                        "🧓 Vieux (75-100%) → Stats ×85%",
                        "💀 Mort au-delà de ~30 jours de vie",
                    ],
                ),
                (
                    "── Système de faim ──",
                    colors::ACCENT_GREEN,
                    &[
                        "🍽️ A faim          → Stats normales (×100%)",
                        "😊 Rassasié (<12h) → Boost ! (×115%)",
                        "🤢 Trop mangé (3×) → Malus (×85%)",
                        "💀 Affamé (3+ jours) → Mort de faim !",
                        "",
                        "Nourrissez votre monstre depuis sa fiche (F).",
                        "Attention : 3 repas en 12h = gavage → malus.",
                    ],
                ),
                (
                    "── Combat ──",
                    colors::ACCENT_GREEN,
                    &[
                        "⚔️  Entraînement docile : 50% XP, pas de mort",
                        "⚔️  Entraînement sauvage : 100% XP, mort possible",
                        "🗡️  PvP en ligne : 200% XP si KO, mort du perdant",
                        "🏳️  Fuite PvP : pas de mort, adversaire +100% XP",
                        "",
                        "Les stats déterminent l'ordre d'attaque et les dégâts.",
                        "Les types élémentaires créent des avantages.",
                    ],
                ),
                (
                    "── Types élémentaires ──",
                    colors::ACCENT_GREEN,
                    &[
                        "🔥 Feu > 🌿 Plante > 💧 Eau > 🔥 Feu",
                        "⚡ Électrique > 💧 Eau   🌍 Terre > ⚡ Électrique",
                        "🌀 Vent > 🌍 Terre   🌑 Ombre > 🌟 Lumière > 🌑 Ombre",
                    ],
                ),
                (
                    "── Reproduction ──",
                    colors::ACCENT_GREEN,
                    &[
                        "🧬 Croisez votre monstre avec un autre joueur.",
                        "Le bébé hérite des types, stats et traits.",
                        "Des mutations peuvent apparaître !",
                        "Le type secondaire est transmis par reproduction.",
                    ],
                ),
                (
                    "── Traits génétiques ──",
                    colors::ACCENT_GREEN,
                    &[
                        "🎯 CriticalStrike  → +crit (20% vs 8%)",
                        "😡 Berserk         → ×1.5 ATK sous 25% PV",
                        "💨 Evasion         → 12% d'esquive",
                        "🌵 Thorns          → 15% dégâts renvoyés",
                        "💪 Tenacity        → 15% survie à 1 PV",
                        "🩹 Regeneration    → 5% PV max régénérés/tour",
                        "📚 FastLearner     → XP ×1.5",
                        "🕰️  Longevity       → +15 jours de vie",
                    ],
                ),
                (
                    "── Commandes ──",
                    colors::ACCENT_GREEN,
                    &[
                        "↑↓      Naviguer",
                        "←→      Docile / Sauvage (entraînement)",
                        "Enter   Sélectionner / Confirmer",
                        "Esc     Retour / Quitter",
                        "F       Nourrir (fiche monstre)",
                    ],
                ),
            ];

            // Conteneur scrollable
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::clip_y(),
                    ..default()
                })
                .with_children(|scroll| {
                    for (title, title_color, lines) in sections {
                        // Titre de section
                        scroll.spawn((
                            Text::new(title.to_string()),
                            TextFont {
                                font_size: fonts::BODY,
                                ..default()
                            },
                            TextColor(*title_color),
                            Node {
                                margin: UiRect::vertical(Val::Px(6.0)),
                                ..default()
                            },
                        ));

                        // Lignes de la section
                        for line in *lines {
                            if line.is_empty() {
                                scroll.spawn((
                                    Text::new(" "),
                                    TextFont {
                                        font_size: fonts::SMALL,
                                        ..default()
                                    },
                                    TextColor(colors::TEXT_PRIMARY),
                                    Node {
                                        height: Val::Px(6.0),
                                        ..default()
                                    },
                                ));
                            } else {
                                scroll.spawn((
                                    Text::new(line.to_string()),
                                    TextFont {
                                        font_size: fonts::SMALL,
                                        ..default()
                                    },
                                    TextColor(colors::TEXT_PRIMARY),
                                    Node {
                                        margin: UiRect::bottom(Val::Px(2.0)),
                                        ..default()
                                    },
                                ));
                            }
                        }
                    }
                });

            // Footer
            parent.spawn((
                Text::new("↑↓ Défiler  Esc/Q Retour"),
                TextFont {
                    font_size: fonts::SMALL,
                    ..default()
                },
                TextColor(colors::TEXT_SECONDARY),
                Node {
                    margin: UiRect::top(Val::Px(12.0)),
                    ..default()
                },
            ));
        });
}

/// Gestion des entrées sur l'aide.
pub(crate) fn handle_help_input(
    mut data: ResMut<GameData>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameScreen>>,
) {
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        data.scroll_offset = data.scroll_offset.saturating_sub(1);
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        data.scroll_offset += 1;
    }
    if keyboard.just_pressed(KeyCode::Escape) || keyboard.just_pressed(KeyCode::KeyQ) {
        data.scroll_offset = 0;
        data.message = None;
        next_state.set(GameScreen::MainMenu);
        data.menu_index = 0;
    }
}
