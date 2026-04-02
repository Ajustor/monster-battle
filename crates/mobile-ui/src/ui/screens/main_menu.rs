//! Écran du menu principal.

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::state::state::NextState;

use crate::connection::{UpdateButton, VersionState, VersionStatus};
use crate::game::{GameData, GameScreen, ScreenEntity, SelectMonsterTarget};
use crate::ui::common::{self, colors, fonts};

/// Marqueur pour les boutons du menu.
#[derive(Component)]
pub struct MenuButton {
    index: usize,
}

/// Construit l'UI du menu principal.
pub(crate) fn spawn_menu(mut commands: Commands, data: Res<GameData>, version: Res<VersionState>) {
    let has_monster = data.has_living_monster();
    log::info!("🖥️ spawn_menu — has_monster={}", has_monster);

    // Nœud racine
    commands
        .spawn((
            common::screen_root(),
            BackgroundColor(colors::BACKGROUND),
            ScreenEntity,
            bevy::state::state_scoped::StateScoped(GameScreen::MainMenu),
        ))
        .with_children(|parent| {
            // Titre
            parent.spawn((
                Text::new("~ Monster Battle ~"),
                TextFont {
                    font_size: fonts::TITLE,
                    ..default()
                },
                TextColor(colors::ACCENT_YELLOW),
                Node {
                    margin: UiRect::bottom(Val::Px(32.0)),
                    ..default()
                },
            ));

            // Bannière de mise à jour si version incompatible
            if let VersionStatus::UpdateRequired { server_version } = &version.status {
                let client_version = env!("CARGO_PKG_VERSION");
                parent
                    .spawn((
                        Node {
                            width: Val::Percent(90.0),
                            padding: UiRect::all(Val::Px(12.0)),
                            margin: UiRect::bottom(Val::Px(16.0)),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: Val::Px(8.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.8, 0.2, 0.1, 0.9)),
                        BorderRadius::all(Val::Px(8.0)),
                    ))
                    .with_children(|banner| {
                        banner.spawn((
                            Text::new("⚠ Mise à jour requise !"),
                            TextFont {
                                font_size: fonts::BODY,
                                ..default()
                            },
                            TextColor(colors::ACCENT_YELLOW),
                        ));
                        banner.spawn((
                            Text::new(format!(
                                "Votre version: {}  →  Serveur: {}",
                                client_version, server_version
                            )),
                            TextFont {
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                        // Bouton de téléchargement
                        banner
                            .spawn((
                                Node {
                                    padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                                    margin: UiRect::top(Val::Px(4.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(colors::ACCENT_YELLOW),
                                BorderRadius::all(Val::Px(6.0)),
                                UpdateButton,
                                Interaction::default(),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Telecharger la mise a jour"),
                                    TextFont {
                                        font_size: fonts::BODY,
                                        ..default()
                                    },
                                    TextColor(Color::BLACK),
                                ));
                            });
                    });
            }

            // Afficher un message temporaire s'il existe
            if let Some(ref msg) = data.message {
                parent.spawn((
                    Text::new(msg.clone()),
                    TextFont {
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(colors::ACCENT_YELLOW),
                    Node {
                        margin: UiRect::bottom(Val::Px(12.0)),
                        ..default()
                    },
                ));
            }

            // Entrées du menu
            let mut entries: Vec<(&str, bool)> = vec![("Mes Monstres", true)];

            if !has_monster {
                entries.push(("Nouveau Monstre", true));
            }
            if has_monster {
                entries.push(("Entrainement", true));
                entries.push(("Combat PvP", true));
                entries.push(("Reproduction", true));
                entries.push(("Mini-jeux", true));
            }
            entries.push(("Cimetiere", true));
            entries.push(("Aide", true));
            entries.push(("Quitter", true));

            for (i, (label, _enabled)) in entries.iter().enumerate() {
                let selected = i == data.menu_index % entries.len();

                let bg_color = if selected {
                    colors::ACCENT_YELLOW
                } else {
                    colors::PANEL
                };
                let text_color = if selected {
                    Color::BLACK
                } else {
                    colors::TEXT_PRIMARY
                };

                parent
                    .spawn((
                        Node {
                            width: Val::Percent(90.0),
                            padding: UiRect::axes(Val::Px(20.0), Val::Px(14.0)),
                            margin: UiRect::bottom(Val::Px(8.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(bg_color),
                        BorderRadius::all(Val::Px(8.0)),
                        MenuButton { index: i },
                        Interaction::default(),
                    ))
                    .with_children(|p| {
                        p.spawn((
                            Text::new(label.to_string()),
                            TextFont {
                                font_size: fonts::BODY,
                                ..default()
                            },
                            TextColor(text_color),
                        ));
                    });
            }
        });
}

/// Gestion des entrées du menu principal.
///
/// Sur mobile : toucher un bouton = sélectionner + entrer.
/// Clavier (desktop dev) : Up/Down + Enter.
pub(crate) fn handle_menu_input(
    mut commands: Commands,
    mut data: ResMut<GameData>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameScreen>>,
    interaction_query: Query<(&Interaction, &MenuButton), Changed<Interaction>>,
    mut exit_events: EventWriter<AppExit>,
    version: Res<VersionState>,
) {
    let has_monster = data.has_living_monster();
    let entry_count = menu_entry_count(has_monster);
    let version_ok = !matches!(version.status, VersionStatus::UpdateRequired { .. });

    // ── Toucher (mobile) ─────────────────────────────────────────
    for (interaction, button) in &interaction_query {
        if *interaction == Interaction::Pressed {
            data.menu_index = button.index;
            activate_menu_entry(
                &mut commands,
                &mut data,
                &mut next_state,
                has_monster,
                &mut exit_events,
                version_ok,
            );
            return;
        }
    }

    // ── Clavier (desktop dev) ────────────────────────────────────
    if keyboard.just_pressed(KeyCode::ArrowUp)
        && data.menu_index > 0 {
            data.menu_index -= 1;
        }
    if keyboard.just_pressed(KeyCode::ArrowDown)
        && data.menu_index < entry_count - 1 {
            data.menu_index += 1;
        }
    if keyboard.just_pressed(KeyCode::Enter) {
        activate_menu_entry(
            &mut commands,
            &mut data,
            &mut next_state,
            has_monster,
            &mut exit_events,
            version_ok,
        );
    }
    if keyboard.just_pressed(KeyCode::KeyQ) {
        exit_events.send(AppExit::Success);
    }
}

fn menu_entry_count(has_monster: bool) -> usize {
    let mut count = 1; // Mes Monstres
    if !has_monster {
        count += 1; // Nouveau Monstre
    }
    if has_monster {
        count += 4; // Entraînement, Combat PvP, Reproduction, Mini-jeux
    }
    count += 3; // Cimetière, Aide, Quitter
    count
}

fn activate_menu_entry(
    commands: &mut Commands,
    data: &mut ResMut<GameData>,
    next_state: &mut ResMut<NextState<GameScreen>>,
    has_monster: bool,
    exit_events: &mut EventWriter<AppExit>,
    version_ok: bool,
) {
    let mut idx = 0;

    // Mes Monstres
    if data.menu_index == idx {
        next_state.set(GameScreen::MonsterList);
        data.menu_index = 0;
        return;
    }
    idx += 1;

    // Nouveau Monstre (si aucun monstre vivant)
    if !has_monster {
        if data.menu_index == idx {
            next_state.set(GameScreen::NewMonster);
            data.menu_index = 0;
            return;
        }
        idx += 1;
    }

    // Entraînement / PvP / Reproduction (si monstre vivant)
    if has_monster {
        if data.menu_index == idx {
            commands.insert_resource(SelectMonsterTarget::Training);
            data.monster_select_index = 0;
            next_state.set(GameScreen::SelectMonster);
            data.menu_index = 0;
            return;
        }
        idx += 1;

        // Combat PvP — bloqué si la version ne correspond pas
        if data.menu_index == idx {
            if !version_ok {
                data.message = Some("Mise a jour requise pour le PvP !".to_string());
                return;
            }
            commands.insert_resource(SelectMonsterTarget::CombatPvP);
            data.monster_select_index = 0;
            next_state.set(GameScreen::SelectMonster);
            data.menu_index = 0;
            return;
        }
        idx += 1;

        // Reproduction — bloquée si la version ne correspond pas
        if data.menu_index == idx {
            if !version_ok {
                data.message = Some("Mise a jour requise pour la reproduction !".to_string());
                return;
            }
            commands.insert_resource(SelectMonsterTarget::Breeding);
            data.monster_select_index = 0;
            next_state.set(GameScreen::SelectMonster);
            data.menu_index = 0;
            return;
        }
        idx += 1;

        if data.menu_index == idx {
            commands.insert_resource(SelectMonsterTarget::Minigame);
            data.monster_select_index = 0;
            next_state.set(GameScreen::SelectMonster);
            data.menu_index = 0;
            return;
        }
        idx += 1;
    }

    // Cimetière
    if data.menu_index == idx {
        next_state.set(GameScreen::Cemetery);
        data.menu_index = 0;
        return;
    }
    idx += 1;

    // Aide
    if data.menu_index == idx {
        next_state.set(GameScreen::Help);
        data.menu_index = 0;
        return;
    }
    idx += 1;

    // Quitter
    if data.menu_index == idx {
        exit_events.send(AppExit::Success);
    }
}
