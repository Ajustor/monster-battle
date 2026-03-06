//! Écran du mini-jeu Réflexe (QTE flèches) — Android / Bevy.

use bevy::prelude::*;
use bevy::state::state::NextState;

use monster_battle_core::minigame::reflex::{Arrow, ReflexGame, RoundResult};

use crate::game::{GameData, GameScreen, ScreenEntity};
use crate::ui::common::{SAFE_BOTTOM, SAFE_TOP, colors, fonts};

use super::{
    ContinueButton, MinigameBackButton, StatusText, apply_minigame_reward, clear_minigame_state,
};

// ═══════════════════════════════════════════════════════════════════
//  Composants
// ═══════════════════════════════════════════════════════════════════

/// Marqueur pour les boutons directionnels.
#[derive(Component)]
pub struct ArrowButton {
    pub arrow: Arrow,
}

// ═══════════════════════════════════════════════════════════════════
//  Spawn
// ═══════════════════════════════════════════════════════════════════

pub fn spawn_reflex_play(mut commands: Commands, data: Res<GameData>) {
    let monster_name = data
        .minigame_monster_name
        .as_deref()
        .unwrap_or("?")
        .to_string();

    let game = match data.reflex_game.as_ref() {
        Some(g) => g,
        None => return,
    };

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::new(
                    Val::Px(16.0),
                    Val::Px(16.0),
                    Val::Px(SAFE_TOP),
                    Val::Px(SAFE_BOTTOM),
                ),
                ..default()
            },
            BackgroundColor(colors::BACKGROUND),
            ScreenEntity,
            bevy::state::state_scoped::StateScoped(GameScreen::ReflexPlay),
        ))
        .with_children(|parent| {
            // Titre
            parent.spawn((
                Text::new(format!(
                    "Reflexe ({}) -- {}",
                    game.difficulty.label(),
                    monster_name
                )),
                TextFont {
                    font_size: fonts::HEADING,
                    ..default()
                },
                TextColor(colors::ACCENT_YELLOW),
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
            ));

            // Statut (score + round)
            parent.spawn((
                Text::new(format!(
                    "Round {}/{} -- Score : {}/{}",
                    game.current_round + 1,
                    game.total_rounds,
                    game.score,
                    game.current_round
                )),
                TextFont {
                    font_size: fonts::BODY,
                    ..default()
                },
                TextColor(colors::TEXT_PRIMARY),
                Node {
                    margin: UiRect::bottom(Val::Px(16.0)),
                    ..default()
                },
                StatusText,
            ));

            // Flèche à deviner (grande)
            if !game.is_over() {
                if let Some(arrow) = game.current_arrow() {
                    parent.spawn((
                        Text::new(arrow.symbol().to_string()),
                        TextFont {
                            font_size: 80.0,
                            ..default()
                        },
                        TextColor(colors::ACCENT_YELLOW),
                        Node {
                            margin: UiRect::bottom(Val::Px(16.0)),
                            ..default()
                        },
                    ));
                }
            }

            // Historique dernier résultat
            if let Some(last) = game.results.last() {
                let (color, label) = match last {
                    RoundResult::Correct => (Color::srgb(0.3, 0.9, 0.3), "Correct !"),
                    RoundResult::Wrong => (Color::srgb(0.9, 0.3, 0.3), "Rate !"),
                };
                parent.spawn((
                    Text::new(label.to_string()),
                    TextFont {
                        font_size: fonts::BODY,
                        ..default()
                    },
                    TextColor(color),
                    Node {
                        margin: UiRect::bottom(Val::Px(16.0)),
                        ..default()
                    },
                ));
            }

            if game.is_over() {
                // Score final
                let reward = game.reward();
                let msg = if reward.is_empty() {
                    format!(
                        "{}\nScore : {}/{}",
                        game.result_label(),
                        game.score,
                        game.total_rounds
                    )
                } else {
                    format!(
                        "{}\nScore : {}/{}\n{}",
                        game.result_label(),
                        game.score,
                        game.total_rounds,
                        reward.summary()
                    )
                };
                parent.spawn((
                    Text::new(msg),
                    TextFont {
                        font_size: fonts::BODY,
                        ..default()
                    },
                    TextColor(colors::TEXT_PRIMARY),
                    Node {
                        margin: UiRect::bottom(Val::Px(16.0)),
                        ..default()
                    },
                ));
            } else {
                // 4 boutons directionnels
                parent
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|grid| {
                        // Haut
                        spawn_arrow_btn(grid, Arrow::Up);

                        // Gauche - Droite
                        grid.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(16.0),
                            ..default()
                        })
                        .with_children(|row| {
                            spawn_arrow_btn(row, Arrow::Left);
                            spawn_arrow_btn(row, Arrow::Right);
                        });

                        // Bas
                        spawn_arrow_btn(grid, Arrow::Down);
                    });
            }

            // Boutons en bas
            parent
                .spawn(Node {
                    margin: UiRect::top(Val::Px(24.0)),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(16.0),
                    ..default()
                })
                .with_children(|bar| {
                    bar.spawn((
                        Node {
                            padding: UiRect::axes(Val::Px(20.0), Val::Px(12.0)),
                            display: if game.is_over() {
                                Display::Flex
                            } else {
                                Display::None
                            },
                            ..default()
                        },
                        BackgroundColor(colors::ACCENT_YELLOW),
                        BorderRadius::all(Val::Px(8.0)),
                        ContinueButton,
                        Interaction::default(),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Continuer"),
                            TextFont {
                                font_size: fonts::BODY,
                                ..default()
                            },
                            TextColor(Color::BLACK),
                        ));
                    });

                    bar.spawn((
                        Node {
                            padding: UiRect::axes(Val::Px(20.0), Val::Px(12.0)),
                            ..default()
                        },
                        BackgroundColor(colors::PANEL),
                        BorderRadius::all(Val::Px(8.0)),
                        MinigameBackButton,
                        Interaction::default(),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Abandonner"),
                            TextFont {
                                font_size: fonts::BODY,
                                ..default()
                            },
                            TextColor(colors::TEXT_PRIMARY),
                        ));
                    });
                });
        });
}

