//! Écran de gestion détaillée d'un monstre.
//!
//! Accessible depuis MonsterList en tapant sur une carte monstre.
//! Affiche stats, types, traits et permet de gérer les attaques,
//! renommer, relâcher ou dévorer un autre monstre.

use bevy::prelude::*;
use bevy::state::state::NextState;

use monster_battle_storage::MonsterStorage;

use crate::game::{
    DevourTargetIndex, GameData, GameScreen, ScreenEntity, SelectedMonsterIndex,
};
use crate::ui::common::{ScrollableContent, colors, fonts, ScreenMetrics};

// ── Composants marqueurs ─────────────────────────────────────────

#[derive(Component)]
pub(crate) struct ManageAttacksButton;

#[derive(Component)]
pub(crate) struct RenameButton;

#[derive(Component)]
pub(crate) struct ReleaseButton;

#[derive(Component)]
pub(crate) struct DevourButton;

#[derive(Component)]
pub(crate) struct DetailBackButton;

/// Popup de confirmation de relâche.
#[derive(Component)]
pub(crate) struct ReleaseConfirmButton;

#[derive(Component)]
pub(crate) struct ReleaseCancelButton;

/// Ressource locale : indique si la popup de confirmation relâche est ouverte.
#[derive(Resource, Default)]
pub(crate) struct ReleaseConfirmOpen(pub bool);

// ── Spawn principal ──────────────────────────────────────────────

/// Point d'entrée : reconstruit l'UI depuis les ressources Bevy.
pub(crate) fn spawn_monster_detail(
    mut commands: Commands,
    data: Res<GameData>,
    selected: Res<SelectedMonsterIndex>,
    confirm: Res<ReleaseConfirmOpen>,
    metrics: Res<ScreenMetrics>,
) {
    spawn_monster_detail_inner(
        &mut commands,
        &data,
        selected.0,
        confirm.0,
        metrics.safe_top,
        metrics.safe_bottom,
    );
}

