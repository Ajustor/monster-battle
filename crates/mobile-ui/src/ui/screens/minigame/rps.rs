//! Écran du mini-jeu PPC élémentaire (Pierre-Papier-Ciseaux) — Android / Bevy.

use bevy::prelude::*;
use bevy::state::state::NextState;

use monster_battle_core::minigame::rps::RoundOutcome;

use crate::game::{GameData, GameScreen, ScreenEntity};
use crate::ui::common::{colors, fonts, ScreenMetrics};

use super::{
    ContinueButton, MinigameBackButton, StatusText, apply_minigame_reward, clear_minigame_state,
};

// ═══════════════════════════════════════════════════════════════════
//  Composants
// ═══════════════════════════════════════════════════════════════════

/// Marqueur pour les boutons de choix d'élément.
#[derive(Component)]
pub struct ElementChoiceButton {
    pub index: usize,
}

/// Marqueur pour le bouton "OK" quand on attend la confirmation.
#[derive(Component)]
pub struct ConfirmButton;

// ═══════════════════════════════════════════════════════════════════
//  Spawn
// ═══════════════════════════════════════════════════════════════════

pub fn spawn_rps_play(mut commands: Commands, data: Res<GameData>,
    metrics: Res<ScreenMetrics>) {
    let monster_name = data
        .minigame_monster_name
        .as_deref()
        .unwrap_or("?")
        .to_string();

    let game = match data.rps_game.as_ref() {
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
                    Val::Px(metrics.safe_top),
                    Val::Px(metrics.safe_bottom),
                ),
                ..default()
            },
            BackgroundColor(colors::BACKGROUND),
            ScreenEntity,
            bevy::state::state_scoped::StateScoped(GameScreen::RpsPlay),
        ))
        .with_children(|parent| {
            // Titre
            parent.spawn((
                Text::new(format!(
                    "PPC ({}) -- {}",
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

            // Score
            parent.spawn((
                Text::new(format!("Score : {}", game.score_display())),
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

            // Dernier round
            if let Some(ref round) = game.last_round {
                let (color, label) = match round.outcome {
                    RoundOutcome::PlayerWin => (Color::srgb(0.3, 0.9, 0.3), "Gagne !"),
                    RoundOutcome::AiWin => (Color::srgb(0.9, 0.3, 0.3), "Perdu !"),
                    RoundOutcome::Draw => (Color::srgb(0.9, 0.9, 0.3), "Nul !"),
                };
                parent.spawn((
                    Text::new(format!(
                        "{} {} vs {} {} -- {}",
                        round.player_choice.icon(),
                        round.player_choice,
                        round.ai_choice.icon(),
                        round.ai_choice,
                        label
                    )),
                    TextFont {
                        font_size: fonts::BODY,
                        ..default()
                    },
                    TextColor(color),
                    Node {
                        margin: UiRect::bottom(Val::Px(12.0)),
                        ..default()
                    },
                ));
            }

            if game.is_over() {
                // Résultat final
                let reward = game.reward();
                let msg = if reward.is_empty() {
                    format!("{}\n{}", game.result_label(), game.score_display())
                } else {
                    format!(
                        "{}\n{}\n{}",
                        game.result_label(),
                        game.score_display(),
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
            } else if game.waiting_confirm {
                // En attente de confirmation
                parent.spawn((
                    Text::new("Appuyez pour continuer..."),
                    TextFont {
                        font_size: fonts::BODY,
                        ..default()
                    },
                    TextColor(colors::TEXT_SECONDARY),
                    Node {
                        margin: UiRect::bottom(Val::Px(12.0)),
                        ..default()
                    },
                ));

                parent
                    .spawn((
                        Node {
                            padding: UiRect::axes(Val::Px(24.0), Val::Px(14.0)),
                            margin: UiRect::bottom(Val::Px(16.0)),
                            ..default()
                        },
                        BackgroundColor(colors::ACCENT_YELLOW),
                        BorderRadius::all(Val::Px(8.0)),
                        ConfirmButton,
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
            } else {
                // Choix des 3 éléments
                parent.spawn((
                    Text::new("Choisissez un element :"),
                    TextFont {
                        font_size: fonts::BODY,
                        ..default()
                    },
                    TextColor(colors::TEXT_SECONDARY),
                    Node {
                        margin: UiRect::bottom(Val::Px(8.0)),
                        ..default()
                    },
                ));

                parent
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(12.0),
                        margin: UiRect::bottom(Val::Px(16.0)),
                        ..default()
                    })
                    .with_children(|row| {
                        let choices = game.choices();
                        for (i, elem) in choices.iter().enumerate() {
                            row.spawn((
                                Node {
                                    width: Val::Px(100.0),
                                    padding: UiRect::all(Val::Px(12.0)),
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(colors::PANEL),
                                BorderRadius::all(Val::Px(8.0)),
                                ElementChoiceButton { index: i },
                                Interaction::default(),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new(elem.icon().to_string()),
                                    TextFont {
                                        font_size: 32.0,
                                        ..default()
                                    },
                                    TextColor(colors::TEXT_PRIMARY),
                                ));
                                btn.spawn((
                                    Text::new(format!("{}", elem)),
                                    TextFont {
                                        font_size: fonts::SMALL,
                                        ..default()
                                    },
                                    TextColor(colors::TEXT_SECONDARY),
                                ));
                            });
                        }
                    });
            }

            // Boutons en bas
            parent
                .spawn(Node {
                    margin: UiRect::top(Val::Px(8.0)),
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
                            Text::new("Recolter"),
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
pub(crate) fn handle_rps_play_input(
    mut commands: Commands,
    mut data: ResMut<GameData>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameScreen>>,
    choice_query: Query<(&Interaction, &ElementChoiceButton), Changed<Interaction>>,
    confirm_query: Query<&Interaction, (Changed<Interaction>, With<ConfirmButton>)>,
    back_query: Query<&Interaction, (Changed<Interaction>, With<MinigameBackButton>)>,
    continue_query: Query<&Interaction, (Changed<Interaction>, With<ContinueButton>)>,
    screen_entities: Query<Entity, With<ScreenEntity>>,
    metrics: Res<ScreenMetrics>,
) {
    let is_over = data.rps_game.as_ref().map(|g| g.is_over()).unwrap_or(true);
    let waiting_confirm = data
        .rps_game
        .as_ref()
        .map(|g| g.waiting_confirm)
        .unwrap_or(false);

    // Retour / abandon
    for interaction in &back_query {
        if *interaction == Interaction::Pressed {
            clear_minigame_state(&mut data);
            next_state.set(GameScreen::MainMenu);
            return;
        }
    }

    // Continuer (fin de partie → récolter)
    for interaction in &continue_query {
        if *interaction == Interaction::Pressed && is_over {
            apply_minigame_reward(&mut data);
            clear_minigame_state(&mut data);
            next_state.set(GameScreen::MainMenu);
            return;
        }
    }

    let mut needs_rebuild = false;

    // Confirmer après un round
    for interaction in &confirm_query {
        if *interaction == Interaction::Pressed && waiting_confirm {
            if let Some(ref mut game) = data.rps_game {
                game.confirm();
                needs_rebuild = true;
            }
            break;
        }
    }

    // Toucher un choix d'élément
    if !is_over && !waiting_confirm && !needs_rebuild {
        for (interaction, choice) in &choice_query {
            if *interaction == Interaction::Pressed {
                if let Some(ref mut game) = data.rps_game {
                    game.play(choice.index);
                    needs_rebuild = true;
                }
                break;
            }
        }
    }

    // Clavier
    if is_over {
        if keyboard.just_pressed(KeyCode::Enter) {
            apply_minigame_reward(&mut data);
            clear_minigame_state(&mut data);
            next_state.set(GameScreen::MainMenu);
            return;
        }
    } else if waiting_confirm && !needs_rebuild {
        if keyboard.just_pressed(KeyCode::Enter)
            && let Some(ref mut game) = data.rps_game {
                game.confirm();
                needs_rebuild = true;
            }
    } else if !needs_rebuild {
        // Sélection avec touches numériques
        let choice = if keyboard.just_pressed(KeyCode::Digit1) {
            Some(0)
        } else if keyboard.just_pressed(KeyCode::Digit2) {
            Some(1)
        } else if keyboard.just_pressed(KeyCode::Digit3) {
            Some(2)
        } else {
            None
        };

        if let Some(idx) = choice
            && let Some(ref mut game) = data.rps_game {
                game.play(idx);
                needs_rebuild = true;
            }
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        clear_minigame_state(&mut data);
        next_state.set(GameScreen::MainMenu);
        return;
    }

    // Reconstruire l'UI après une action (despawn + respawn)
    if needs_rebuild {
        for entity in &screen_entities {
            commands.entity(entity).despawn_recursive();
        }
        spawn_rps_play(commands, data.into(), metrics);
    }
}
