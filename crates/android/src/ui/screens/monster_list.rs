//! Écran de la liste des monstres vivants.

use bevy::prelude::*;
use bevy::state::state::NextState;

use crate::game::{GameData, GameScreen, ScreenEntity};
use crate::sprites;
use crate::ui::common::{SAFE_TOP, colors, fonts};
use monster_battle_storage::MonsterStorage;

/// Marqueur pour les cartes de monstre cliquables.
#[derive(Component)]
pub(crate) struct MonsterCardButton {
    index: usize,
}

/// Marqueur pour le bouton retour.
#[derive(Component)]
pub(crate) struct BackButton;

/// Marqueur pour le bouton « Nourrir ».
#[derive(Component)]
pub(crate) struct FeedButton;

/// Construit l'UI de la liste des monstres.
pub(crate) fn spawn_monster_list(
    mut commands: Commands,
    data: Res<GameData>,
    mut images: ResMut<Assets<Image>>,
    mut atlas: ResMut<sprites::MonsterSpriteAtlas>,
) {
    let monsters = data.storage.list_alive().unwrap_or_default();

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
                    Val::Px(12.0),
                ),
                ..default()
            },
            BackgroundColor(colors::BACKGROUND),
            ScreenEntity,
            bevy::state::state_scoped::StateScoped(GameScreen::MonsterList),
        ))
        .with_children(|parent| {
            // Titre
            parent.spawn((
                Text::new("Mes Monstres"),
                TextFont {
                    font_size: fonts::HEADING,
                    ..default()
                },
                TextColor(colors::ACCENT_YELLOW),
                Node {
                    margin: UiRect::bottom(Val::Px(16.0)),
                    ..default()
                },
            ));

            if monsters.is_empty() {
                parent.spawn((
                    Text::new("Aucun monstre vivant. Créez-en un depuis le menu !"),
                    TextFont {
                        font_size: fonts::BODY,
                        ..default()
                    },
                    TextColor(colors::TEXT_SECONDARY),
                ));
            } else {
                // Zone scrollable pour les monstres
                parent
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        overflow: Overflow::clip_y(),
                        flex_grow: 1.0,
                        ..default()
                    })
                    .with_children(|list| {
                        // Liste des monstres
                        for (i, monster) in monsters.iter().enumerate() {
                            let selected = i == data.monster_select_index;
                            let border_color = if selected {
                                colors::ACCENT_YELLOW
                            } else {
                                colors::BORDER
                            };

                            list.spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    padding: UiRect::all(Val::Px(10.0)),
                                    margin: UiRect::bottom(Val::Px(6.0)),
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(12.0),
                                    border: UiRect::all(Val::Px(2.0)),
                                    ..default()
                                },
                                BorderColor(border_color),
                                BorderRadius::all(Val::Px(8.0)),
                                BackgroundColor(colors::PANEL),
                                MonsterCardButton { index: i },
                                Interaction::default(),
                            ))
                            .with_children(|card| {
                                // Sprite du monstre
                                let grid = sprites::get_pixel_sprite(
                                    monster.primary_type,
                                    monster.secondary_type,
                                );
                                let handle = atlas.get_or_create_front(
                                    monster.primary_type,
                                    monster.secondary_type,
                                    grid,
                                    &mut images,
                                );

                                card.spawn((
                                    ImageNode::new(handle),
                                    Node {
                                        width: Val::Px(64.0),
                                        height: Val::Px(64.0),
                                        ..default()
                                    },
                                ));

                                // Infos du monstre
                                card.spawn(Node {
                                    flex_direction: FlexDirection::Column,
                                    flex_grow: 1.0,
                                    ..default()
                                })
                                .with_children(|info| {
                                    let secondary = monster
                                        .secondary_type
                                        .map(|t| format!("/{}", t))
                                        .unwrap_or_default();

                                    info.spawn((
                                        Text::new(format!(
                                            "[{}{}] {}  Nv.{}",
                                            monster.primary_type,
                                            secondary,
                                            monster.name,
                                            monster.level,
                                        )),
                                        TextFont {
                                            font_size: fonts::BODY,
                                            ..default()
                                        },
                                        TextColor(colors::TEXT_PRIMARY),
                                    ));

                                    info.spawn((
                                        Text::new(format!(
                                            "PV {}/{}  ATK {} DEF {} SPD {}",
                                            monster.current_hp,
                                            monster.max_hp(),
                                            monster.effective_attack(),
                                            monster.effective_defense(),
                                            monster.effective_speed(),
                                        )),
                                        TextFont {
                                            font_size: fonts::SMALL,
                                            ..default()
                                        },
                                        TextColor(colors::TEXT_SECONDARY),
                                    ));

                                    // Niveau de faim
                                    let hunger = monster.hunger_level();
                                    let hunger_color = match hunger {
                                        monster_battle_core::HungerLevel::Starving => {
                                            colors::ACCENT_RED
                                        }
                                        monster_battle_core::HungerLevel::Hungry => {
                                            colors::ACCENT_YELLOW
                                        }
                                        monster_battle_core::HungerLevel::Satisfied => {
                                            colors::ACCENT_GREEN
                                        }
                                        monster_battle_core::HungerLevel::Overfed => {
                                            colors::ACCENT_MAGENTA
                                        }
                                    };
                                    info.spawn((
                                        Text::new(format!("Faim: {}", hunger)),
                                        TextFont {
                                            font_size: fonts::SMALL,
                                            ..default()
                                        },
                                        TextColor(hunger_color),
                                    ));
                                });
                            });
                        }
                    });

                // Barre d'actions pour le monstre sélectionné
                parent
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        margin: UiRect::top(Val::Px(10.0)),
                        ..default()
                    })
                    .with_children(|bar| {
                        // Bouton « Nourrir »
                        bar.spawn((
                            Node {
                                flex_grow: 1.0,
                                padding: UiRect::axes(Val::Px(16.0), Val::Px(12.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(colors::ACCENT_GREEN),
                            BorderRadius::all(Val::Px(8.0)),
                            FeedButton,
                            Interaction::default(),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("Nourrir"),
                                TextFont {
                                    font_size: fonts::BODY,
                                    ..default()
                                },
                                TextColor(Color::BLACK),
                            ));
                        });
                    });
            }

            // Message éventuel
            if let Some(ref msg) = data.message {
                parent.spawn((
                    Text::new(msg.clone()),
                    TextFont {
                        font_size: fonts::SMALL,
                        ..default()
                    },
                    TextColor(colors::ACCENT_GREEN),
                    Node {
                        margin: UiRect::top(Val::Px(6.0)),
                        ..default()
                    },
                ));
            }

            // Bouton retour (tactile)
            parent
                .spawn((
                    Node {
                        padding: UiRect::axes(Val::Px(24.0), Val::Px(12.0)),
                        margin: UiRect::top(Val::Px(10.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(colors::PANEL),
                    BorderRadius::all(Val::Px(8.0)),
                    BackButton,
                    Interaction::default(),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("< Retour"),
                        TextFont {
                            font_size: fonts::BODY,
                            ..default()
                        },
                        TextColor(colors::TEXT_PRIMARY),
                    ));
                });
        });
}