fn spawn_monster_detail_inner(
    commands: &mut Commands,
    data: &GameData,
    idx: usize,
    confirm_open: bool,
    safe_top: f32,
    safe_bottom: f32,
) {
    let monsters = data.storage.list_alive().unwrap_or_default();
    let idx = idx.min(monsters.len().saturating_sub(1));
    let monster_opt = monsters.get(idx);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::new(
                    Val::Px(12.0),
                    Val::Px(12.0),
                    Val::Px(safe_top),
                    Val::Px(safe_bottom),
                ),
                ..default()
            },
            BackgroundColor(colors::BACKGROUND),
            ScreenEntity,
            bevy::state::state_scoped::StateScoped(GameScreen::MonsterDetail),
        ))
        .with_children(|parent| {
            // ── Barre de navigation (Retour) ──────────────────────
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
                        DetailBackButton,
                        Interaction::default(),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("< Retour"),
                            TextFont { font_size: fonts::SMALL, ..default() },
                            TextColor(colors::TEXT_PRIMARY),
                        ));
                    });
                });

            if let Some(monster) = monster_opt {
                // ── Zone scrollable principale ────────────────────
                parent
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            overflow: Overflow::scroll_y(),
                            flex_grow: 1.0,
                            row_gap: Val::Px(12.0),
                            ..default()
                        },
                        ScrollPosition::default(),
                        ScrollableContent,
                    ))
                    .with_children(|scroll| {
                        // ── 1. Header ─────────────────────────────
                        scroll
                            .spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    padding: UiRect::all(Val::Px(12.0)),
                                    row_gap: Val::Px(6.0),
                                    ..default()
                                },
                                BackgroundColor(colors::PANEL),
                                BorderRadius::all(Val::Px(8.0)),
                            ))
                            .with_children(|header| {
                                // Emoji type + nom
                                let type_emoji = type_emoji(monster.primary_type);
                                let secondary = monster
                                    .secondary_type
                                    .map(|t| format!("/{}", t))
                                    .unwrap_or_default();
                                header.spawn((
                                    Text::new(format!(
                                        "{} {} {}",
                                        type_emoji,
                                        monster.name,
                                        secondary
                                    )),
                                    TextFont { font_size: fonts::HEADING, ..default() },
                                    TextColor(colors::ACCENT_YELLOW),
                                ));

                                // Niveau
                                header.spawn((
                                    Text::new(format!("Niveau {}", monster.level)),
                                    TextFont { font_size: fonts::BODY, ..default() },
                                    TextColor(colors::TEXT_PRIMARY),
                                ));

                                // Barre XP
                                let xp_to_next = monster.xp_to_next_level();
                                let xp_ratio =
                                    (monster.xp as f32 / xp_to_next.max(1) as f32).clamp(0.0, 1.0);
                                header.spawn((
                                    Text::new(format!("XP {}/{}", monster.xp, xp_to_next)),
                                    TextFont { font_size: fonts::SMALL, ..default() },
                                    TextColor(colors::TEXT_SECONDARY),
                                ));
                                spawn_progress_bar(header, xp_ratio, colors::ACCENT_YELLOW);
                            });

                        // ── 2. Stats ──────────────────────────────
                        scroll
                            .spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Column,
                                    padding: UiRect::all(Val::Px(10.0)),
                                    row_gap: Val::Px(6.0),
                                    ..default()
                                },
                                BackgroundColor(colors::PANEL),
                                BorderRadius::all(Val::Px(8.0)),
                            ))
                            .with_children(|stats_panel| {
                                stats_panel.spawn((
                                    Text::new("STATS"),
                                    TextFont { font_size: fonts::SMALL, ..default() },
                                    TextColor(colors::TEXT_SECONDARY),
                                ));

                                let max_hp = monster.max_hp();
                                spawn_stat_row(
                                    stats_panel,
                                    "HP",
                                    monster.current_hp,
                                    max_hp,
                                    colors::ACCENT_GREEN,
                                );

                                let eff_atk = monster.effective_attack();
                                spawn_stat_row(
                                    stats_panel,
                                    "ATQ",
                                    eff_atk,
                                    255,
                                    colors::ACCENT_RED,
                                );

                                let eff_def = monster.effective_defense();
                                spawn_stat_row(
                                    stats_panel,
                                    "DEF",
                                    eff_def,
                                    255,
                                    colors::ACCENT_BLUE,
                                );

                                let eff_vit = monster.effective_speed();
                                spawn_stat_row(
                                    stats_panel,
                                    "VIT",
                                    eff_vit,
                                    255,
                                    colors::ACCENT_YELLOW,
                                );

                                let eff_sp_atk = monster.effective_sp_attack();
                                spawn_stat_row(
                                    stats_panel,
                                    "ATQ.SP",
                                    eff_sp_atk,
                                    255,
                                    colors::ACCENT_MAGENTA,
                                );

                                let eff_sp_def = monster.effective_sp_defense();
                                spawn_stat_row(
                                    stats_panel,
                                    "DEF.SP",
                                    eff_sp_def,
                                    255,
                                    Color::srgb(0.4, 0.8, 0.8),
                                );
                            });

                        // ── 3. Types ──────────────────────────────
                        scroll
                            .spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Row,
                                    padding: UiRect::all(Val::Px(10.0)),
                                    column_gap: Val::Px(8.0),
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(colors::PANEL),
                                BorderRadius::all(Val::Px(8.0)),
                            ))
                            .with_children(|type_row| {
                                type_row.spawn((
                                    Text::new("Types :"),
                                    TextFont { font_size: fonts::SMALL, ..default() },
                                    TextColor(colors::TEXT_SECONDARY),
                                ));

                                // Badge type principal
                                spawn_type_badge(type_row, monster.primary_type);

                                // Badge type secondaire
                                if let Some(sec) = monster.secondary_type {
                                    spawn_type_badge(type_row, sec);
                                }
                            });

                        // ── 4. Traits ─────────────────────────────
                        if !monster.traits.is_empty() {
                            scroll
                                .spawn((
                                    Node {
                                        width: Val::Percent(100.0),
                                        flex_direction: FlexDirection::Column,
                                        padding: UiRect::all(Val::Px(10.0)),
                                        row_gap: Val::Px(4.0),
                                        ..default()
                                    },
                                    BackgroundColor(colors::PANEL),
                                    BorderRadius::all(Val::Px(8.0)),
                                ))
                                .with_children(|traits_panel| {
                                    traits_panel.spawn((
                                        Text::new("TRAITS"),
                                        TextFont { font_size: fonts::SMALL, ..default() },
                                        TextColor(colors::TEXT_SECONDARY),
                                    ));
                                    for t in &monster.traits {
                                        traits_panel.spawn((
                                            Text::new(format!("• {}", t)),
                                            TextFont { font_size: fonts::BODY, ..default() },
                                            TextColor(colors::ACCENT_YELLOW),
                                        ));
                                    }
                                });
                        }

                        // ── 5. Boutons d'action ───────────────────
                        scroll
                            .spawn(Node {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(8.0),
                                ..default()
                            })
                            .with_children(|btns| {
                                spawn_action_button(btns, "⚔️  Gérer les attaques", colors::ACCENT_BLUE, ManageAttacksButton);
                                spawn_action_button(btns, "✏️  Renommer", colors::PANEL, RenameButton);
                                spawn_action_button(btns, "🐲  Dévorer un monstre", Color::srgb(0.55, 0.2, 0.05), DevourButton);
                                spawn_action_button(btns, "🕊️  Relâcher", colors::ACCENT_RED, ReleaseButton);
                            });
                    });
            } else {
                parent.spawn((
                    Text::new("Aucun monstre sélectionné."),
                    TextFont { font_size: fonts::BODY, ..default() },
                    TextColor(colors::TEXT_SECONDARY),
                ));
            }

            // ── Popup confirmation relâche ────────────────────────
            if confirm_open {
                if let Some(monster) = monster_opt {
                    spawn_release_confirm_popup(parent, &monster.name);
                }
            }
        });
}

