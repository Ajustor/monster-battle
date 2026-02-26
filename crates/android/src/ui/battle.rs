//! Écran de combat interactif.

use bevy::prelude::*;
use bevy::state::state::NextState;

use crate::game::{GameData, GameScreen, ScreenEntity};
use crate::sprites;
use crate::ui::common::{self, colors, fonts};
use monster_battle_core::battle::BattlePhase;

/// Construit l'UI de combat.
pub fn spawn_battle_ui(
    mut commands: Commands,
    data: Res<GameData>,
    mut images: ResMut<Assets<Image>>,
    mut atlas: ResMut<sprites::MonsterSpriteAtlas>,
) {
    let battle = match &data.battle_state {
        Some(b) => b,
        None => return,
    };

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(colors::BACKGROUND),
            ScreenEntity,
        ))
        .with_children(|root| {
            // ── Zone adversaire (haut) ───────────────────────────
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(12.0),
                margin: UiRect::bottom(Val::Px(16.0)),
                ..default()
            })
            .with_children(|top| {
                // Sprite adversaire (de face)
                let grid = sprites::get_pixel_sprite(
                    battle.opponent.element,
                    battle.opponent.secondary_element,
                );
                let handle = atlas.get_or_create_front(
                    battle.opponent.element,
                    battle.opponent.secondary_element,
                    grid,
                    &mut images,
                );

                top.spawn((
                    ImageNode::new(handle),
                    Node {
                        width: Val::Px(96.0),
                        height: Val::Px(96.0),
                        ..default()
                    },
                ));

                // Stats adversaire
                top.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|info| {
                    info.spawn((
                        Text::new(format!(
                            "{} Nv.{}",
                            battle.opponent.name, battle.opponent.level,
                        )),
                        TextFont {
                            font_size: fonts::BODY,
                            ..default()
                        },
                        TextColor(colors::TEXT_PRIMARY),
                    ));

                    let hp_color =
                        common::hp_color(battle.opponent.current_hp, battle.opponent.max_hp);
                    info.spawn((
                        Text::new(format!(
                            "PV {}/{}",
                            battle.opponent.current_hp, battle.opponent.max_hp,
                        )),
                        TextFont {
                            font_size: fonts::SMALL,
                            ..default()
                        },
                        TextColor(hp_color),
                    ));
                });
            });

            // ── Zone joueur (bas) ────────────────────────────────
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(12.0),
                justify_content: JustifyContent::FlexEnd,
                ..default()
            })
            .with_children(|bottom| {
                // Stats joueur
                bottom
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexEnd,
                        ..default()
                    })
                    .with_children(|info| {
                        info.spawn((
                            Text::new(
                                format!("{} Nv.{}", battle.player.name, battle.player.level,),
                            ),
                            TextFont {
                                font_size: fonts::BODY,
                                ..default()
                            },
                            TextColor(colors::TEXT_PRIMARY),
                        ));

                        let hp_color =
                            common::hp_color(battle.player.current_hp, battle.player.max_hp);
                        info.spawn((
                            Text::new(format!(
                                "PV {}/{}",
                                battle.player.current_hp, battle.player.max_hp,
                            )),
                            TextFont {
                                font_size: fonts::SMALL,
                                ..default()
                            },
                            TextColor(hp_color),
                        ));
                    });

                // Sprite joueur (de dos)
                let grid = sprites::get_pixel_back_sprite(
                    battle.player.element,
                    battle.player.secondary_element,
                );
                let handle = atlas.get_or_create_back(
                    battle.player.element,
                    battle.player.secondary_element,
                    grid,
                    &mut images,
                );

                bottom.spawn((
                    ImageNode::new(handle),
                    Node {
                        width: Val::Px(96.0),
                        height: Val::Px(96.0),
                        ..default()
                    },
                ));
            });

            // ── Zone actions / messages (bas) ────────────────────
            root.spawn(Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            })
            .with_children(|actions| {
                // Message courant
                if let Some(ref msg) = battle.current_message {
                    actions.spawn((
                        Text::new(msg.text.clone()),
                        TextFont {
                            font_size: fonts::BODY,
                            ..default()
                        },
                        TextColor(colors::TEXT_PRIMARY),
                        Node {
                            margin: UiRect::bottom(Val::Px(8.0)),
                            ..default()
                        },
                    ));
                }

                // Boutons d'attaque si c'est au joueur de choisir
                if battle.phase == BattlePhase::PlayerChooseAttack {
                    for (i, attack) in battle.player.attacks.iter().enumerate() {
                        let selected = i == battle.attack_menu_index;
                        let bg = if selected {
                            colors::ACCENT_YELLOW
                        } else {
                            colors::PANEL
                        };
                        let txt_color = if selected {
                            Color::BLACK
                        } else {
                            colors::TEXT_PRIMARY
                        };

                        actions
                            .spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
                                    margin: UiRect::bottom(Val::Px(4.0)),
                                    ..default()
                                },
                                BackgroundColor(bg),
                                BorderRadius::all(Val::Px(6.0)),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new(format!(
                                        "{} {}  (Puissance: {})",
                                        attack.element.icon(),
                                        attack.name,
                                        attack.power,
                                    )),
                                    TextFont {
                                        font_size: fonts::BODY,
                                        ..default()
                                    },
                                    TextColor(txt_color),
                                ));
                            });
                    }
                }
            });
        });
}

/// Gestion des entrées en combat.
pub fn handle_battle_input(
    mut data: ResMut<GameData>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameScreen>>,
) {
    let battle = match &mut data.battle_state {
        Some(b) => b,
        None => {
            next_state.set(GameScreen::MainMenu);
            return;
        }
    };

    match battle.phase {
        BattlePhase::PlayerChooseAttack => {
            let attack_count = battle.player.attacks.len();
            if keyboard.just_pressed(KeyCode::ArrowUp) && battle.attack_menu_index > 0 {
                battle.attack_menu_index -= 1;
            }
            if keyboard.just_pressed(KeyCode::ArrowDown)
                && battle.attack_menu_index < attack_count.saturating_sub(1)
            {
                battle.attack_menu_index += 1;
            }
            if keyboard.just_pressed(KeyCode::Enter) {
                // L'attaque sera traitée par le système de combat.
            }
            if keyboard.just_pressed(KeyCode::Escape) {
                data.battle_state = None;
                next_state.set(GameScreen::MainMenu);
            }
        }
        BattlePhase::Victory | BattlePhase::Defeat => {
            if keyboard.just_pressed(KeyCode::Enter) {
                data.battle_state = None;
                next_state.set(GameScreen::MainMenu);
            }
        }
        _ => {
            if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space) {
                // Avancer dans les messages.
            }
            if keyboard.just_pressed(KeyCode::Escape) {
                data.battle_state = None;
                next_state.set(GameScreen::MainMenu);
            }
        }
    }
}