fn spawn_arrow_btn(parent: &mut ChildBuilder, arrow: Arrow) {
    parent
        .spawn((
            Node {
                width: Val::Px(70.0),
                height: Val::Px(70.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(colors::PANEL),
            BorderRadius::all(Val::Px(8.0)),
            ArrowButton { arrow },
            Interaction::default(),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(arrow.symbol().to_string()),
                TextFont {
                    font_size: 36.0,
                    ..default()
                },
                TextColor(colors::TEXT_PRIMARY),
            ));
        });
}

// ═══════════════════════════════════════════════════════════════════
//  Input
// ═══════════════════════════════════════════════════════════════════

pub fn handle_reflex_play_input(
    mut data: ResMut<GameData>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameScreen>>,
    arrow_query: Query<(&Interaction, &ArrowButton), Changed<Interaction>>,
    back_query: Query<&Interaction, (Changed<Interaction>, With<MinigameBackButton>)>,
    continue_query: Query<&Interaction, (Changed<Interaction>, With<ContinueButton>)>,
) {
    let is_over = data
        .reflex_game
        .as_ref()
        .map(|g| g.is_over())
        .unwrap_or(true);

    // Retour / abandon
    for interaction in &back_query {
        if *interaction == Interaction::Pressed {
            clear_minigame_state(&mut data);
            next_state.set(GameScreen::MainMenu);
            return;
        }
    }

    // Continuer (fin de partie)
    for interaction in &continue_query {
        if *interaction == Interaction::Pressed && is_over {
            apply_minigame_reward(&mut data);
            clear_minigame_state(&mut data);
            next_state.set(GameScreen::MainMenu);
            return;
        }
    }

    // Toucher une flèche
    if !is_over {
        for (interaction, arrow_btn) in &arrow_query {
            if *interaction == Interaction::Pressed {
                if let Some(ref mut game) = data.reflex_game {
                    game.submit(arrow_btn.arrow);
                    // Reconstruct UI after action (Bevy respawns via StateScoped)
                    if game.is_over() || !game.is_over() {
                        // Force UI rebuild
                        next_state.set(GameScreen::ReflexPlay);
                    }
                }
            }
        }
    }

    // Clavier
    if !is_over {
        let arrow = if keyboard.just_pressed(KeyCode::ArrowUp) {
            Some(Arrow::Up)
        } else if keyboard.just_pressed(KeyCode::ArrowDown) {
            Some(Arrow::Down)
        } else if keyboard.just_pressed(KeyCode::ArrowLeft) {
            Some(Arrow::Left)
        } else if keyboard.just_pressed(KeyCode::ArrowRight) {
            Some(Arrow::Right)
        } else {
            None
        };

        if let Some(a) = arrow {
            if let Some(ref mut game) = data.reflex_game {
                game.submit(a);
                // Force UI rebuild
                next_state.set(GameScreen::ReflexPlay);
            }
        }
    } else if keyboard.just_pressed(KeyCode::Enter) {
        apply_minigame_reward(&mut data);
        clear_minigame_state(&mut data);
        next_state.set(GameScreen::MainMenu);
        return;
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        clear_minigame_state(&mut data);
        next_state.set(GameScreen::MainMenu);
    }
}