// ── Helpers UI ───────────────────────────────────────────────────

fn spawn_progress_bar(parent: &mut ChildBuilder, ratio: f32, fill_color: Color) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            BorderRadius::all(Val::Px(4.0)),
        ))
        .with_children(|bar| {
            bar.spawn((
                Node {
                    width: Val::Percent(ratio * 100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(fill_color),
                BorderRadius::all(Val::Px(4.0)),
            ));
        });
}

fn spawn_stat_row(
    parent: &mut ChildBuilder,
    label: &str,
    value: u32,
    max: u32,
    color: Color,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(format!("{} : {}", label, value)),
                TextFont { font_size: fonts::SMALL, ..default() },
                TextColor(colors::TEXT_PRIMARY),
            ));
            let ratio = (value as f32 / max.max(1) as f32).clamp(0.0, 1.0);
            spawn_progress_bar(row, ratio, color);
        });
}

fn spawn_type_badge(parent: &mut ChildBuilder, element: monster_battle_core::ElementType) {
    let (bg, emoji) = type_badge_style(element);
    parent
        .spawn((
            Node {
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(bg),
            BorderRadius::all(Val::Px(6.0)),
        ))
        .with_children(|badge| {
            badge.spawn((
                Text::new(format!("{} {}", emoji, element)),
                TextFont { font_size: fonts::SMALL, ..default() },
                TextColor(Color::WHITE),
            ));
        });
}

fn spawn_action_button<M: Component>(
    parent: &mut ChildBuilder,
    label: &str,
    bg: Color,
    marker: M,
) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::axes(Val::Px(16.0), Val::Px(12.0)),
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(bg),
            BorderRadius::all(Val::Px(8.0)),
            marker,
            Interaction::default(),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont { font_size: fonts::BODY, ..default() },
                TextColor(colors::TEXT_PRIMARY),
            ));
        });
}