/// Gestion des entrées de la liste des monstres.
pub(crate) fn handle_monster_list_input(
    mut data: ResMut<GameData>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameScreen>>,
    card_query: Query<(&Interaction, &MonsterCardButton), Changed<Interaction>>,
    back_query: Query<&Interaction, (Changed<Interaction>, With<BackButton>)>,
    feed_query: Query<&Interaction, (Changed<Interaction>, With<FeedButton>)>,
) {
    let monster_count = data.storage.list_alive().map(|v| v.len()).unwrap_or(0);

    // Toucher retour
    for interaction in &back_query {
        if *interaction == Interaction::Pressed {
            next_state.set(GameScreen::MainMenu);
            data.menu_index = 0;
            data.message = None;
            return;
        }
    }

    // Toucher nourrir
    for interaction in &feed_query {
        if *interaction == Interaction::Pressed {
            feed_selected_monster(&mut data);
            return;
        }
    }

    // Toucher carte monstre (sélection visuelle)
    for (interaction, card) in &card_query {
        if *interaction == Interaction::Pressed {
            data.monster_select_index = card.index;
            return;
        }
    }

    if keyboard.just_pressed(KeyCode::ArrowUp) && data.monster_select_index > 0 {
        data.monster_select_index -= 1;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown)
        && monster_count > 0
        && data.monster_select_index < monster_count - 1
    {
        data.monster_select_index += 1;
    }
    if keyboard.just_pressed(KeyCode::KeyF) {
        feed_selected_monster(&mut data);
    }
    if keyboard.just_pressed(KeyCode::KeyQ) || keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameScreen::MainMenu);
        data.menu_index = 0;
        data.message = None;
    }
}

/// Nourrit le monstre sélectionné.
fn feed_selected_monster(data: &mut ResMut<GameData>) {
    let mut monsters = data.storage.list_alive().unwrap_or_default();
    let idx = data.monster_select_index;
    if let Some(monster) = monsters.get_mut(idx) {
        let hunger = monster.feed();
        let name = monster.name.clone();
        let _ = data.storage.save(monster);
        data.message = Some(format!("{} a ete nourri ! ({})", name, hunger));
    } else {
        data.message = Some("Pas de monstre a nourrir.".to_string());
    }
}
