//! Écran du mini-jeu morpion (Tic-Tac-Toe) — Android / Bevy.

use bevy::prelude::*;
use bevy::state::state::NextState;

use monster_battle_core::minigame::MinigameType;
use monster_battle_core::minigame::tictactoe::{Cell, Difficulty, TicTacToe};
use monster_battle_storage::MonsterStorage;

use crate::game::{GameData, GameScreen, ScreenEntity};
use crate::ui::common::{SAFE_BOTTOM, SAFE_TOP, colors, fonts};

use super::{
    ContinueButton, MinigameBackButton, StatusText, apply_minigame_reward, clear_minigame_state,
};

// ═══════════════════════════════════════════════════════════════════
//  Composants
// ═══════════════════════════════════════════════════════════════════

/// Marqueur pour les boutons de difficulté.
#[derive(Component)]
pub(crate) struct DifficultyButton {
    pub difficulty: Difficulty,
}

/// Marqueur pour les cases du morpion.
#[derive(Component)]
pub(crate) struct BoardCell {
    pub index: usize,
}

/// Marqueur pour le texte d'une case.
#[derive(Component)]
pub(crate) struct CellText {
    pub index: usize,
}

// ═══════════════════════════════════════════════════════════════════
//  Sélection de la difficulté
// ═══════════════════════════════════════════════════════════════════