fn spawn_release_confirm_popup(parent: &mut ChildBuilder, monster_name: &str) {
    // Overlay semi-transparent
    parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(20.0)),
                        row_gap: Val::Px(12.0),
                        max_width: Val::Px(300.0),
                        ..default()
                    },
                    BackgroundColor(colors::PANEL),
                    BorderRadius::all(Val::Px(12.0)),
                ))
                .with_children(|popup| {
                    popup.spawn((
                        Text::new(format!(
                            "Relâcher {} ?\nCette action est irréversible.",
                            monster_name
                        )),
                        TextFont { font_size: fonts::BODY, ..default() },
                        TextColor(colors::TEXT_PRIMARY),
                    ));

                    // Boutons côte à côte
                    popup
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(8.0),
                            width: Val::Percent(100.0),
                            ..default()
                        })
                        .with_children(|row| {
                            // Confirmer
                            row.spawn((
                                Node {
                                    flex_grow: 1.0,
                                    padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                BackgroundColor(colors::ACCENT_RED),
                                BorderRadius::all(Val::Px(6.0)),
                                ReleaseConfirmButton,
                                Interaction::default(),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Relâcher"),
                                    TextFont { font_size: fonts::SMALL, ..default() },
                                    TextColor(Color::WHITE),
                                ));
                            });

                            // Annuler
                            row.spawn((
                                Node {
                                    flex_grow: 1.0,
                                    padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                BackgroundColor(colors::PANEL),
                                BorderRadius::all(Val::Px(6.0)),
                                ReleaseCancelButton,
                                Interaction::default(),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Annuler"),
                                    TextFont { font_size: fonts::SMALL, ..default() },
                                    TextColor(colors::TEXT_PRIMARY),
                                ));
                            });
                        });
                });
        });
}

// ── Helpers de style par type élémentaire ────────────────────────

fn type_emoji(t: monster_battle_core::ElementType) -> &'static str {
    use monster_battle_core::ElementType::*;
    match t {
        Fire => "🔥",
        Water => "💧",
        Plant => "🌿",
        Electric => "⚡",
        Earth => "🪨",
        Wind => "🌪️",
        Shadow => "🌑",
        Light => "✨",
        Normal => "⬜",
    }
}

fn type_badge_style(t: monster_battle_core::ElementType) -> (Color, &'static str) {
    use monster_battle_core::ElementType::*;
    let (r, g, b) = match t {
        Fire => (0.9, 0.3, 0.1),
        Water => (0.1, 0.4, 0.9),
        Plant => (0.2, 0.7, 0.2),
        Electric => (0.9, 0.8, 0.1),
        Earth => (0.6, 0.4, 0.2),
        Wind => (0.5, 0.8, 0.9),
        Shadow => (0.3, 0.1, 0.4),
        Light => (0.9, 0.9, 0.5),
        Normal => (0.5, 0.5, 0.5),
    };
    (Color::srgb(r, g, b), type_emoji(t))
}

// ── Système de gestion des inputs ────────────────────────────────

