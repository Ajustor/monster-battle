//! Écrans de reproduction (searching / naming / result).

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use bevy::state::state::NextState;
use bevy::window::Ime;
use monster_battle_storage::MonsterStorage;

use crate::game::{GameData, GameScreen, ScreenEntity};
use crate::ui::common::{
    InputDisplayText, KEYBOARD_SCROLL_STEP, SAFE_BOTTOM, SAFE_TOP, ScrollableContent,
    TextInputField, colors, fonts,
};

// ═══════════════════════════════════════════════════════════════════
//  Breeding Searching
// ═══════════════════════════════════════════════════════════════════

/// Marqueur du bouton d'annulation.
#[derive(Component)]
pub(crate) struct CancelButton;

/// Construit l'UI de recherche de partenaire.
pub(crate) fn spawn_breeding_searching(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::new(
                    Val::Px(24.0),
                    Val::Px(24.0),
                    Val::Px(SAFE_TOP),
                    Val::Px(SAFE_BOTTOM),
                ),
                ..default()
            },
            BackgroundColor(colors::BACKGROUND),
            ScreenEntity,
            bevy::state::state_scoped::StateScoped(GameScreen::BreedingSearching),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Reproduction"),
                TextFont {
                    font_size: fonts::TITLE,
                    ..default()
                },
                TextColor(colors::ACCENT_MAGENTA),
                Node {
                    margin: UiRect::bottom(Val::Px(24.0)),
                    ..default()
                },
            ));

            parent.spawn((
                Text::new("Recherche d'un partenaire de reproduction..."),
                TextFont {
                    font_size: fonts::BODY,
                    ..default()
                },
                TextColor(colors::ACCENT_MAGENTA),
                Node {
                    margin: UiRect::bottom(Val::Px(16.0)),
                    ..default()
                },
            ));

            parent.spawn((
                Text::new(
                    "La reproduction commencera automatiquement\n\
                     dès qu'un autre joueur sera trouvé.",
                ),
                TextFont {
                    font_size: fonts::SMALL,
                    ..default()
                },
                TextColor(colors::TEXT_SECONDARY),
                Node {
                    margin: UiRect::bottom(Val::Px(24.0)),
                    ..default()
                },
            ));

            // Bouton Annuler
            parent
                .spawn((
                    Node {
                        padding: UiRect::axes(Val::Px(24.0), Val::Px(12.0)),
                        ..default()
                    },
                    BackgroundColor(colors::ACCENT_RED),
                    BorderRadius::all(Val::Px(8.0)),
                    Interaction::default(),
                    CancelButton,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Annuler (Esc)"),
                        TextFont {
                            font_size: fonts::BODY,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

/// Gestion des entrées en recherche de partenaire.
pub(crate) fn handle_breeding_searching_input(
    mut commands: Commands,
    mut data: ResMut<GameData>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameScreen>>,
    interaction_query: Query<(&Interaction, &CancelButton), Changed<Interaction>>,
) {
    for (interaction, _) in &interaction_query {
        if *interaction == Interaction::Pressed {
            commands.remove_resource::<crate::net_task::NetTask>();
            data.message = None;
            data.remote_monster = None;
            next_state.set(GameScreen::MainMenu);
            data.menu_index = 0;
            return;
        }
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        commands.remove_resource::<crate::net_task::NetTask>();
        data.message = None;
        data.remote_monster = None;
        next_state.set(GameScreen::MainMenu);
        data.menu_index = 0;
    }
}

// ═══════════════════════════════════════════════════════════════════
//  Breeding Naming (nom du bébé)
// ═══════════════════════════════════════════════════════════════════

/// Marqueur du bouton confirmer.
#[derive(Component)]
pub(crate) struct ConfirmButton;

/// Marqueur pour le bouton retour (nommage reproduction).
#[derive(Component)]
pub(crate) struct BreedingNamingBackButton;

/// Construit l'UI de nommage du bébé.
pub(crate) fn spawn_breeding_naming(mut commands: Commands, data: Res<GameData>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::new(
                    Val::Px(16.0),
                    Val::Px(16.0),
                    Val::Px(SAFE_TOP),
                    Val::Px(SAFE_BOTTOM),
                ),
                ..default()
            },
            BackgroundColor(colors::BACKGROUND),
            ScreenEntity,
            bevy::state::state_scoped::StateScoped(GameScreen::BreedingNaming),
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
                        BreedingNamingBackButton,
                        Interaction::default(),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("< Annuler"),
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
                Text::new("La reproduction a reussi !"),
                TextFont {
                    font_size: fonts::HEADING,
                    ..default()
                },
                TextColor(colors::ACCENT_MAGENTA),
                Node {
                    margin: UiRect::bottom(Val::Px(16.0)),
                    ..default()
                },
            ));

            parent.spawn((
                Text::new("Donnez un nom au nouveau monstre :"),
                TextFont {
                    font_size: fonts::BODY,
                    ..default()
                },
                TextColor(colors::TEXT_PRIMARY),
                Node {
                    margin: UiRect::bottom(Val::Px(12.0)),
                    ..default()
                },
            ));

            // Champ de saisie visuel (tap = ouvrir clavier système)
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
                    TextInputField,
                    Interaction::default(),
                ))
                .with_children(|p| {
                    let display = if data.name_input.is_empty() {
                        "Toucher pour saisir...".to_string()
                    } else {
                        format!("{}|", data.name_input)
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
                        InputDisplayText,
                    ));
                });

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
                        Text::new("Confirmer"),
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
        });
}

