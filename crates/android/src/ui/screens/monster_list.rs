//! Écran de la liste des monstres vivants.

use bevy::prelude::*;
use bevy::state::state::NextState;

use crate::game::{GameData, GameScreen, ScreenEntity};
use crate::sprites;
use crate::ui::common::{colors, fonts};
use monster_battle_storage::MonsterStorage;

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
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(colors::BACKGROUND),
            ScreenEntity,
        ))
        .with_children(|parent| {
            // Titre
            parent.spawn((
                Text::new("🐾 Mes Monstres"),
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
                return;
            }

            // Liste des monstres
            for (i, monster) in monsters.iter().enumerate() {
                let selected = i == data.monster_select_index;
                let border_color = if selected {
                    colors::ACCENT_YELLOW
                } else {
                    colors::BORDER
                };

                parent
                    .spawn((
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
                    ))
                    .with_children(|card| {
                        // Sprite du monstre
                        let grid =
                            sprites::get_pixel_sprite(monster.primary_type, monster.secondary_type);
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
                            ..default()
                        })
                        .with_children(|info| {
                            let secondary = monster
                                .secondary_type
                                .map(|t| format!("/{}", t.icon()))
                                .unwrap_or_default();

                            info.spawn((
                                Text::new(format!(
                                    "{}{} {}  Nv.{}",
                                    monster.primary_type.icon(),
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
                        });
                    });
            }
        });
}

/// Gestion des entrées de la liste des monstres.
pub(crate) fn handle_monster_list_input(
    mut data: ResMut<GameData>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameScreen>>,
) {
    let monster_count = data.storage.list_alive().map(|v| v.len()).unwrap_or(0);

    if keyboard.just_pressed(KeyCode::ArrowUp) && data.monster_select_index > 0 {
        data.monster_select_index -= 1;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown)
        && monster_count > 0
        && data.monster_select_index < monster_count - 1
    {
        data.monster_select_index += 1;
    }
    if keyboard.just_pressed(KeyCode::KeyQ) || keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameScreen::MainMenu);
        data.menu_index = 0;
    }
}