#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_monster_detail_input(
    mut commands: Commands,
    mut data: ResMut<GameData>,
    selected: Res<SelectedMonsterIndex>,
    mut confirm: ResMut<ReleaseConfirmOpen>,
    mut next_state: ResMut<NextState<GameScreen>>,
    screen_entities: Query<Entity, With<ScreenEntity>>,
    metrics: Res<ScreenMetrics>,
    back_query: Query<&Interaction, (Changed<Interaction>, With<DetailBackButton>)>,
    attacks_query: Query<&Interaction, (Changed<Interaction>, With<ManageAttacksButton>)>,
    rename_query: Query<&Interaction, (Changed<Interaction>, With<RenameButton>)>,
    devour_query: Query<&Interaction, (Changed<Interaction>, With<DevourButton>)>,
    release_query: Query<&Interaction, (Changed<Interaction>, With<ReleaseButton>)>,
    confirm_query: Query<&Interaction, (Changed<Interaction>, With<ReleaseConfirmButton>)>,
    cancel_query: Query<&Interaction, (Changed<Interaction>, With<ReleaseCancelButton>)>,
) {
    let idx = selected.0;
    let mut needs_rebuild = false;

    // Retour → MonsterList
    for interaction in &back_query {
        if *interaction == Interaction::Pressed {
            next_state.set(GameScreen::MonsterList);
            return;
        }
    }

    // Gérer les attaques → SelectAttacks
    for interaction in &attacks_query {
        if *interaction == Interaction::Pressed {
            data.monster_select_index = idx;
            next_state.set(GameScreen::SelectAttacks);
            return;
        }
    }

    // Renommer → NamingMonster (flag rename implicite via monster_select_index)
    for interaction in &rename_query {
        if *interaction == Interaction::Pressed {
            data.monster_select_index = idx;
            data.name_input.clear();
            next_state.set(GameScreen::NamingMonster);
            return;
        }
    }

    // Dévorer → DevourSelect
    for interaction in &devour_query {
        if *interaction == Interaction::Pressed {
            commands.insert_resource(DevourTargetIndex(0));
            next_state.set(GameScreen::DevourSelect);
            return;
        }
    }

    // Relâcher → ouvrir popup de confirmation
    for interaction in &release_query {
        if *interaction == Interaction::Pressed {
            confirm.0 = true;
            needs_rebuild = true;
            break;
        }
    }

    // Popup : confirmer relâche
    for interaction in &confirm_query {
        if *interaction == Interaction::Pressed {
            let monsters = data.storage.list_alive().unwrap_or_default();
            let real_idx = idx.min(monsters.len().saturating_sub(1));
            if let Some(monster) = monsters.get(real_idx) {
                let id = monster.id;
                let _ = data.storage.delete(id);
            }
            confirm.0 = false;
            next_state.set(GameScreen::MonsterList);
            return;
        }
    }

    // Popup : annuler relâche
    for interaction in &cancel_query {
        if *interaction == Interaction::Pressed {
            confirm.0 = false;
            needs_rebuild = true;
            break;
        }
    }

    if needs_rebuild {
        for entity in &screen_entities {
            commands.entity(entity).despawn_recursive();
        }
        spawn_monster_detail_inner(
            &mut commands,
            &data,
            idx,
            confirm.0,
            metrics.safe_top,
            metrics.safe_bottom,
        );
    }
}

// ════════════════════════════════════════════════════════════════
//  Écran DevourSelect
// ════════════════════════════════════════════════════════════════

/// Marqueur pour un bouton de cible de dévoration.
#[derive(Component)]
pub(crate) struct DevourTargetButton {
    pub index: usize,
}

/// Marqueur pour le bouton retour de l'écran DevourSelect.
#[derive(Component)]
pub(crate) struct DevourBackButton;

/// Marqueur pour le bouton de confirmation de la popup dévoration.
#[derive(Component)]
pub(crate) struct DevourConfirmButton;

/// Marqueur pour le bouton d'annulation de la popup dévoration.
#[derive(Component)]
pub(crate) struct DevourCancelButton;

/// Ressource locale : popup de confirmation ouverte + index de la proie sélectionnée.
#[derive(Resource, Default)]
pub(crate) struct DevourConfirmState {
    pub open: bool,
    /// Index dans la liste filtrée (sans le prédateur).
    pub prey_filtered_index: usize,
}

/// Point d'entrée spawn DevourSelect.
pub(crate) fn spawn_devour_select(
    mut commands: Commands,
    data: Res<GameData>,
    selected: Res<SelectedMonsterIndex>,
    devour_target: Res<DevourTargetIndex>,
    confirm: Res<DevourConfirmState>,
    metrics: Res<ScreenMetrics>,
) {
    spawn_devour_select_inner(
        &mut commands,
        &data,
        selected.0,
        devour_target.0,
        confirm.open,
        confirm.prey_filtered_index,
        metrics.safe_top,
        metrics.safe_bottom,
    );
}

