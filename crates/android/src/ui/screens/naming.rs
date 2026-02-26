//! Écran de saisie du nom du monstre.

use bevy::prelude::*;
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::state::state::NextState;

use monster_battle_core::Monster;
use monster_battle_core::genetics::generate_starter_stats;
use monster_battle_core::types::ElementType;
use monster_battle_storage::MonsterStorage;

use crate::game::{GameData, GameScreen, ScreenEntity};
use crate::ui::common::{colors, fonts};

/// Marqueur pour le bouton « Confirmer ».
#[derive(Component)]
pub(crate) struct ConfirmButton;

/// Construit l'UI de saisie du nom.
pub(crate) fn spawn_naming(mut commands: Commands, data: Res<GameData>) {
    let types = ElementType::all();
    let chosen = types[data.type_choice_index % types.len()];

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(colors::BACKGROUND),
            ScreenEntity,
        ))
        .with_children(|parent| {
            // Info type choisi
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        padding: UiRect::all(Val::Px(12.0)),
                        margin: UiRect::bottom(Val::Px(16.0)),
                        ..default()
                    },
                    BackgroundColor(colors::PANEL),
                    BorderRadius::all(Val::Px(8.0)),
                ))
                .with_children(|p| {
                    p.spawn((
                        Text::new(format!("Type choisi : {} {}", chosen.icon(), chosen)),
                        TextFont {
                            font_size: fonts::BODY,
                            ..default()
                        },
                        TextColor(colors::ACCENT_YELLOW),
                    ));
                });

            // Titre
            parent.spawn((
                Text::new("Nom de votre monstre"),
                TextFont {
                    font_size: fonts::HEADING,
                    ..default()
                },
                TextColor(colors::ACCENT_MAGENTA),
                Node {
                    margin: UiRect::bottom(Val::Px(12.0)),
                    ..default()
                },
            ));

            // Champ de saisie visuel
            parent
                .spawn((
                    Node {
                        width: Val::Percent(90.0),
                        padding: UiRect::all(Val::Px(14.0)),
                        margin: UiRect::bottom(Val::Px(12.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(colors::PANEL),
                    BorderColor(colors::ACCENT_MAGENTA),
                    BorderRadius::all(Val::Px(8.0)),
                ))
                .with_children(|p| {
                    let display = if data.name_input.is_empty() {
                        "Tapez un nom...".to_string()
                    } else {
                        format!("{}█", data.name_input)
                    };
                    let color = if data.name_input.is_empty() {
                        colors::TEXT_SECONDARY
                    } else {
                        colors::TEXT_PRIMARY
                    };
                    p.spawn((
                        Text::new(display),
                        TextFont {
                            font_size: fonts::BODY,
                            ..default()
                        },
                        TextColor(color),
                    ));
                });

            // Hint
            parent.spawn((
                Text::new("Choisissez un nom unique pour votre compagnon ! (max 20 car.)"),
                TextFont {
                    font_size: fonts::SMALL,
                    ..default()
                },
                TextColor(colors::TEXT_SECONDARY),
                Node {
                    margin: UiRect::bottom(Val::Px(16.0)),
                    ..default()
                },
            ));

            // Bouton confirmer
            parent
                .spawn((
                    Node {
                        width: Val::Percent(60.0),
                        padding: UiRect::axes(Val::Px(20.0), Val::Px(14.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(colors::ACCENT_GREEN),
                    BorderRadius::all(Val::Px(8.0)),
                    ConfirmButton,
                    Interaction::default(),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("✅ Confirmer"),
                        TextFont {
                            font_size: fonts::BODY,
                            ..default()
                        },
                        TextColor(Color::BLACK),
                    ));
                });

            // Message d'erreur éventuel
            if let Some(ref msg) = data.message {
                parent.spawn((
                    Text::new(msg.clone()),
                    TextFont {
                        font_size: fonts::SMALL,
                        ..default()
                    },
                    TextColor(colors::ACCENT_RED),
                    Node {
                        margin: UiRect::top(Val::Px(12.0)),
                        ..default()
                    },
                ));
            }

            // Footer
            parent.spawn((
                Text::new("⏎ Confirmer  Esc Retour"),
                TextFont {
                    font_size: fonts::SMALL,
                    ..default()
                },
                TextColor(colors::TEXT_SECONDARY),
                Node {
                    margin: UiRect::top(Val::Px(16.0)),
                    ..default()
                },
            ));
        });
}

/// Gestion des entrées de la saisie du nom.
pub(crate) fn handle_naming_input(
    mut data: ResMut<GameData>,
    keyboard: Res<ButtonInput<KeyCode>>,
    key_events: EventReader<KeyboardInput>,
    mut next_state: ResMut<NextState<GameScreen>>,
    interaction_query: Query<(&Interaction, &ConfirmButton), Changed<Interaction>>,
) {
    // Toucher le bouton confirmer
    for (interaction, _) in &interaction_query {
        if *interaction == Interaction::Pressed {
            try_create_monster(&mut data, &mut next_state);
            return;
        }
    }

    // Saisie clavier
    handle_text_input(&mut data, &keyboard, key_events);

    if keyboard.just_pressed(KeyCode::Enter) {
        try_create_monster(&mut data, &mut next_state);
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        data.name_input.clear();
        data.message = None;
        next_state.set(GameScreen::NewMonster);
    }
}

/// Gestion de la saisie de texte clavier.
fn handle_text_input(
    data: &mut ResMut<GameData>,
    keyboard: &Res<ButtonInput<KeyCode>>,
    mut key_events: EventReader<KeyboardInput>,
) {
    for ev in key_events.read() {
        if ev.state != ButtonState::Pressed {
            continue;
        }
        if let Key::Character(ref s) = ev.logical_key {
            for c in s.chars() {
                if !c.is_control() && data.name_input.len() < 20 {
                    data.name_input.push(c);
                }
            }
        }
    }

    if keyboard.just_pressed(KeyCode::Backspace) {
        data.name_input.pop();
    }
}

/// Tente de créer le monstre starter.
fn try_create_monster(data: &mut ResMut<GameData>, next_state: &mut ResMut<NextState<GameScreen>>) {
    if data.name_input.trim().is_empty() {
        data.message = Some("Le nom ne peut pas être vide !".to_string());
        return;
    }

    if data.has_living_monster() {
        data.message = Some("Vous avez déjà un monstre vivant !".to_string());
        next_state.set(GameScreen::MainMenu);
        data.menu_index = 0;
        return;
    }

    let types = ElementType::all();
    let chosen_type = types[data.type_choice_index % types.len()];
    let stats = generate_starter_stats(chosen_type);
    let name = data.name_input.trim().to_string();

    let monster = Monster::new_starter(name.clone(), chosen_type, stats);

    match data.storage.save(&monster) {
        Ok(()) => {
            data.message = Some(format!("🥚 {} est né ! Prenez-en soin.", name));
        }
        Err(e) => {
            data.message = Some(format!("Erreur : {}", e));
        }
    }

    data.name_input.clear();
    next_state.set(GameScreen::MonsterList);
    data.menu_index = 0;
}
