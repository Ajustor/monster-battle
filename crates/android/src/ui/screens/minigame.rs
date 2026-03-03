//! Écran du mini-jeu morpion (Tic-Tac-Toe).

use bevy::prelude::*;
use bevy::state::state::NextState;

use monster_battle_core::minigame::apply_reward;
use monster_battle_core::minigame::tictactoe::{Cell, Difficulty, TicTacToe};
use monster_battle_storage::MonsterStorage;

use crate::game::{GameData, GameScreen, ScreenEntity};
use crate::ui::common::{SAFE_BOTTOM, SAFE_TOP, colors, fonts};

// ═══════════════════════════════════════════════════════════════════
//  Composants
// ═══════════════════════════════════════════════════════════════════

/// Marqueur pour les boutons de difficulté.
#[derive(Component)]
pub(crate) struct DifficultyButton {
    pub difficulty: Difficulty,
}

/// Marqueur pour le bouton retour.
#[derive(Component)]
pub(crate) struct MinigameBackButton;

/// Marqueur pour les cases du morpion.
#[derive(Component)]
pub(crate) struct BoardCell {
    pub index: usize,
}

/// Marqueur pour le bouton "Continuer" après fin de partie.
#[derive(Component)]
pub(crate) struct ContinueButton;

/// Marqueur pour le texte de statut.
#[derive(Component)]
pub(crate) struct StatusText;

/// Marqueur pour le texte d'une case.
#[derive(Component)]
pub(crate) struct CellText {
    pub index: usize,
}

// ═══════════════════════════════════════════════════════════════════
//  Sélection de la difficulté
// ═══════════════════════════════════════════════════════════════════

pub(crate) fn spawn_minigame_select(mut commands: Commands, data: Res<GameData>) {
    let monster_name = data
        .minigame_monster_name
        .as_deref()
        .unwrap_or("?")
        .to_string();

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
                Text::new(format!("Morpion -- {}", monster_name)),
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

            // Boutons de difficulté
            for d in Difficulty::all() {
                let desc = match d {
                    Difficulty::Easy => "IA aleatoire -- recompense faible",
                    Difficulty::Medium => "IA mixte -- recompense moyenne",
                    Difficulty::Hard => "IA imbattable -- recompense elevee",
                };

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
                        DifficultyButton { difficulty: *d },
                        Interaction::default(),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(d.label().to_string()),
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

pub(crate) fn handle_minigame_select_input(
    mut data: ResMut<GameData>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameScreen>>,
    difficulty_query: Query<(&Interaction, &DifficultyButton), Changed<Interaction>>,
    back_query: Query<&Interaction, (Changed<Interaction>, With<MinigameBackButton>)>,
) {
    // Toucher retour
    for interaction in &back_query {
        if *interaction == Interaction::Pressed {
            data.tictactoe = None;
            data.minigame_monster_id = None;
            data.minigame_monster_name = None;
            next_state.set(GameScreen::MainMenu);
            data.menu_index = 0;
            return;
        }
    }

    // Toucher difficulté
    for (interaction, btn) in &difficulty_query {
        if *interaction == Interaction::Pressed {
            data.tictactoe = Some(TicTacToe::new(btn.difficulty));
            next_state.set(GameScreen::MinigamePlay);
            return;
        }
    }

    // Clavier
    if keyboard.just_pressed(KeyCode::Escape) {
        data.tictactoe = None;
        data.minigame_monster_id = None;
        data.minigame_monster_name = None;
        next_state.set(GameScreen::MainMenu);
        data.menu_index = 0;
    }
}

// ═══════════════════════════════════════════════════════════════════
//  Plateau de jeu
// ═══════════════════════════════════════════════════════════════════

pub(crate) fn spawn_minigame_play(mut commands: Commands, data: Res<GameData>) {
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

            // Bouton retour (en bas)
            parent
                .spawn(Node {
                    margin: UiRect::top(Val::Px(24.0)),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(16.0),
                    ..default()
                })
                .with_children(|bar| {
                    // Bouton continuer (caché initialement, affiché à la fin)
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

                    // Bouton abandon
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

pub(crate) fn handle_minigame_play_input(
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
            data.tictactoe = None;
            data.minigame_monster_id = None;
            data.minigame_monster_name = None;
            next_state.set(GameScreen::MainMenu);
            data.menu_index = 0;
            return;
        }
    }

    // Toucher continuer (fin de partie)
    for interaction in &continue_query {
        if *interaction == Interaction::Pressed && is_over {
            apply_minigame_reward(&mut data);
            data.tictactoe = None;
            data.minigame_monster_id = None;
            data.minigame_monster_name = None;
            next_state.set(GameScreen::MainMenu);
            data.menu_index = 0;
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
        data.tictactoe = None;
        data.minigame_monster_id = None;
        data.minigame_monster_name = None;
        next_state.set(GameScreen::MainMenu);
        data.menu_index = 0;
        return;
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        data.tictactoe = None;
        data.minigame_monster_id = None;
        data.minigame_monster_name = None;
        next_state.set(GameScreen::MainMenu);
        data.menu_index = 0;
        return;
    }

    // Rafraîchir l'affichage
    if let Some(ref game) = data.tictactoe {
        // Mettre à jour le statut
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

        // Mettre à jour les cases
        let winning = game.winning_line.unwrap_or([usize::MAX; 3]);
        for (mut text, mut bg, cell_marker) in &mut cell_texts {
            let cell = game.board[cell_marker.index];
            let is_winning = winning.contains(&cell_marker.index);
            **text = cell.symbol().to_string();
            *bg = BackgroundColor(cell_bg(cell, is_winning));
        }

        // Afficher le bouton continuer si la partie est finie
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

/// Applique la récompense du mini-jeu au monstre sélectionné et sauvegarde.
fn apply_minigame_reward(data: &mut ResMut<GameData>) {
    let Some(ref game) = data.tictactoe else {
        return;
    };
    let reward = game.reward();
    if reward.is_empty() {
        data.message = Some(format!("{} -- Pas de recompense.", game.result_label()));
        return;
    }

    let Some(monster_id) = data.minigame_monster_id else {
        return;
    };

    if let Ok(mut monsters) = data.storage.list_alive() {
        if let Some(m) = monsters.iter_mut().find(|m| m.id == monster_id) {
            apply_reward(&mut m.base_stats, &reward);
            let levels = m.gain_xp(reward.xp);
            let mut msg = format!("{} -- {}", game.result_label(), reward.summary());
            if levels > 0 {
                msg.push_str(&format!(
                    " +{} niveau{} !",
                    levels,
                    if levels > 1 { "x" } else { "" }
                ));
            }
            data.message = Some(msg);
            let _ = data.storage.save(m);
        }
    }
}