#[allow(clippy::too_many_arguments)]
fn spawn_devour_select_inner(
    commands: &mut Commands,
    data: &GameData,
    predator_idx: usize,
    _selected_target: usize,
    confirm_open: bool,
    prey_filtered_index: usize,
    safe_top: f32,
    safe_bottom: f32,
) {
    let all_monsters = data.storage.list_alive().unwrap_or_default();
    let predator_idx = predator_idx.min(all_monsters.len().saturating_sub(1));
    let predator = all_monsters.get(predator_idx);

    // Liste des proies potentielles (tous sauf le prédateur)
    let prey_list: Vec<(usize, &monster_battle_core::Monster)> = all_monsters
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != predator_idx)
        .collect();

    let _prey_for_popup = prey_list.get(prey_filtered_index);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::new(
                    Val::Px(12.0),
                    Val::Px(12.0),
                    Val::Px(safe_top),
                    Val::Px(safe_bottom),
                ),
                ..default()
            },
            BackgroundColor(colors::BACKGROUND),
            ScreenEntity,
            bevy::state::state_scoped::StateScoped(GameScreen::DevourSelect),
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
                        DevourBackButton,
                        Interaction::default(),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("< Retour"),
                            TextFont { font_size: fonts::SMALL, ..default() },
                            TextColor(colors::TEXT_PRIMARY),
                        ));
                    });
                });

            // Titre
            let pred_name = predator
                .map(|m| m.name.as_str())
                .unwrap_or("?");
            parent.spawn((
                Text::new(format!("🐲 {} dévore…", pred_name)),
                TextFont { font_size: fonts::HEADING, ..default() },
                TextColor(colors::ACCENT_RED),
                Node { margin: UiRect::bottom(Val::Px(12.0)), ..default() },
            ));

            if prey_list.is_empty() {
                parent.spawn((
                    Text::new("Aucun autre monstre disponible."),
                    TextFont { font_size: fonts::BODY, ..default() },
                    TextColor(colors::TEXT_SECONDARY),
                ));
            } else {
                // Liste scrollable des proies
                parent
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            overflow: Overflow::scroll_y(),
                            flex_grow: 1.0,
                            row_gap: Val::Px(6.0),
                            ..default()
                        },
                        ScrollPosition::default(),
                        ScrollableContent,
                    ))
                    .with_children(|list| {
                        for (filtered_idx, (_, prey)) in prey_list.iter().enumerate() {
                            let secondary = prey
                                .secondary_type
                                .map(|t| format!("/{}", t))
                                .unwrap_or_default();

                            list.spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    padding: UiRect::all(Val::Px(10.0)),
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(2.0),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BorderColor(colors::BORDER),
                                BorderRadius::all(Val::Px(6.0)),
                                BackgroundColor(colors::PANEL),
                                DevourTargetButton { index: filtered_idx },
                                Interaction::default(),
                            ))
                            .with_children(|card| {
                                card.spawn((
                                    Text::new(format!(
                                        "{} {} Nv.{}",
                                        type_emoji(prey.primary_type),
                                        prey.name,
                                        prey.level,
                                    )),
                                    TextFont { font_size: fonts::BODY, ..default() },
                                    TextColor(colors::TEXT_PRIMARY),
                                ));
                                card.spawn((
                                    Text::new(format!(
                                        "Type: {}{}  PV: {}/{}",
                                        prey.primary_type,
                                        secondary,
                                        prey.current_hp,
                                        prey.max_hp(),
                                    )),
                                    TextFont { font_size: fonts::SMALL, ..default() },
                                    TextColor(colors::TEXT_SECONDARY),
                                ));
                            });
                        }
                    });
            }

            // Popup confirmation dévoration
            if confirm_open {
                if let (Some(pred), Some((_, prey))) =
                    (predator, prey_list.get(prey_filtered_index))
                {
                    spawn_devour_confirm_popup(parent, &pred.name, &prey.name);
                }
            }
        });
}

fn spawn_devour_confirm_popup(parent: &mut ChildBuilder, pred_name: &str, prey_name: &str) {
    parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(20.0)),
                        row_gap: Val::Px(12.0),
                        max_width: Val::Px(320.0),
                        ..default()
                    },
                    BackgroundColor(colors::PANEL),
                    BorderRadius::all(Val::Px(12.0)),
                ))
                .with_children(|popup| {
                    popup.spawn((
                        Text::new(format!(
                            "Voulez-vous que {} dévore {} ?\nCette action est irréversible.",
                            pred_name, prey_name
                        )),
                        TextFont { font_size: fonts::BODY, ..default() },
                        TextColor(colors::TEXT_PRIMARY),
                    ));

                    popup
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(8.0),
                            width: Val::Percent(100.0),
                            ..default()
                        })
                        .with_children(|row| {
                            // Confirmer
                            row.spawn((
                                Node {
                                    flex_grow: 1.0,
                                    padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                BackgroundColor(colors::ACCENT_RED),
                                BorderRadius::all(Val::Px(6.0)),
                                DevourConfirmButton,
                                Interaction::default(),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Dévorer !"),
                                    TextFont { font_size: fonts::SMALL, ..default() },
                                    TextColor(Color::WHITE),
                                ));
                            });

                            // Annuler
                            row.spawn((
                                Node {
                                    flex_grow: 1.0,
                                    padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                BackgroundColor(colors::PANEL),
                                BorderRadius::all(Val::Px(6.0)),
                                DevourCancelButton,
                                Interaction::default(),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Annuler"),
                                    TextFont { font_size: fonts::SMALL, ..default() },
                                    TextColor(colors::TEXT_PRIMARY),
                                ));
                            });
                        });
                });
        });
}