pub fn spawn_minigame_select(mut commands: Commands, data: Res<GameData>) {
    let monster_name = data
        .minigame_monster_name
        .as_deref()
        .unwrap_or("?")
        .to_string();

    let game_type = data.minigame_type.unwrap_or(MinigameType::TicTacToe);

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
            bevy::state::state_scoped::StateScoped(GameScreen::MinigameSelect),
        ))
        .with_children(|parent| {
            // Bouton retour
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
                        MinigameBackButton,
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
                Text::new(format!(
                    "{} {} -- {}",
                    game_type.icon(),
                    game_type.label(),
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

            parent.spawn((
                Text::new("Choisir la difficulte"),
                TextFont {
                    font_size: fonts::BODY,
                    ..default()
                },
                TextColor(colors::TEXT_SECONDARY),
                Node {
                    margin: UiRect::bottom(Val::Px(24.0)),
                    ..default()
                },
            ));

            // Boutons de difficulté (selon le type de jeu)
            let difficulties = difficulty_labels(game_type);
            for (label, desc, idx) in difficulties {
                parent
                    .spawn((
                        Node {
                            width: Val::Percent(85.0),
                            padding: UiRect::all(Val::Px(14.0)),
                            margin: UiRect::bottom(Val::Px(10.0)),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(colors::PANEL),
                        BorderRadius::all(Val::Px(8.0)),
                        DifficultyButton {
                            difficulty: idx_to_difficulty(idx),
                        },
                        Interaction::default(),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(label.to_string()),
                            TextFont {
                                font_size: fonts::BODY,
                                ..default()
                            },
                            TextColor(colors::TEXT_PRIMARY),
                        ));
                        btn.spawn((
                            Text::new(desc.to_string()),
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

fn difficulty_labels(game_type: MinigameType) -> Vec<(&'static str, &'static str, usize)> {
    match game_type {
        MinigameType::TicTacToe => vec![
            ("Facile", "IA aleatoire -- recompense faible", 0),
            ("Moyen", "IA mixte -- recompense moyenne", 1),
            ("Difficile", "IA imbattable -- recompense elevee", 2),
        ],
        MinigameType::Memory => vec![
            ("Facile", "Grille 4x3 -- recompense faible", 0),
            ("Moyen", "Grille 4x4 -- recompense moyenne", 1),
            ("Difficile", "Grille 4x5 -- recompense elevee", 2),
        ],
        MinigameType::Reflex => vec![
            ("Facile", "8 rounds -- recompense faible", 0),
            ("Moyen", "12 rounds -- recompense moyenne", 1),
            ("Difficile", "16 rounds -- recompense elevee", 2),
        ],
        MinigameType::Rps => vec![
            ("Facile", "BO3 -- recompense faible", 0),
            ("Moyen", "BO5 -- recompense moyenne", 1),
            ("Difficile", "BO7 -- recompense elevee", 2),
        ],
    }
}

fn idx_to_difficulty(idx: usize) -> Difficulty {
    match idx {
        0 => Difficulty::Easy,
        1 => Difficulty::Medium,
        _ => Difficulty::Hard,
    }
}

pub fn handle_minigame_select_input(
    mut data: ResMut<GameData>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameScreen>>,
    difficulty_query: Query<(&Interaction, &DifficultyButton), Changed<Interaction>>,
    back_query: Query<&Interaction, (Changed<Interaction>, With<MinigameBackButton>)>,
) {
    // Toucher retour
    for interaction in &back_query {
        if *interaction == Interaction::Pressed {
            next_state.set(GameScreen::MinigameTypeSelect);
            data.menu_index = 0;
            return;
        }
    }

    // Toucher difficulté
    for (interaction, btn) in &difficulty_query {
        if *interaction == Interaction::Pressed {
            let game_type = data.minigame_type.unwrap_or(MinigameType::TicTacToe);
            start_game(&mut data, &mut next_state, game_type, btn.difficulty);
            return;
        }
    }

    // Clavier
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameScreen::MinigameTypeSelect);
        data.menu_index = 0;
    }
}

fn start_game(
    data: &mut ResMut<GameData>,
    next_state: &mut ResMut<NextState<GameScreen>>,
    game_type: MinigameType,
    difficulty: Difficulty,
) {
    match game_type {
        MinigameType::TicTacToe => {
            data.tictactoe = Some(TicTacToe::new(difficulty));
            next_state.set(GameScreen::MinigamePlay);
        }
        MinigameType::Memory => {
            use monster_battle_core::minigame::memory;
            let d = match difficulty {
                Difficulty::Easy => memory::Difficulty::Easy,
                Difficulty::Medium => memory::Difficulty::Medium,
                Difficulty::Hard => memory::Difficulty::Hard,
            };
            data.memory_game = Some(memory::MemoryGame::new(d));
            next_state.set(GameScreen::MemoryPlay);
        }
        MinigameType::Reflex => {
            use monster_battle_core::minigame::reflex;
            let d = match difficulty {
                Difficulty::Easy => reflex::Difficulty::Easy,
                Difficulty::Medium => reflex::Difficulty::Medium,
                Difficulty::Hard => reflex::Difficulty::Hard,
            };
            data.reflex_game = Some(reflex::ReflexGame::new(d));
            next_state.set(GameScreen::ReflexPlay);
        }
        MinigameType::Rps => {
            use monster_battle_core::minigame::rps;
            let d = match difficulty {
                Difficulty::Easy => rps::Difficulty::Easy,
                Difficulty::Medium => rps::Difficulty::Medium,
                Difficulty::Hard => rps::Difficulty::Hard,
            };
            // Récupérer le type du monstre pour le bonus
            let monster_type = data
                .minigame_monster_id
                .and_then(|id| {
                    data.storage
                        .list_alive()
                        .ok()
                        .and_then(|ms| ms.into_iter().find(|m| m.id == id))
                })
                .map(|m| m.primary_type)
                .unwrap_or(monster_battle_core::types::ElementType::Fire);
            data.rps_game = Some(rps::RpsGame::new(d, monster_type));
            next_state.set(GameScreen::RpsPlay);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
//  Plateau de jeu morpion
// ═══════════════════════════════════════════════════════════════════

pub fn spawn_minigame_play(mut commands: Commands, data: Res<GameData>) {
    let monster_name = data
        .minigame_monster_name
        .as_deref()
        .unwrap_or("?")
        .to_string();
    let difficulty_label = data
        .tictactoe
        .as_ref()
        .map(|g| g.difficulty.label())
        .unwrap_or("?");

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
            bevy::state::state_scoped::StateScoped(GameScreen::MinigamePlay),
        ))
        .with_children(|parent| {
            // Titre
            parent.spawn((
                Text::new(format!(
                    "Morpion ({}) -- {}",
                    difficulty_label, monster_name
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

            // Statut
            parent.spawn((
                Text::new("A vous de jouer !"),
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

            // Grille 3×3
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|grid| {
                    for row in 0..3 {
                        grid.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(4.0),
                            ..default()
                        })
                        .with_children(|row_node| {
                            for col in 0..3 {
                                let idx = row * 3 + col;
                                let cell = data
                                    .tictactoe
                                    .as_ref()
                                    .map(|g| g.board[idx])
                                    .unwrap_or(Cell::Empty);

                                let bg = cell_bg(cell, false);

                                row_node
                                    .spawn((
                                        Node {
                                            width: Val::Px(80.0),
                                            height: Val::Px(80.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        BackgroundColor(bg),
                                        BorderRadius::all(Val::Px(6.0)),
                                        BoardCell { index: idx },
                                        Interaction::default(),
                                    ))
                                    .with_children(|c| {
                                        c.spawn((
                                            Text::new(cell.symbol().to_string()),
                                            TextFont {
                                                font_size: 40.0,
                                                ..default()
                                            },
                                            TextColor(cell_text_color(cell)),
                                            CellText { index: idx },
                                        ));
                                    });
                            }
                        });
                    }
                });

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

pub fn handle_minigame_play_input(
    mut data: ResMut<GameData>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameScreen>>,
    cell_query: Query<(&Interaction, &BoardCell), Changed<Interaction>>,
    back_query: Query<&Interaction, (Changed<Interaction>, With<MinigameBackButton>)>,
    continue_query: Query<&Interaction, (Changed<Interaction>, With<ContinueButton>)>,
    mut status_text: Query<&mut Text, With<StatusText>>,
    mut cell_texts: Query<(&mut Text, &mut BackgroundColor, &CellText), Without<StatusText>>,
    mut continue_btn: Query<&mut Node, With<ContinueButton>>,
) {
    let is_over = data.tictactoe.as_ref().map(|g| g.is_over()).unwrap_or(true);

    // Toucher retour / abandon
    for interaction in &back_query {
        if *interaction == Interaction::Pressed {
            clear_minigame_state(&mut data);
            next_state.set(GameScreen::MainMenu);
            return;
        }
    }

    // Toucher continuer (fin de partie)
    for interaction in &continue_query {
        if *interaction == Interaction::Pressed && is_over {
            apply_minigame_reward(&mut data);
            clear_minigame_state(&mut data);
            next_state.set(GameScreen::MainMenu);
            return;
        }
    }

    // Toucher une case (mobile)
    if !is_over {
        for (interaction, cell) in &cell_query {
            if *interaction == Interaction::Pressed {
                if let Some(ref mut game) = data.tictactoe {
                    game.cursor = cell.index;
                    game.play();
                }
            }
        }
    }

    // Clavier
    if !is_over {
        if let Some(ref mut game) = data.tictactoe {
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
            if keyboard.just_pressed(KeyCode::Enter) {
                game.play();
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
    if let Some(ref game) = data.tictactoe {
        let status_msg = if game.is_over() {
            let reward = game.reward();
            if reward.is_empty() {
                format!("{}", game.result_label())
            } else {
                format!("{} -- {}", game.result_label(), reward.summary())
            }
        } else {
            "A vous de jouer !".to_string()
        };

        for mut text in &mut status_text {
            **text = status_msg.clone();
        }

        let winning = game.winning_line.unwrap_or([usize::MAX; 3]);
        for (mut text, mut bg, cell_marker) in &mut cell_texts {
            let cell = game.board[cell_marker.index];
            let is_winning = winning.contains(&cell_marker.index);
            **text = cell.symbol().to_string();
            *bg = BackgroundColor(cell_bg(cell, is_winning));
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

fn cell_bg(cell: Cell, is_winning: bool) -> Color {
    if is_winning {
        Color::srgb(0.2, 0.7, 0.3)
    } else {
        match cell {
            Cell::Empty => colors::PANEL,
            Cell::X => Color::srgb(0.15, 0.3, 0.5),
            Cell::O => Color::srgb(0.5, 0.15, 0.15),
        }
    }
}

fn cell_text_color(cell: Cell) -> Color {
    match cell {
        Cell::Empty => colors::TEXT_SECONDARY,
        Cell::X => Color::srgb(0.4, 0.8, 1.0),
        Cell::O => Color::srgb(1.0, 0.4, 0.4),
    }
}
