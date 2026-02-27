//! Écran de combat interactif.

use bevy::prelude::*;
use bevy::state::state::NextState;

use crate::game::{GameData, GameScreen, ScreenEntity};
use crate::sprites;
use crate::ui::common::{self, SAFE_TOP, colors, fonts};
use monster_battle_core::battle::BattlePhase;

/// Marqueur pour les boutons d'attaque.
#[derive(Component)]
pub(crate) struct AttackButton {
    index: usize,
}

/// Marqueur pour le bouton « Fuir ».
#[derive(Component)]
pub(crate) struct FleeButton;

/// Marqueur pour le bouton « Continuer » / « Retour au menu ».
#[derive(Component)]
pub(crate) struct ContinueButton;

/// Construit l'UI de combat.
pub(crate) fn spawn_battle_ui(
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
                padding: UiRect::new(
                    Val::Px(12.0),
                    Val::Px(12.0),
                    Val::Px(SAFE_TOP),
                    Val::Px(12.0),
                ),
                ..default()
            },
            BackgroundColor(colors::BACKGROUND),
            ScreenEntity,
            bevy::state::state_scoped::StateScoped(GameScreen::Battle),
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

                match battle.phase {
                    BattlePhase::PlayerChooseAttack => {
                        // Boutons d'attaque
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
                                    AttackButton { index: i },
                                    Interaction::default(),
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new(format!(
                                            "[{}] {}  (Puissance: {})",
                                            attack.element, attack.name, attack.power,
                                        )),
                                        TextFont {
                                            font_size: fonts::BODY,
                                            ..default()
                                        },
                                        TextColor(txt_color),
                                    ));
                                });
                        }

                        // Bouton « Fuir »
                        actions
                            .spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
                                    margin: UiRect::top(Val::Px(8.0)),
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                BackgroundColor(colors::ACCENT_RED),
                                BorderRadius::all(Val::Px(6.0)),
                                FleeButton,
                                Interaction::default(),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Fuir le combat"),
                                    TextFont {
                                        font_size: fonts::BODY,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            });
                    }
                    BattlePhase::Victory => {
                        // Bouton retour après victoire
                        actions
                            .spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    padding: UiRect::axes(Val::Px(16.0), Val::Px(14.0)),
                                    margin: UiRect::top(Val::Px(8.0)),
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                BackgroundColor(colors::ACCENT_GREEN),
                                BorderRadius::all(Val::Px(8.0)),
                                ContinueButton,
                                Interaction::default(),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Victoire ! Retour au menu"),
                                    TextFont {
                                        font_size: fonts::BODY,
                                        ..default()
                                    },
                                    TextColor(Color::BLACK),
                                ));
                            });
                    }
                    BattlePhase::Defeat => {
                        // Bouton retour après défaite
                        actions
                            .spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    padding: UiRect::axes(Val::Px(16.0), Val::Px(14.0)),
                                    margin: UiRect::top(Val::Px(8.0)),
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                BackgroundColor(colors::ACCENT_RED),
                                BorderRadius::all(Val::Px(8.0)),
                                ContinueButton,
                                Interaction::default(),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Defaite... Retour au menu"),
                                    TextFont {
                                        font_size: fonts::BODY,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            });
                    }
                    _ => {
                        // Autres phases : bouton « Continuer » pour avancer les messages
                        actions
                            .spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    padding: UiRect::axes(Val::Px(16.0), Val::Px(12.0)),
                                    margin: UiRect::top(Val::Px(8.0)),
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                BackgroundColor(colors::PANEL),
                                BorderRadius::all(Val::Px(6.0)),
                                ContinueButton,
                                Interaction::default(),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Continuer..."),
                                    TextFont {
                                        font_size: fonts::BODY,
                                        ..default()
                                    },
                                    TextColor(colors::TEXT_PRIMARY),
                                ));
                            });
                    }
                }
            });
        });
}

/// Gestion des entrées en combat.
pub(crate) fn handle_battle_input(
    mut data: ResMut<GameData>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameScreen>>,
    attack_query: Query<(&Interaction, &AttackButton), Changed<Interaction>>,
    flee_query: Query<&Interaction, (Changed<Interaction>, With<FleeButton>)>,
    continue_query: Query<&Interaction, (Changed<Interaction>, With<ContinueButton>)>,
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

            // Toucher bouton attaque (mobile)
            for (interaction, btn) in &attack_query {
                if *interaction == Interaction::Pressed {
                    battle.attack_menu_index = btn.index;
                    // L'attaque sera traitée par le système de combat.
                    return;
                }
            }

            // Toucher bouton fuir (mobile)
            for interaction in &flee_query {
                if *interaction == Interaction::Pressed {
                    data.battle_state = None;
                    next_state.set(GameScreen::MainMenu);
                    return;
                }
            }

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
            // Toucher « Retour au menu » (mobile)
            for interaction in &continue_query {
                if *interaction == Interaction::Pressed {
                    data.battle_state = None;
                    next_state.set(GameScreen::MainMenu);
                    return;
                }
            }

            if keyboard.just_pressed(KeyCode::Enter) {
                data.battle_state = None;
                next_state.set(GameScreen::MainMenu);
            }
        }
        _ => {
            // Toucher « Continuer » (mobile)
            for interaction in &continue_query {
                if *interaction == Interaction::Pressed {
                    // Avancer dans les messages.
                    return;
                }
            }

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