/// Gestion des inputs de l'écran DevourSelect.
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_devour_select_input(
    mut commands: Commands,
    mut data: ResMut<GameData>,
    selected: Res<SelectedMonsterIndex>,
    mut devour_target: ResMut<DevourTargetIndex>,
    mut confirm: ResMut<DevourConfirmState>,
    mut next_state: ResMut<NextState<GameScreen>>,
    screen_entities: Query<Entity, With<ScreenEntity>>,
    metrics: Res<ScreenMetrics>,
    back_query: Query<&Interaction, (Changed<Interaction>, With<DevourBackButton>)>,
    target_query: Query<(&Interaction, &DevourTargetButton), Changed<Interaction>>,
    confirm_query: Query<&Interaction, (Changed<Interaction>, With<DevourConfirmButton>)>,
    cancel_query: Query<&Interaction, (Changed<Interaction>, With<DevourCancelButton>)>,
) {
    let predator_idx = selected.0;
    let mut needs_rebuild = false;

    // Retour → MonsterDetail
    for interaction in &back_query {
        if *interaction == Interaction::Pressed {
            confirm.open = false;
            next_state.set(GameScreen::MonsterDetail);
            return;
        }
    }

    // Tap proie → ouvrir popup
    for (interaction, btn) in &target_query {
        if *interaction == Interaction::Pressed {
            confirm.open = true;
            confirm.prey_filtered_index = btn.index;
            devour_target.0 = btn.index;
            needs_rebuild = true;
            break;
        }
    }

    // Confirmer la dévoration
    for interaction in &confirm_query {
        if *interaction == Interaction::Pressed {
            let mut all_monsters = data.storage.list_alive().unwrap_or_default();
            let real_pred_idx = predator_idx.min(all_monsters.len().saturating_sub(1));

            // Construire la liste filtrée (proies)
            let prey_indices: Vec<usize> = (0..all_monsters.len())
                .filter(|i| *i != real_pred_idx)
                .collect();

            if let Some(&prey_real_idx) = prey_indices.get(confirm.prey_filtered_index) {
                // Cloner la proie avant de muter le prédateur
                let prey_clone = all_monsters[prey_real_idx].clone();
                let prey_id = prey_clone.id;

                // Appel devour (modifie le prédateur in-place)
                let result = all_monsters[real_pred_idx].devour(&prey_clone);

                // Sauvegarder le prédateur modifié
                let predator_clone = all_monsters[real_pred_idx].clone();
                let _ = data.storage.save(&predator_clone);

                // Supprimer la proie du storage
                let _ = data.storage.delete(prey_id);

                // Afficher le résultat
                data.message = Some(result.description);
            }

            confirm.open = false;
            next_state.set(GameScreen::MonsterDetail);
            return;
        }
    }

    // Annuler popup
    for interaction in &cancel_query {
        if *interaction == Interaction::Pressed {
            confirm.open = false;
            needs_rebuild = true;
            break;
        }
    }

    if needs_rebuild {
        for entity in &screen_entities {
            commands.entity(entity).despawn_recursive();
        }
        spawn_devour_select_inner(
            &mut commands,
            &data,
            predator_idx,
            devour_target.0,
            confirm.open,
            confirm.prey_filtered_index,
            metrics.safe_top,
            metrics.safe_bottom,
        );
    }
}