/// Gestion des entrées de nommage du bébé.
pub(crate) fn handle_breeding_naming_input(
    mut data: ResMut<GameData>,
    keyboard: Res<ButtonInput<KeyCode>>,
    key_events: EventReader<KeyboardInput>,
    mut ime_events: EventReader<Ime>,
    mut next_state: ResMut<NextState<GameScreen>>,
    interaction_query: Query<(&Interaction, &ConfirmButton), Changed<Interaction>>,
    back_query: Query<&Interaction, (Changed<Interaction>, With<BreedingNamingBackButton>)>,
) {
    // Toucher annuler/retour
    for interaction in &back_query {
        if *interaction == Interaction::Pressed {
            data.name_input.clear();
            data.remote_monster = None;
            data.message = None;
            next_state.set(GameScreen::MainMenu);
            data.menu_index = 0;
            return;
        }
    }

    // Toucher confirmer
    for (interaction, _) in &interaction_query {
        if *interaction == Interaction::Pressed {
            try_confirm_breeding_name(&mut data, &mut next_state);
            return;
        }
    }

    // Gestion IME (clavier virtuel Android)
    for event in ime_events.read() {
        if let Ime::Commit { value, .. } = event {
            for c in value.chars() {
                if !c.is_control() && data.name_input.len() < 20 {
                    data.name_input.push(c);
                }
            }
        }
    }

    // Saisie texte
    handle_text_input(&mut data, &keyboard, key_events);

    if keyboard.just_pressed(KeyCode::Enter) {
        try_confirm_breeding_name(&mut data, &mut next_state);
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        data.name_input.clear();
        data.remote_monster = None;
        data.message = None;
        next_state.set(GameScreen::MainMenu);
        data.menu_index = 0;
    }
}

/// Saisie de texte (partagée).
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

/// Tente de confirmer le nom du bébé et passe au résultat.
fn try_confirm_breeding_name(
    data: &mut ResMut<GameData>,
    next_state: &mut ResMut<NextState<GameScreen>>,
) {
    if data.name_input.trim().is_empty() {
        data.message = Some("Le nom ne peut pas être vide !".to_string());
        return;
    }

    // Effectuer la reproduction
    let remote = match data.remote_monster.take() {
        Some(m) => m,
        None => {
            data.message = Some("Données du partenaire manquantes.".to_string());
            next_state.set(GameScreen::MainMenu);
            return;
        }
    };

    let monsters: Vec<monster_battle_core::Monster> = match data.storage.list_alive() {
        Ok(m) if !m.is_empty() => m,
        _ => {
            data.message = Some("Pas de monstre vivant !".to_string());
            next_state.set(GameScreen::MainMenu);
            return;
        }
    };

    let idx = data.monster_select_index.min(monsters.len() - 1);
    let parent = &monsters[idx];
    let child_name = data.name_input.trim().to_string();

    use monster_battle_core::genetics::breed;
    match breed(parent, &remote, child_name) {
        Ok(result) => {
            // Sauvegarder l'enfant
            match data.storage.save(&result.child) {
                Ok(()) => {
                    let mut msg = result.description.clone();
                    if result.mutation_occurred {
                        msg.push_str("\nUne mutation génétique s'est produite !");
                    }
                    data.message = Some(msg);
                    data.breed_result = Some(result.child);
                }
                Err(e) => {
                    data.message = Some(format!("Erreur de sauvegarde : {}", e));
                    data.breed_result = None;
                }
            }
        }
        Err(e) => {
            data.message = Some(format!("Erreur : {}", e));
            data.breed_result = None;
        }
    }

    data.name_input.clear();
    data.scroll_offset = 0;
    next_state.set(GameScreen::BreedingResult);
}

