//! Écran du cimetière — liste des monstres morts.

use bevy::prelude::*;
use bevy::state::state::NextState;

use monster_battle_storage::MonsterStorage;

use crate::game::{GameData, GameScreen, ScreenEntity};
use crate::sprites;
use crate::ui::common::{colors, fonts};

/// Construit l'UI du cimetière.
pub(crate) fn spawn_cemetery(
    mut commands: Commands,
    data: Res<GameData>,
    mut images: ResMut<Assets<Image>>,
    mut atlas: ResMut<sprites::MonsterSpriteAtlas>,
) {
    let dead = data.storage.list_dead().unwrap_or_default();

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(colors::BACKGROUND),
            ScreenEntity,
        ))
        .with_children(|parent| {
            // Titre
            parent.spawn((
                Text::new(format!("💀 Cimetière ({})", dead.len())),
                TextFont {
                    font_size: fonts::HEADING,
                    ..default()
                },
                TextColor(colors::TEXT_SECONDARY),
                Node {
                    margin: UiRect::bottom(Val::Px(16.0)),
                    ..default()
                },
            ));

            if dead.is_empty() {
                parent.spawn((
                    Text::new(
                        "Le cimetière est vide.\n\
                         Vos monstres sont en sécurité... pour l'instant.",
                    ),
                    TextFont {
                        font_size: fonts::BODY,
                        ..default()
                    },
                    TextColor(colors::TEXT_SECONDARY),
                ));
            } else {
                // Conteneur scrollable
                parent
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        overflow: Overflow::clip_y(),
                        ..default()
                    })
                    .with_children(|list| {
                        for monster in &dead {
                            list.spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    padding: UiRect::all(Val::Px(10.0)),
                                    margin: UiRect::bottom(Val::Px(6.0)),
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(12.0),
                                    ..default()
                                },
                                BackgroundColor(colors::PANEL),
                                BorderRadius::all(Val::Px(8.0)),
                            ))
                            .with_children(|card| {
                                // Sprite (grisé — le monstre est mort)
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
                                        width: Val::Px(48.0),
                                        height: Val::Px(48.0),
                                        ..default()
                                    },
                                ));

                                // Infos
                                card.spawn(Node {
                                    flex_direction: FlexDirection::Column,
                                    ..default()
                                })
                                .with_children(|info| {
                                    info.spawn((
                                        Text::new(format!(
                                            "💀 {} {} — Nv.{}",
                                            monster.primary_type.icon(),
                                            monster.name,
                                            monster.level,
                                        )),
                                        TextFont {
                                            font_size: fonts::BODY,
                                            ..default()
                                        },
                                        TextColor(colors::TEXT_SECONDARY),
                                    ));

                                    info.spawn((
                                        Text::new(format!(
                                            "Vécu {}j — {} victoires",
                                            monster.age_days(),
                                            monster.wins,
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
            }

            // Footer
            parent.spawn((
                Text::new("Esc Retour"),
                TextFont {
                    font_size: fonts::SMALL,
                    ..default()
                },
                TextColor(colors::TEXT_SECONDARY),
                Node {
                    margin: UiRect::top(Val::Px(12.0)),
                    ..default()
                },
            ));
        });
}

/// Gestion des entrées sur le cimetière.
pub(crate) fn handle_cemetery_input(
    mut data: ResMut<GameData>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameScreen>>,
) {
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        data.scroll_offset = data.scroll_offset.saturating_sub(1);
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        data.scroll_offset += 1;
    }
    if keyboard.just_pressed(KeyCode::Escape) || keyboard.just_pressed(KeyCode::KeyQ) {
        next_state.set(GameScreen::MainMenu);
        data.menu_index = 0;
    }
}
