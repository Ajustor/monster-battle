//! Écran d'entraînement — choix du mode (docile/sauvage) et du type adverse.

use bevy::prelude::*;
use bevy::state::state::NextState;

use monster_battle_core::Monster;
use monster_battle_core::battle::BattleState;
use monster_battle_core::genetics::generate_starter_stats;
use monster_battle_core::types::ElementType;
use monster_battle_storage::MonsterStorage;

use crate::game::{GameData, GameScreen, ScreenEntity};
use crate::ui::common::{SAFE_BOTTOM, SAFE_TOP, ScrollableContent, colors, fonts};

/// Ressource indiquant si le mode sauvage est actif.
#[derive(Resource)]
pub struct TrainingWild(pub bool);

/// Marqueur pour les boutons de type adverse.
#[derive(Component)]
pub(crate) struct BotTypeButton {
    index: usize,
}

/// Marqueur pour le bouton toggle mode.
#[derive(Component)]
pub(crate) struct ModeToggleButton;

/// Marqueur pour le bouton retour.
#[derive(Component)]
pub(crate) struct TrainingBackButton;

/// Construit l'UI d'entraînement.
pub(crate) fn spawn_training(mut commands: Commands, data: Res<GameData>, wild: Res<TrainingWild>) {
    spawn_training_inner(&mut commands, &data, wild.0);
}

/// Logique interne de création de l'UI d'entraînement (réutilisable).
fn spawn_training_inner(commands: &mut Commands, data: &GameData, wild: bool) {
    let monsters = data.storage.list_alive().unwrap_or_default();
    let selected_idx = data.monster_select_index;
    let monster = monsters.get(selected_idx).or(monsters.first());

    let types = ElementType::all();

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
            bevy::state::state_scoped::StateScoped(GameScreen::Training),
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
                        TrainingBackButton,
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

            // Info monstre du joueur
            if let Some(m) = monster {
                parent
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            padding: UiRect::all(Val::Px(10.0)),
                            margin: UiRect::bottom(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(colors::PANEL),
                        BorderRadius::all(Val::Px(8.0)),
                    ))
                    .with_children(|p| {
                        p.spawn((
                            Text::new(format!(
                                "Votre monstre : {} -- Nv.{} -- PV {}/{}",
                                m.name,
                                m.level,
                                m.current_hp,
                                m.max_hp()
                            )),
                            TextFont {
                                font_size: fonts::BODY,
                                ..default()
                            },
                            TextColor(colors::ACCENT_GREEN),
                        ));
                    });
            }

            // Mode d'entraînement (toggle)
            let (mode_label, mode_color, mode_desc) = if wild {
                (
                    "SAUVAGE",
                    colors::ACCENT_RED,
                    "100% XP -- /!\\ Defaite = mort du monstre",
                )
            } else {
                (
                    "DOCILE",
                    colors::ACCENT_GREEN,
                    "50% XP -- Pas de risque de mort",
                )
            };

            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        padding: UiRect::all(Val::Px(12.0)),
                        margin: UiRect::bottom(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0),
                        ..default()
                    },
                    BorderColor(mode_color),
                    BorderRadius::all(Val::Px(8.0)),
                    BackgroundColor(colors::PANEL),
                    ModeToggleButton,
                    Interaction::default(),
                ))
                .with_children(|block| {
                    block.spawn((
                        Text::new(format!("Mode : < {} >", mode_label)),
                        TextFont {
                            font_size: fonts::BODY,
                            ..default()
                        },
                        TextColor(mode_color),
                    ));
                    block.spawn((
                        Text::new(mode_desc),
                        TextFont {
                            font_size: fonts::SMALL,
                            ..default()
                        },
                        TextColor(colors::TEXT_SECONDARY),
                    ));
                    block.spawn((
                        Text::new("← → Changer de mode  |  Toucher pour basculer"),
                        TextFont {
                            font_size: fonts::SMALL,
                            ..default()
                        },
                        TextColor(colors::TEXT_SECONDARY),
                    ));
                });

            // Titre liste des adversaires
            let list_title = if wild {
                "Choisir un adversaire (Sauvage — 100% XP)"
            } else {
                "Choisir un adversaire (Docile — 50% XP)"
            };
            parent.spawn((
                Text::new(list_title),
                TextFont {
                    font_size: fonts::SMALL,
                    ..default()
                },
                TextColor(colors::ACCENT_RED),
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
            ));

            // Liste des types d'adversaires (scrollable)
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
                    for (i, t) in types.iter().enumerate() {
                        let selected = i == data.menu_index % types.len();
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

                        // Indicateur d'efficacité
                        let indicator = if let Some(m) = monster {
                            let eff = m.primary_type.effectiveness_against(t);
                            if eff > 1.0 {
                                " [+] avantage"
                            } else if eff < 1.0 {
                                " [-] desavantage"
                            } else {
                                ""
                            }
                        } else {
                            ""
                        };

                        scroll
                            .spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
                                    margin: UiRect::bottom(Val::Px(4.0)),
                                    ..default()
                                },
                                BackgroundColor(bg),
                                BorderRadius::all(Val::Px(6.0)),
                                BotTypeButton { index: i },
                                Interaction::default(),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new(format!("Bot {}{}", t, indicator)),
                                    TextFont {
                                        font_size: fonts::BODY,
                                        ..default()
                                    },
                                    TextColor(txt_color),
                                ));
                            });
                    }
                });
        });
}

