//! Écran de sélection du monstre (entraînement / PvP / reproduction).

use bevy::prelude::*;
use bevy::state::state::NextState;

use monster_battle_storage::MonsterStorage;

use crate::game::{GameData, GameScreen, ScreenEntity, SelectMonsterTarget};
use crate::sprites;
use crate::ui::common::{colors, fonts};

/// Marqueur pour les cartes de monstre cliquables.
#[derive(Component)]
pub(crate) struct MonsterCard {
    index: usize,
}

/// Construit l'UI de sélection du monstre.
pub(crate) fn spawn_select_monster(
    mut commands: Commands,
    data: Res<GameData>,
    target: Option<Res<SelectMonsterTarget>>,
    mut images: ResMut<Assets<Image>>,
    mut atlas: ResMut<sprites::MonsterSpriteAtlas>,
) {
    let monsters = data.storage.list_alive().unwrap_or_default();

    let title = match target.as_deref() {
        Some(SelectMonsterTarget::Training) => "⚔️  Choisir un monstre — Entraînement",
        Some(SelectMonsterTarget::CombatPvP) => "🗡️  Choisir un monstre — Combat PvP",
        Some(SelectMonsterTarget::Breeding) => "🧬 Choisir un monstre — Reproduction",
        None => "Choisir un monstre",
    };

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
                Text::new(title),
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
                    Text::new("Aucun monstre vivant !"),
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
                        MonsterCard { index: i },
                        Interaction::default(),
                    ))
                    .with_children(|card| {
                        // Sprite
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

                        // Infos
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

            // Footer
            parent.spawn((
                Text::new("↑↓ Naviguer  ⏎ Sélectionner  Esc Retour"),
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

/// Gestion des entrées de sélection du monstre.
pub(crate) fn handle_select_monster_input(
    mut data: ResMut<GameData>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameScreen>>,
    target: Option<Res<SelectMonsterTarget>>,
    interaction_query: Query<(&Interaction, &MonsterCard), Changed<Interaction>>,
) {
    let monster_count = data.storage.list_alive().map(|v| v.len()).unwrap_or(0);

    // Toucher (mobile)
    for (interaction, card) in &interaction_query {
        if *interaction == Interaction::Pressed {
            data.monster_select_index = card.index;
            dispatch_selection(&mut data, &mut next_state, target.as_deref());
            return;
        }
    }

    // Clavier
    if keyboard.just_pressed(KeyCode::ArrowUp) && data.monster_select_index > 0 {
        data.monster_select_index -= 1;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown)
        && monster_count > 0
        && data.monster_select_index < monster_count - 1
    {
        data.monster_select_index += 1;
    }
    if keyboard.just_pressed(KeyCode::Enter) && monster_count > 0 {
        dispatch_selection(&mut data, &mut next_state, target.as_deref());
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameScreen::MainMenu);
        data.menu_index = 0;
    }
}

/// Redirige vers l'écran approprié après sélection du monstre.
fn dispatch_selection(
    data: &mut ResMut<GameData>,
    next_state: &mut ResMut<NextState<GameScreen>>,
    target: Option<&SelectMonsterTarget>,
) {
    match target {
        Some(SelectMonsterTarget::Training) => {
            data.menu_index = 0;
            next_state.set(GameScreen::Training);
        }
        Some(SelectMonsterTarget::CombatPvP) => {
            next_state.set(GameScreen::PvpSearching);
        }
        Some(SelectMonsterTarget::Breeding) => {
            next_state.set(GameScreen::BreedingSearching);
        }
        None => {
            // Fallback → entraînement
            data.menu_index = 0;
            next_state.set(GameScreen::Training);
        }
    }
}
