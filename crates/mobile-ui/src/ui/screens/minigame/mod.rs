//! Écrans des mini-jeux — un sous-module par jeu.

pub mod memory;
pub mod reflex;
pub mod rps;
pub mod tictactoe;

use bevy::prelude::*;
use bevy::state::state::NextState;

use monster_battle_core::minigame::MinigameType;
use monster_battle_core::minigame::apply_reward;
use monster_battle_storage::MonsterStorage;

use crate::game::{GameData, GameScreen, ScreenEntity};
use crate::ui::common::{colors, fonts, ScreenMetrics};

// Re-exports pour ui/mod.rs (pub car le crate est une lib)
pub use memory::handle_memory_play_input;
pub use memory::spawn_memory_play;
pub use reflex::handle_reflex_play_input;
pub use reflex::spawn_reflex_play;
pub use rps::handle_rps_play_input;
pub use rps::spawn_rps_play;
pub use tictactoe::handle_minigame_play_input;
pub use tictactoe::handle_minigame_select_input;
pub use tictactoe::spawn_minigame_play;
pub use tictactoe::spawn_minigame_select;

// ═══════════════════════════════════════════════════════════════════
//  Composants partagés
// ═══════════════════════════════════════════════════════════════════

/// Marqueur pour le bouton retour.
#[derive(Component)]
pub(crate) struct MinigameBackButton;

/// Marqueur pour le bouton "Continuer" après fin de partie.
#[derive(Component)]
pub(crate) struct ContinueButton;

/// Marqueur pour le texte de statut.
#[derive(Component)]
pub(crate) struct StatusText;

/// Marqueur pour les boutons de type de jeu.
#[derive(Component)]
pub(crate) struct GameTypeButton {
    pub game_type: MinigameType,
}

// ═══════════════════════════════════════════════════════════════════
//  Sélection du type de mini-jeu
// ═══════════════════════════════════════════════════════════════════

pub fn spawn_minigame_type_select(mut commands: Commands, data: Res<GameData>,
    metrics: Res<ScreenMetrics>) {
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
                    Val::Px(metrics.safe_top),
                    Val::Px(metrics.safe_bottom),
                ),
                ..default()
            },
            BackgroundColor(colors::BACKGROUND),
            ScreenEntity,
            bevy::state::state_scoped::StateScoped(GameScreen::MinigameTypeSelect),
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
                Text::new(format!("Mini-jeux -- {}", monster_name)),
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
                Text::new("Choisir un mini-jeu"),
                TextFont {
                    font_size: fonts::BODY,
                    ..default()
                },
                TextColor(colors::TEXT_SECONDARY),
                Node {
                    margin: UiRect::bottom(Val::Px(16.0)),
                    ..default()
                },
            ));

            // Boutons de type de jeu
            for t in MinigameType::all() {
                parent
                    .spawn((
                        Node {
                            width: Val::Percent(90.0),
                            padding: UiRect::all(Val::Px(14.0)),
                            margin: UiRect::bottom(Val::Px(10.0)),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(colors::PANEL),
                        BorderRadius::all(Val::Px(8.0)),
                        GameTypeButton { game_type: *t },
                        Interaction::default(),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(format!("{} {}", t.icon(), t.label())),
                            TextFont {
                                font_size: fonts::BODY,
                                ..default()
                            },
                            TextColor(colors::TEXT_PRIMARY),
                        ));
                        btn.spawn((
                            Text::new(format!("{} ({})", t.description(), t.stat_focus())),
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

pub fn handle_minigame_type_select_input(
    mut data: ResMut<GameData>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameScreen>>,
    type_query: Query<(&Interaction, &GameTypeButton), Changed<Interaction>>,
    back_query: Query<&Interaction, (Changed<Interaction>, With<MinigameBackButton>)>,
) {
    // Toucher retour
    for interaction in &back_query {
        if *interaction == Interaction::Pressed {
            data.minigame_type = None;
            data.minigame_monster_id = None;
            data.minigame_monster_name = None;
            next_state.set(GameScreen::MainMenu);
            data.menu_index = 0;
            return;
        }
    }

    // Toucher type de jeu
    for (interaction, btn) in &type_query {
        if *interaction == Interaction::Pressed {
            data.minigame_type = Some(btn.game_type);
            data.menu_index = 0;
            next_state.set(GameScreen::MinigameSelect);
            return;
        }
    }

    // Clavier
    if keyboard.just_pressed(KeyCode::Escape) {
        data.minigame_type = None;
        data.minigame_monster_id = None;
        data.minigame_monster_name = None;
        next_state.set(GameScreen::MainMenu);
        data.menu_index = 0;
    }
}

// ═══════════════════════════════════════════════════════════════════
//  Récompenses (partagé par tous les jeux)
// ═══════════════════════════════════════════════════════════════════

/// Applique la récompense du mini-jeu au monstre sélectionné et sauvegarde.
pub(crate) fn apply_minigame_reward(data: &mut ResMut<GameData>) {
    let reward = if let Some(ref game) = data.tictactoe {
        let r = game.reward();
        let label = game.result_label().to_string();
        (r, label)
    } else if let Some(ref game) = data.memory_game {
        let r = game.reward();
        let label = game.result_label().to_string();
        (r, label)
    } else if let Some(ref game) = data.reflex_game {
        let r = game.reward();
        let label = game.result_label().to_string();
        (r, label)
    } else if let Some(ref game) = data.rps_game {
        let r = game.reward();
        let label = game.result_label().to_string();
        (r, label)
    } else {
        return;
    };

    let (reward, label) = reward;
    if reward.is_empty() {
        data.message = Some(format!("{} -- Pas de recompense.", label));
        return;
    }

    let Some(monster_id) = data.minigame_monster_id else {
        return;
    };

    if let Ok(mut monsters) = data.storage.list_alive() {
        if let Some(m) = monsters.iter_mut().find(|m| m.id == monster_id) {
            apply_reward(&mut m.base_stats, &reward);
            let levels = m.gain_xp(reward.xp);
            m.adjust_happiness(10);
            m.record_interaction();
            m.increase_bond(1);
            let mut msg = format!("{} -- {}", label, reward.summary());
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

/// Nettoie tous les états de mini-jeu.
pub(crate) fn clear_minigame_state(data: &mut ResMut<GameData>) {
    data.minigame_type = None;
    data.tictactoe = None;
    data.memory_game = None;
    data.reflex_game = None;
    data.rps_game = None;
    data.minigame_monster_id = None;
    data.minigame_monster_name = None;
    data.menu_index = 0;
}
