//! Écran de sélection des 4 attaques actives d'un monstre.

use bevy::prelude::*;
use bevy::state::state::NextState;

use monster_battle_core::attack::Attack;
use monster_battle_storage::MonsterStorage;

use crate::game::{GameData, GameScreen, ScreenEntity};
use crate::ui::common::{SAFE_BOTTOM_DEFAULT, SAFE_TOP_DEFAULT, ScrollableContent, colors, fonts};

/// Marqueur pour un bouton d'attaque (toggle sélection).
#[derive(Component)]
pub(crate) struct AttackToggleButton {
    pub index: usize,
}

/// Marqueur pour le bouton "Confirmer".
#[derive(Component)]
pub(crate) struct ConfirmAttacksButton;

/// Marqueur pour le bouton "Retour".
#[derive(Component)]
pub(crate) struct SelectAttacksBackButton;

/// Ressource contenant l'état de sélection en cours.
#[derive(Resource, Default)]
pub(crate) struct AttackSelectionState {
    pub selected: Vec<usize>,
    /// Vrai quand l'UI doit être reconstruite suite à un toggle.
    pub dirty: bool,
}

/// Construit l'UI de sélection des attaques actives.
pub(crate) fn spawn_select_attacks(
    mut commands: Commands,
    data: Res<GameData>,
    selection: Res<AttackSelectionState>,
) {
    spawn_select_attacks_ui(&mut commands, &data, &selection);
}

/// Logique interne réutilisable pour construire l'UI.
fn spawn_select_attacks_ui(
    commands: &mut Commands,
    data: &GameData,
    selection: &AttackSelectionState,
) {
    let monsters = data.storage.list_alive().unwrap_or_default();
    let idx = data
        .monster_select_index
        .min(monsters.len().saturating_sub(1));
    let monster_opt = monsters.get(idx);

    let known_attacks: Vec<Attack> = monster_opt.map(|m| m.known_attacks()).unwrap_or_default();

    let monster_name = monster_opt
        .map(|m| m.name.clone())
        .unwrap_or_else(|| "Monstre".to_string());

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::new(
                    Val::Px(12.0),
                    Val::Px(12.0),
                    Val::Px(SAFE_TOP_DEFAULT),
                    Val::Px(SAFE_BOTTOM_DEFAULT),
                ),
                ..default()
            },
            BackgroundColor(colors::BACKGROUND),
            ScreenEntity,
            bevy::state::state_scoped::StateScoped(GameScreen::SelectAttacks),
        ))
        .with_children(|parent| {
            // Barre de navigation
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
                        SelectAttacksBackButton,
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
                Text::new(format!("Attaques actives - {}", monster_name)),
                TextFont {
                    font_size: fonts::HEADING,
                    ..default()
                },
                TextColor(colors::ACCENT_YELLOW),
                Node {
                    margin: UiRect::bottom(Val::Px(4.0)),
                    ..default()
                },
            ));

            // Sous-titre
            parent.spawn((
                Text::new(format!(
                    "Selectionnez jusqu'a 4 attaques ({}/4 choisies)",
                    selection.selected.len()
                )),
                TextFont {
                    font_size: fonts::SMALL,
                    ..default()
                },
                TextColor(colors::TEXT_SECONDARY),
                Node {
                    margin: UiRect::bottom(Val::Px(12.0)),
                    ..default()
                },
            ));

            if known_attacks.is_empty() {
                parent.spawn((
                    Text::new("Aucune attaque disponible."),
                    TextFont {
                        font_size: fonts::BODY,
                        ..default()
                    },
                    TextColor(colors::TEXT_SECONDARY),
                ));
                return;
            }

            // Liste des attaques scrollable
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
                    for (i, attack) in known_attacks.iter().enumerate() {
                        let is_selected = selection.selected.contains(&i);
                        let border_color = if is_selected {
                            colors::ACCENT_YELLOW
                        } else {
                            colors::BORDER
                        };
                        let bg_color: Color = if is_selected {
                            Color::srgba(0.3, 0.25, 0.0, 0.4)
                        } else {
                            colors::PANEL
                        };
                        let special_label = if attack.is_special { "Spe." } else { "Phys." };

                        scroll
                            .spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    padding: UiRect::all(Val::Px(10.0)),
                                    margin: UiRect::bottom(Val::Px(6.0)),
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::SpaceBetween,
                                    border: UiRect::all(Val::Px(2.0)),
                                    ..default()
                                },
                                BorderColor(border_color),
                                BorderRadius::all(Val::Px(8.0)),
                                BackgroundColor(bg_color),
                                AttackToggleButton { index: i },
                                Interaction::default(),
                            ))
                            .with_children(|card| {
                                card.spawn(Node {
                                    flex_direction: FlexDirection::Column,
                                    ..default()
                                })
                                .with_children(|info| {
                                    info.spawn((
                                        Text::new(format!(
                                            "{} {}  [{}]",
                                            attack.element.icon(),
                                            attack.name,
                                            attack.element,
                                        )),
                                        TextFont {
                                            font_size: fonts::BODY,
                                            ..default()
                                        },
                                        TextColor(colors::TEXT_PRIMARY),
                                    ));
                                    info.spawn((
                                        Text::new(format!(
                                            "{} | Force: {} | Precision: {}%",
                                            special_label, attack.power, attack.accuracy
                                        )),
                                        TextFont {
                                            font_size: fonts::SMALL,
                                            ..default()
                                        },
                                        TextColor(colors::TEXT_SECONDARY),
                                    ));
                                });

                                if is_selected {
                                    card.spawn((
                                        Text::new("OK"),
                                        TextFont {
                                            font_size: fonts::BODY,
                                            ..default()
                                        },
                                        TextColor(colors::ACCENT_YELLOW),
                                    ));
                                }
                            });
                    }
                });

            // Bouton Confirmer
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        padding: UiRect::vertical(Val::Px(12.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Px(8.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BorderColor(colors::ACCENT_GREEN),
                    BorderRadius::all(Val::Px(8.0)),
                    BackgroundColor(Color::srgba(0.0, 0.4, 0.1, 0.5)),
                    ConfirmAttacksButton,
                    Interaction::default(),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Confirmer la selection"),
                        TextFont {
                            font_size: fonts::BODY,
                            ..default()
                        },
                        TextColor(colors::TEXT_PRIMARY),
                    ));
                });
        });
}

