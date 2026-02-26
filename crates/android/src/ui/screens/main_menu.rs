//! Écran du menu principal.

use bevy::prelude::*;
use bevy::state::state::NextState;

use crate::game::{GameData, GameScreen, ScreenEntity, SelectMonsterTarget};
use crate::ui::common::{self, colors, fonts};

/// Marqueur pour les boutons du menu.
#[derive(Component)]
pub struct MenuButton {
    index: usize,
}

/// Construit l'UI du menu principal.
pub(crate) fn spawn_menu(mut commands: Commands, data: Res<GameData>) {
    let has_monster = data.has_living_monster();
    log::info!("🖥️ spawn_menu — has_monster={}", has_monster);

    // Nœud racine
    commands
        .spawn((
            common::screen_root(),
            BackgroundColor(colors::BACKGROUND),
            ScreenEntity,
        ))
        .with_children(|parent| {
            // Titre
            parent.spawn((
                Text::new("🐉 Monster Battle"),
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

            // Entrées du menu
            let mut entries: Vec<(&str, bool)> = vec![("🐾 Mes Monstres", true)];

            if !has_monster {
                entries.push(("🥚 Nouveau Monstre", true));
            }
            if has_monster {
                entries.push(("⚔️  Entraînement", true));
                entries.push(("🗡️  Combat PvP", true));
                entries.push(("🧬 Reproduction", true));
            }
            entries.push(("💀 Cimetière", true));
            entries.push(("❓ Aide", true));
            entries.push(("🚪 Quitter", true));

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
) {
    let has_monster = data.has_living_monster();
    let entry_count = menu_entry_count(has_monster);

    // ── Toucher (mobile) ─────────────────────────────────────────
    for (interaction, button) in &interaction_query {
        if *interaction == Interaction::Pressed {
            data.menu_index = button.index;
            activate_menu_entry(&mut commands, &mut data, &mut next_state, has_monster);
            return;
        }
    }

    // ── Clavier (desktop dev) ────────────────────────────────────
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        if data.menu_index > 0 {
            data.menu_index -= 1;
        }
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        if data.menu_index < entry_count - 1 {
            data.menu_index += 1;
        }
    }
    if keyboard.just_pressed(KeyCode::Enter) {
        activate_menu_entry(&mut commands, &mut data, &mut next_state, has_monster);
    }
    if keyboard.just_pressed(KeyCode::KeyQ) {
        std::process::exit(0);
    }
}

fn menu_entry_count(has_monster: bool) -> usize {
    let mut count = 1; // Mes Monstres
    if !has_monster {
        count += 1; // Nouveau Monstre
    }
    if has_monster {
        count += 3; // Entraînement, Combat PvP, Reproduction
    }
    count += 3; // Cimetière, Aide, Quitter
    count
}

fn activate_menu_entry(
    commands: &mut Commands,
    data: &mut ResMut<GameData>,
    next_state: &mut ResMut<NextState<GameScreen>>,
    has_monster: bool,
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

        if data.menu_index == idx {
            commands.insert_resource(SelectMonsterTarget::CombatPvP);
            data.monster_select_index = 0;
            next_state.set(GameScreen::SelectMonster);
            data.menu_index = 0;
            return;
        }
        idx += 1;

        if data.menu_index == idx {
            commands.insert_resource(SelectMonsterTarget::Breeding);
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
        std::process::exit(0);
    }
}
