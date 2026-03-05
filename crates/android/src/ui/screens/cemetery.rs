//! Écran du cimetière — liste des monstres morts.

use bevy::prelude::*;
use bevy::state::state::NextState;

use monster_battle_storage::MonsterStorage;

use crate::game::{GameData, GameScreen, ScreenEntity};
use crate::sprites;
use crate::ui::common::{
    KEYBOARD_SCROLL_STEP, SAFE_BOTTOM, SAFE_TOP, ScrollableContent, colors, fonts,
};

/// Marqueur pour le bouton retour.
#[derive(Component)]
pub(crate) struct CemeteryBackButton;

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
            bevy::state::state_scoped::StateScoped(GameScreen::Cemetery),
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
                        CemeteryBackButton,
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
                Text::new(format!("Cimetiere ({})", dead.len())),
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
                                let age = monster.age_stage();
                                let grid = sprites::get_blended_sprite(
                                    monster.primary_type,
                                    monster.secondary_type,
                                    age,
                                );
                                let handle = atlas.get_or_create_front(
                                    monster.primary_type,
                                    monster.secondary_type,
                                    age,
                                    &grid,
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
                                            "[x] {} {} -- Nv.{}",
                                            monster.primary_type, monster.name, monster.level,
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
        });
}

/// Gestion des entrées sur le cimetière.
pub(crate) fn handle_cemetery_input(
    mut data: ResMut<GameData>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameScreen>>,
    back_query: Query<(&Interaction, &CemeteryBackButton), Changed<Interaction>>,
    mut scroll_query: Query<&mut ScrollPosition, With<ScrollableContent>>,
) {
    for (interaction, _) in &back_query {
        if *interaction == Interaction::Pressed {
            next_state.set(GameScreen::MainMenu);
            data.menu_index = 0;
            return;
        }
    }

    if keyboard.just_pressed(KeyCode::ArrowUp) {
        for mut scroll_pos in &mut scroll_query {
            scroll_pos.offset_y = (scroll_pos.offset_y - KEYBOARD_SCROLL_STEP).max(0.0);
        }
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        for mut scroll_pos in &mut scroll_query {
            scroll_pos.offset_y += KEYBOARD_SCROLL_STEP;
        }
    }
    if keyboard.just_pressed(KeyCode::Escape) || keyboard.just_pressed(KeyCode::KeyQ) {
        next_state.set(GameScreen::MainMenu);
        data.menu_index = 0;
    }
}