/// Reconstruit l'UI quand le flag dirty est positionne.
pub(crate) fn refresh_select_attacks_ui(
    mut commands: Commands,
    data: Res<GameData>,
    mut selection: ResMut<AttackSelectionState>,
    screen_entities: Query<Entity, With<ScreenEntity>>,
) {
    if !selection.dirty {
        return;
    }
    selection.dirty = false;

    for entity in &screen_entities {
        commands.entity(entity).despawn_recursive();
    }
    spawn_select_attacks_ui(&mut commands, &data, &selection);
}

/// Gestion des interactions de l'écran de sélection d'attaques.
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_select_attacks_input(
    mut commands: Commands,
    mut data: ResMut<GameData>,
    mut selection: ResMut<AttackSelectionState>,
    mut next_state: ResMut<NextState<GameScreen>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    toggle_query: Query<(&Interaction, &AttackToggleButton), Changed<Interaction>>,
    confirm_query: Query<&Interaction, (Changed<Interaction>, With<ConfirmAttacksButton>)>,
    back_query: Query<&Interaction, (Changed<Interaction>, With<SelectAttacksBackButton>)>,
) {
    // Retour
    let mut go_back = keyboard.just_pressed(KeyCode::Escape);
    for interaction in &back_query {
        if *interaction == Interaction::Pressed {
            go_back = true;
            break;
        }
    }
    if go_back {
        commands.remove_resource::<AttackSelectionState>();
        next_state.set(GameScreen::MonsterList);
        return;
    }

    // Toggle attaque
    let mut toggled: Option<usize> = None;
    for (interaction, btn) in &toggle_query {
        if *interaction == Interaction::Pressed {
            toggled = Some(btn.index);
            break;
        }
    }
    if let Some(idx) = toggled {
        if selection.selected.contains(&idx) {
            selection.selected.retain(|&x| x != idx);
        } else if selection.selected.len() < 4 {
            selection.selected.push(idx);
        }
        selection.dirty = true;
        return;
    }

    // Confirmer
    let mut confirmed = keyboard.just_pressed(KeyCode::Enter);
    for interaction in &confirm_query {
        if *interaction == Interaction::Pressed {
            confirmed = true;
            break;
        }
    }
    if confirmed {
        if let Ok(mut monsters) = data.storage.list_alive() {
            let idx = data
                .monster_select_index
                .min(monsters.len().saturating_sub(1));
            if let Some(monster) = monsters.get_mut(idx) {
                let selected = selection.selected.clone();
                match monster.set_active_attacks(selected) {
                    Ok(()) => {
                        let _ = data.storage.save(monster);
                        data.message = Some("Attaques actives mises a jour !".to_string());
                    }
                    Err(e) => {
                        data.message = Some(format!("Erreur : {}", e));
                    }
                }
            }
        }
        commands.remove_resource::<AttackSelectionState>();
        next_state.set(GameScreen::MonsterList);
    }
}
