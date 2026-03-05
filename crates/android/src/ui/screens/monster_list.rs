//! Écran de la liste des monstres vivants.

use bevy::prelude::*;
use bevy::state::state::NextState;

use crate::game::{GameData, GameScreen, ScreenEntity};
use crate::sprites;
use crate::ui::common::{SAFE_BOTTOM, SAFE_TOP, ScrollableContent, colors, fonts};
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
    spawn_monster_list_inner(&mut commands, &data, &mut images, &mut atlas);
}

/// Logique interne de création de l'UI (réutilisable pour les rebuilds).
fn spawn_monster_list_inner(
    commands: &mut Commands,
    data: &GameData,
    images: &mut Assets<Image>,
    atlas: &mut sprites::MonsterSpriteAtlas,
) {
    let monsters = data.storage.list_alive().unwrap_or_default();

    // Pre-compute sprite handles to avoid mutable borrow issues in closures
    let sprite_handles: Vec<Handle<Image>> = monsters
        .iter()
        .map(|m| {
            let age = m.age_stage();
            let grid = sprites::get_blended_sprite(m.primary_type, m.secondary_type, age);
            atlas.get_or_create_front(m.primary_type, m.secondary_type, age, &grid, images)
        })
        .collect();

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
            bevy::state::state_scoped::StateScoped(GameScreen::MonsterList),
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
                        BackButton,
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
                    .with_children(|list| {
                        // Liste des monstres
                        for (i, monster) in monsters.iter().enumerate() {
                            let selected = i == data.monster_select_index;
                            let border_color = if selected {
                                colors::ACCENT_YELLOW
                            } else {
                                colors::BORDER
                            };

                            let handle = sprite_handles[i].clone();

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
                                    row_gap: Val::Px(2.0),
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

                                    // XP
                                    let xp_to_next = monster.xp_to_next_level();
                                    info.spawn((
                                        Text::new(format!("XP {}/{}", monster.xp, xp_to_next,)),
                                        TextFont {
                                            font_size: fonts::SMALL,
                                            ..default()
                                        },
                                        TextColor(colors::TEXT_SECONDARY),
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

                                    // S.ATK et S.DEF
                                    info.spawn((
                                        Text::new(format!(
                                            "S.ATK {} S.DEF {}",
                                            monster.effective_sp_attack(),
                                            monster.effective_sp_defense(),
                                        )),
                                        TextFont {
                                            font_size: fonts::SMALL,
                                            ..default()
                                        },
                                        TextColor(colors::TEXT_SECONDARY),
                                    ));

                                    // Traits
                                    if !monster.traits.is_empty() {
                                        let traits_str: Vec<String> = monster
                                            .traits
                                            .iter()
                                            .map(|t| format!("{}", t))
                                            .collect();
                                        info.spawn((
                                            Text::new(format!(
                                                "Traits : {}",
                                                traits_str.join(", ")
                                            )),
                                            TextFont {
                                                font_size: fonts::SMALL,
                                                ..default()
                                            },
                                            TextColor(colors::ACCENT_YELLOW),
                                        ));
                                    }

                                    // Stade et Âge
                                    let stage = monster.age_stage();
                                    info.spawn((
                                        Text::new(format!(
                                            "Stade : {}  Age : {}j/{}j",
                                            stage,
                                            monster.age_days(),
                                            monster.max_age_days(),
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

                                    // Generation et V/D
                                    info.spawn((
                                        Text::new(format!(
                                            "Gen. {}  V:{} / D:{}",
                                            monster.generation, monster.wins, monster.losses,
                                        )),
                                        TextFont {
                                            font_size: fonts::SMALL,
                                            ..default()
                                        },
                                        TextColor(colors::TEXT_SECONDARY),
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
        });
}

/// Gestion des entrées de la liste des monstres.
pub(crate) fn handle_monster_list_input(
    mut commands: Commands,
    mut data: ResMut<GameData>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameScreen>>,
    mut images: ResMut<Assets<Image>>,
    mut atlas: ResMut<sprites::MonsterSpriteAtlas>,
    screen_entities: Query<Entity, With<ScreenEntity>>,
    card_query: Query<(&Interaction, &MonsterCardButton), Changed<Interaction>>,
    back_query: Query<&Interaction, (Changed<Interaction>, With<BackButton>)>,
    feed_query: Query<&Interaction, (Changed<Interaction>, With<FeedButton>)>,
) {
    let monster_count = data.storage.list_alive().map(|v| v.len()).unwrap_or(0);
    let mut needs_rebuild = false;

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
            needs_rebuild = true;
            break;
        }
    }

    // Toucher carte monstre (sélection visuelle)
    if !needs_rebuild {
        for (interaction, card) in &card_query {
            if *interaction == Interaction::Pressed {
                data.monster_select_index = card.index;
                needs_rebuild = true;
                break;
            }
        }
    }

    if keyboard.just_pressed(KeyCode::ArrowUp) && data.monster_select_index > 0 {
        data.monster_select_index -= 1;
        needs_rebuild = true;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown)
        && monster_count > 0
        && data.monster_select_index < monster_count - 1
    {
        data.monster_select_index += 1;
        needs_rebuild = true;
    }
    if keyboard.just_pressed(KeyCode::KeyF) {
        feed_selected_monster(&mut data);
        needs_rebuild = true;
    }
    if keyboard.just_pressed(KeyCode::KeyQ) || keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameScreen::MainMenu);
        data.menu_index = 0;
        data.message = None;
        return;
    }

    // Reconstruire l'UI si les données ont changé
    if needs_rebuild {
        for entity in &screen_entities {
            commands.entity(entity).despawn_recursive();
        }
        spawn_monster_list_inner(&mut commands, &data, &mut images, &mut atlas);
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
