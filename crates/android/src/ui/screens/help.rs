//! Écran d'aide — tutoriel / comment jouer.

use bevy::prelude::*;
use bevy::state::state::NextState;

use crate::game::{GameData, GameScreen, ScreenEntity};
use crate::ui::common::{
    KEYBOARD_SCROLL_STEP, SAFE_BOTTOM, SAFE_TOP, ScrollableContent, colors, fonts,
};

/// Marqueur pour le bouton retour.
#[derive(Component)]
pub(crate) struct HelpBackButton;

/// Construit l'UI d'aide.
pub(crate) fn spawn_help(mut commands: Commands, data: Res<GameData>) {
    let _ = &data; // on utilise data pour scroll_offset au rebuild

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::new(
                    Val::Px(12.0),
                    Val::Px(12.0),
                    Val::Px(SAFE_TOP),
                    Val::Px(SAFE_BOTTOM),
                ),
                ..default()
            },
            BackgroundColor(colors::BACKGROUND),
            ScreenEntity,
            bevy::state::state_scoped::StateScoped(GameScreen::Help),
        ))
        .with_children(|parent| {
            // Bouton retour (haut gauche)
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                })
                .with_children(|bar| {
                    bar.spawn((
                        Node {
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                            ..default()
                        },
                        BackgroundColor(colors::PANEL),
                        BorderRadius::all(Val::Px(6.0)),
                        HelpBackButton,
                        Interaction::default(),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("< Retour"),
                            TextFont {
                                font_size: fonts::SMALL,
                                ..default()
                            },
                            TextColor(colors::TEXT_PRIMARY),
                        ));
                    });
                });

            // Titre
            parent.spawn((
                Text::new("Aide -- Comment jouer"),
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
                    "Bienvenue dans Monster Battle !",
                    colors::ACCENT_YELLOW,
                    &[],
                ),
                (
                    "-- But du jeu --",
                    colors::ACCENT_GREEN,
                    &[
                        "Elevez un monstre unique, nourrissez-le, entrainez-le",
                        "et affrontez d'autres joueurs en combat PvP !",
                        "Votre monstre est mortel : il vieillit et peut mourir",
                        "de vieillesse, de faim ou au combat.",
                        "Reproduisez-le pour creer une lignee plus puissante.",
                    ],
                ),
                (
                    "-- Cycle de vie --",
                    colors::ACCENT_GREEN,
                    &[
                        "Bebe (0-15%)    > Stats x80%",
                        "Jeune (15-40%)  > Stats x95%",
                        "Adulte (40-75%) > Stats x110%  < pic",
                        "Vieux (75-100%) > Stats x85%",
                        "Mort au-dela de ~30 jours de vie",
                    ],
                ),
                (
                    "-- Systeme de faim --",
                    colors::ACCENT_GREEN,
                    &[
                        "A faim          > Stats normales (x100%)",
                        "Rassasie (<12h) > Boost ! (x115%)",
                        "Trop mange (3x) > Malus (x85%)",
                        "Affame (3+ jours) > Mort de faim !",
                        "",
                        "Nourrissez votre monstre depuis sa fiche (F).",
                        "Attention : 3 repas en 12h = gavage > malus.",
                    ],
                ),
                (
                    "-- Combat --",
                    colors::ACCENT_GREEN,
                    &[
                        "Entrainement docile : 50% XP, pas de mort",
                        "Entrainement sauvage : 100% XP, mort possible",
                        "PvP en ligne : 200% XP si KO, mort du perdant",
                        "Fuite PvP : pas de mort, adversaire +100% XP",
                        "",
                        "Les stats determinent l'ordre d'attaque et les degats.",
                        "Les types elementaires creent des avantages.",
                    ],
                ),
                (
                    "-- Types elementaires --",
                    colors::ACCENT_GREEN,
                    &[
                        "Feu > Plante > Eau > Feu",
                        "Electrique > Eau   Terre > Electrique",
                        "Vent > Terre   Ombre > Lumiere > Ombre",
                    ],
                ),
                (
                    "-- Reproduction --",
                    colors::ACCENT_GREEN,
                    &[
                        "Croisez votre monstre avec un autre joueur.",
                        "Le bebe herite des types, stats et traits.",
                        "Des mutations peuvent apparaitre !",
                        "Le type secondaire est transmis par reproduction.",
                    ],
                ),
                (
                    "-- Traits genetiques --",
                    colors::ACCENT_GREEN,
                    &[
                        "CriticalStrike  > +crit (20% vs 8%)",
                        "Berserk         > x1.5 ATK sous 25% PV",
                        "Evasion         > 12% d'esquive",
                        "Thorns          > 15% degats renvoyes",
                        "Tenacity        > 15% survie a 1 PV",
                        "Regeneration    > 5% PV max regeneres/tour",
                        "FastLearner     > XP x1.5",
                        "Longevity       > +15 jours de vie",
                    ],
                ),
                (
                    "-- Commandes --",
                    colors::ACCENT_GREEN,
                    &[
                        "Toucher un bouton pour selectionner",
                        "Bouton < Retour pour revenir",
                    ],
                ),
            ];

            // Conteneur scrollable
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        overflow: Overflow::scroll_y(),
                        flex_grow: 1.0,
                        ..default()
                    },
                    ScrollPosition::default(),
                    ScrollableContent,
                ))
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
        });
}

/// Gestion des entrées sur l'aide.
pub(crate) fn handle_help_input(
    mut data: ResMut<GameData>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameScreen>>,
    back_query: Query<(&Interaction, &HelpBackButton), Changed<Interaction>>,
    mut scroll_query: Query<&mut ScrollPosition, With<ScrollableContent>>,
) {
    for (interaction, _) in &back_query {
        if *interaction == Interaction::Pressed {
            data.message = None;
            next_state.set(GameScreen::MainMenu);
            data.menu_index = 0;
            return;
        }
    }

    if keyboard.just_pressed(KeyCode::ArrowUp) {
        for mut scroll_pos in &mut scroll_query {
            scroll_pos.offset_y = (scroll_pos.offset_y - KEYBOARD_SCROLL_STEP).max(0.0);
        }
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        for mut scroll_pos in &mut scroll_query {
            scroll_pos.offset_y += KEYBOARD_SCROLL_STEP;
        }
    }
    if keyboard.just_pressed(KeyCode::Escape) || keyboard.just_pressed(KeyCode::KeyQ) {
        data.message = None;
        next_state.set(GameScreen::MainMenu);
        data.menu_index = 0;
    }
}