// ═══════════════════════════════════════════════════════════════════
//  Breeding Result
// ═══════════════════════════════════════════════════════════════════

/// Marqueur du bouton retour.
#[derive(Component)]
pub(crate) struct BackButton;

/// Construit l'UI du résultat de reproduction.
pub(crate) fn spawn_breeding_result(mut commands: Commands, data: Res<GameData>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::new(
                    Val::Px(16.0),
                    Val::Px(16.0),
                    Val::Px(SAFE_TOP),
                    Val::Px(SAFE_BOTTOM),
                ),
                ..default()
            },
            BackgroundColor(colors::BACKGROUND),
            ScreenEntity,
            bevy::state::state_scoped::StateScoped(GameScreen::BreedingResult),
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
                        BackgroundColor(colors::ACCENT_YELLOW),
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
                            TextColor(Color::BLACK),
                        ));
                    });
                });

            parent.spawn((
                Text::new("Resultat de la reproduction"),
                TextFont {
                    font_size: fonts::HEADING,
                    ..default()
                },
                TextColor(colors::ACCENT_MAGENTA),
                Node {
                    margin: UiRect::bottom(Val::Px(16.0)),
                    ..default()
                },
            ));

            // Afficher le bébé si disponible
            if let Some(ref baby) = data.breed_result {
                let secondary = baby
                    .secondary_type
                    .map(|t| format!("/{}", t))
                    .unwrap_or_default();

                parent
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            padding: UiRect::all(Val::Px(12.0)),
                            margin: UiRect::bottom(Val::Px(10.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(6.0),
                            ..default()
                        },
                        BackgroundColor(colors::PANEL),
                        BorderColor(colors::ACCENT_MAGENTA),
                        BorderRadius::all(Val::Px(8.0)),
                    ))
                    .with_children(|card| {
                        card.spawn((
                            Text::new(format!(
                                "[{}{}] {}  Nv.{}",
                                baby.primary_type, secondary, baby.name, baby.level,
                            )),
                            TextFont {
                                font_size: fonts::BODY,
                                ..default()
                            },
                            TextColor(colors::ACCENT_YELLOW),
                        ));

                        card.spawn((
                            Text::new(format!(
                                "PV {}  ATK {}  DEF {}  SPD {}",
                                baby.max_hp(),
                                baby.effective_attack(),
                                baby.effective_defense(),
                                baby.effective_speed(),
                            )),
                            TextFont {
                                font_size: fonts::SMALL,
                                ..default()
                            },
                            TextColor(colors::TEXT_SECONDARY),
                        ));

                        card.spawn((
                            Text::new(format!("Generation {}", baby.generation)),
                            TextFont {
                                font_size: fonts::SMALL,
                                ..default()
                            },
                            TextColor(colors::TEXT_SECONDARY),
                        ));
                    });
            } else {
                parent.spawn((
                    Text::new("Aucun résultat de reproduction disponible."),
                    TextFont {
                        font_size: fonts::BODY,
                        ..default()
                    },
                    TextColor(colors::TEXT_SECONDARY),
                ));
            }

            // Message
            if let Some(ref msg) = data.message {
                parent.spawn((
                    Text::new(msg.clone()),
                    TextFont {
                        font_size: fonts::BODY,
                        ..default()
                    },
                    TextColor(colors::ACCENT_GREEN),
                    Node {
                        margin: UiRect::top(Val::Px(12.0)),
                        ..default()
                    },
                ));
            }
        });
}

/// Gestion des entrées sur le résultat de reproduction.
pub(crate) fn handle_breeding_result_input(
    mut data: ResMut<GameData>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameScreen>>,
    interaction_query: Query<(&Interaction, &BackButton), Changed<Interaction>>,
    mut scroll_query: Query<&mut ScrollPosition, With<ScrollableContent>>,
) {
    for (interaction, _) in &interaction_query {
        if *interaction == Interaction::Pressed {
            go_back(&mut data, &mut next_state);
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
    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Escape) {
        go_back(&mut data, &mut next_state);
    }
}

fn go_back(data: &mut ResMut<GameData>, next_state: &mut ResMut<NextState<GameScreen>>) {
    data.breed_result = None;
    data.message = None;
    data.scroll_offset = 0;
    next_state.set(GameScreen::MainMenu);
    data.menu_index = 0;
}