/// Gestion des entrées sur l'écran d'entraînement.
pub(crate) fn handle_training_input(
    mut commands: Commands,
    mut data: ResMut<GameData>,
    mut wild: ResMut<TrainingWild>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameScreen>>,
    type_buttons: Query<(&Interaction, &BotTypeButton), Changed<Interaction>>,
    mode_buttons: Query<
        (&Interaction, &ModeToggleButton),
        (Changed<Interaction>, Without<BotTypeButton>),
    >,
    back_query: Query<&Interaction, (Changed<Interaction>, With<TrainingBackButton>)>,
    screen_entities: Query<Entity, With<ScreenEntity>>,
) {
    let types = ElementType::all();
    let type_count = types.len();

    let mut needs_rebuild = false;

    // Toucher retour
    for interaction in &back_query {
        if *interaction == Interaction::Pressed {
            next_state.set(GameScreen::MainMenu);
            data.menu_index = 0;
            return;
        }
    }

    // Toucher toggle mode
    for (interaction, _) in &mode_buttons {
        if *interaction == Interaction::Pressed {
            wild.0 = !wild.0;
            needs_rebuild = true;
            break;
        }
    }

    // Toucher type adverse
    if !needs_rebuild {
        for (interaction, button) in &type_buttons {
            if *interaction == Interaction::Pressed {
                data.menu_index = button.index;
                start_training_fight(&mut data, &mut next_state, wild.0);
                return;
            }
        }
    }

    // Clavier : toggle mode
    if keyboard.just_pressed(KeyCode::ArrowLeft) || keyboard.just_pressed(KeyCode::ArrowRight) {
        wild.0 = !wild.0;
        needs_rebuild = true;
    }

    // Clavier : navigation type
    if keyboard.just_pressed(KeyCode::ArrowUp) && data.menu_index > 0 {
        data.menu_index -= 1;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) && data.menu_index < type_count - 1 {
        data.menu_index += 1;
    }

    // Lancer le combat
    if keyboard.just_pressed(KeyCode::Enter) {
        start_training_fight(&mut data, &mut next_state, wild.0);
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameScreen::MainMenu);
        data.menu_index = 0;
    }

    // Reconstruire l'UI si le mode a changé
    if needs_rebuild {
        for entity in &screen_entities {
            commands.entity(entity).despawn_recursive();
        }
        spawn_training_inner(&mut commands, &data, wild.0);
    }
}

/// Crée un combat d'entraînement et transite vers Battle.
fn start_training_fight(
    data: &mut ResMut<GameData>,
    next_state: &mut ResMut<NextState<GameScreen>>,
    wild: bool,
) {
    let monsters = match data.storage.list_alive() {
        Ok(m) if !m.is_empty() => m,
        _ => {
            data.message = Some("Pas de monstre vivant !".to_string());
            return;
        }
    };

    let selected_idx = data.monster_select_index;
    let player_monster = monsters.get(selected_idx).unwrap_or(&monsters[0]);

    let types = ElementType::all();
    let bot_type = types[data.menu_index % types.len()];

    // Créer un bot du même niveau que le joueur (±2)
    let player_level = player_monster.level;
    let bot_level = player_level.saturating_sub(2).max(1);
    let mut bot_stats = generate_starter_stats(bot_type);
    bot_stats.hp += bot_level * 2;

    let mut bot = Monster::new_starter(format!("Bot {}", bot_type), bot_type, bot_stats);
    if bot_level > 1 {
        bot.gain_xp(bot_level * bot_level * 10);
    }

    // is_training = true signifie docile (pas de mort) — wild = !is_training
    let battle = BattleState::new(player_monster, &bot, !wild);
    data.battle_state = Some(battle);
    next_state.set(GameScreen::Battle);
}
