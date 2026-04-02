//! Écran du mini-jeu Memory (paires) — Android / Bevy.

use bevy::prelude::*;
use bevy::state::state::NextState;

use monster_battle_core::minigame::memory::{CardState, MemoryGame};

use crate::game::{GameData, GameScreen, ScreenEntity};
use crate::ui::common::{colors, fonts, ScreenMetrics};

use super::{
    ContinueButton, MinigameBackButton, StatusText, apply_minigame_reward, clear_minigame_state,
};

// ═══════════════════════════════════════════════════════════════════
//  Composants
// ═══════════════════════════════════════════════════════════════════

/// Marqueur pour les cartes du Memory.
#[derive(Component)]
pub struct MemoryCard {
    pub index: usize,
}

/// Marqueur pour le texte d'une carte.
#[derive(Component)]
pub struct CardText {
    pub index: usize,
}

// ═══════════════════════════════════════════════════════════════════
//  Spawn
// ═══════════════════════════════════════════════════════════════════

pub fn spawn_memory_play(mut commands: Commands, data: Res<GameData>,
    metrics: Res<ScreenMetrics>) {
    let monster_name = data
        .minigame_monster_name
        .as_deref()
        .unwrap_or("?")
        .to_string();

    let game = match data.memory_game.as_ref() {
        Some(g) => g,
        None => return,
    };

    let cols = game.cols;
    let rows = game.rows;

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::new(
                    Val::Px(12.0),
                    Val::Px(12.0),
                    Val::Px(metrics.safe_top),
                    Val::Px(metrics.safe_bottom),
                ),
                ..default()
            },
            BackgroundColor(colors::BACKGROUND),
            ScreenEntity,
            bevy::state::state_scoped::StateScoped(GameScreen::MemoryPlay),
        ))
        .with_children(|parent| {
            // Titre
            parent.spawn((
                Text::new(format!(
                    "Memory ({}) -- {}",
                    game.difficulty.label(),
                    monster_name
                )),
                TextFont {
                    font_size: fonts::HEADING,
                    ..default()
                },
                TextColor(colors::ACCENT_YELLOW),
                Node {
                    margin: UiRect::bottom(Val::Px(4.0)),
                    ..default()
                },
            ));

            // Statut
            parent.spawn((
                Text::new(format!(
                    "Paires : {}/{} | Tentatives : {}",
                    game.pairs_found, game.total_pairs, game.attempts
                )),
                TextFont {
                    font_size: fonts::BODY,
                    ..default()
                },
                TextColor(colors::TEXT_PRIMARY),
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
                StatusText,
            ));

            // Grille de cartes
            let card_size = if cols <= 4 && rows <= 3 { 70.0 } else { 58.0 };
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|grid| {
                    for row in 0..rows {
                        grid.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(4.0),
                            ..default()
                        })
                        .with_children(|row_node| {
                            for col in 0..cols {
                                let idx = row * cols + col;
                                let (text, bg) = card_display(idx, game);

                                row_node
                                    .spawn((
                                        Node {
                                            width: Val::Px(card_size),
                                            height: Val::Px(card_size),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        BackgroundColor(bg),
                                        BorderRadius::all(Val::Px(6.0)),
                                        MemoryCard { index: idx },
                                        Interaction::default(),
                                    ))
                                    .with_children(|c| {
                                        c.spawn((
                                            Text::new(text),
                                            TextFont {
                                                font_size: if card_size > 60.0 {
                                                    32.0
                                                } else {
                                                    24.0
                                                },
                                                ..default()
                                            },
                                            TextColor(colors::TEXT_PRIMARY),
                                            CardText { index: idx },
                                        ));
                                    });
                            }
                        });
                    }
                });

            // Boutons en bas
            parent
                .spawn(Node {
                    margin: UiRect::top(Val::Px(16.0)),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(16.0),
                    ..default()
                })
                .with_children(|bar| {
                    bar.spawn((
                        Node {
                            padding: UiRect::axes(Val::Px(20.0), Val::Px(12.0)),
                            display: Display::None,
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

// ═══════════════════════════════════════════════════════════════════
//  Input
// ═══════════════════════════════════════════════════════════════════

#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_memory_play_input(
    mut data: ResMut<GameData>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameScreen>>,
    card_query: Query<(&Interaction, &MemoryCard), Changed<Interaction>>,
    back_query: Query<&Interaction, (Changed<Interaction>, With<MinigameBackButton>)>,
    continue_query: Query<&Interaction, (Changed<Interaction>, With<ContinueButton>)>,
    mut status_text: Query<&mut Text, With<StatusText>>,
    mut card_texts: Query<(&mut Text, &mut BackgroundColor, &CardText), Without<StatusText>>,
    mut continue_btn: Query<&mut Node, With<ContinueButton>>,
) {
    let is_over = data
        .memory_game
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

    // Toucher une carte
    if !is_over {
        for (interaction, card) in &card_query {
            if *interaction == Interaction::Pressed
                && let Some(ref mut game) = data.memory_game {
                    if game.needs_dismiss {
                        game.dismiss();
                    } else {
                        game.cursor = card.index;
                        game.reveal();
                    }
                }
        }
    }

    // Clavier
    if !is_over {
        if let Some(ref mut game) = data.memory_game {
            if keyboard.just_pressed(KeyCode::Enter) {
                if game.needs_dismiss {
                    game.dismiss();
                } else {
                    game.reveal();
                }
            }
            if keyboard.just_pressed(KeyCode::ArrowUp) {
                game.move_cursor_up();
            }
            if keyboard.just_pressed(KeyCode::ArrowDown) {
                game.move_cursor_down();
            }
            if keyboard.just_pressed(KeyCode::ArrowLeft) {
                game.move_cursor_left();
            }
            if keyboard.just_pressed(KeyCode::ArrowRight) {
                game.move_cursor_right();
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
        return;
    }

    // Rafraîchir l'affichage
    if let Some(ref game) = data.memory_game {
        let status_msg = if game.is_over() {
            let reward = game.reward();
            if reward.is_empty() {
                game.result_label().to_string()
            } else {
                format!("{} -- {}", game.result_label(), reward.summary())
            }
        } else {
            format!(
                "Paires : {}/{} | Tentatives : {}",
                game.pairs_found, game.total_pairs, game.attempts
            )
        };

        for mut text in &mut status_text {
            **text = status_msg.clone();
        }

        for (mut text, mut bg, card_marker) in &mut card_texts {
            let (display, color) = card_display(card_marker.index, game);
            **text = display;
            *bg = BackgroundColor(color);
        }

        if game.is_over() {
            for mut node in &mut continue_btn {
                node.display = Display::Flex;
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
//  Helpers
// ═══════════════════════════════════════════════════════════════════

fn card_display(idx: usize, game: &MemoryGame) -> (String, Color) {
    match game.states[idx] {
        CardState::Hidden => ("?".to_string(), colors::PANEL),
        CardState::Revealed => {
            let icon = game.card_icon(idx);
            (icon.to_string(), Color::srgb(0.15, 0.35, 0.55))
        }
        CardState::Matched => {
            let icon = game.card_icon(idx);
            (icon.to_string(), Color::srgb(0.2, 0.6, 0.3))
        }
    }
}
