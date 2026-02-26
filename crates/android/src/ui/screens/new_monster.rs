//! Écran de création d'un monstre — choix du type élémentaire.

use bevy::prelude::*;
use bevy::state::state::NextState;

use monster_battle_core::types::ElementType;

use crate::game::{GameData, GameScreen, ScreenEntity};
use crate::ui::common::{colors, fonts};

/// Marqueur pour les boutons de type.
#[derive(Component)]
pub(crate) struct TypeButton {
    index: usize,
}

/// Construit l'UI de sélection du type de starter.
pub(crate) fn spawn_new_monster(mut commands: Commands, data: Res<GameData>) {
    let types = ElementType::all();

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            },
            BackgroundColor(colors::BACKGROUND),
            ScreenEntity,
        ))
        .with_children(|parent| {
            // Titre
            parent.spawn((
                Text::new("🥚 Choisir un type de starter"),
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

            // Liste des types
            for (i, t) in types.iter().enumerate() {
                let selected = i == data.type_choice_index % types.len();
                let bg = if selected {
                    colors::ACCENT_YELLOW
                } else {
                    colors::PANEL
                };
                let txt_color = if selected {
                    Color::BLACK
                } else {
                    colors::TEXT_PRIMARY
                };

                parent
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            padding: UiRect::axes(Val::Px(20.0), Val::Px(12.0)),
                            margin: UiRect::bottom(Val::Px(6.0)),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(bg),
                        BorderRadius::all(Val::Px(8.0)),
                        TypeButton { index: i },
                        Interaction::default(),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(format!("{} {}", t.icon(), t)),
                            TextFont {
                                font_size: fonts::BODY,
                                ..default()
                            },
                            TextColor(txt_color),
                        ));
                    });
            }

            // Pied de page
            parent.spawn((
                Text::new("↑↓ Naviguer  ⏎ Confirmer  Esc Retour"),
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

/// Gestion des entrées sur l'écran de choix du type.
pub(crate) fn handle_new_monster_input(
    mut data: ResMut<GameData>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameScreen>>,
    interaction_query: Query<(&Interaction, &TypeButton), Changed<Interaction>>,
) {
    let types = ElementType::all();
    let type_count = types.len();

    // Toucher (mobile)
    for (interaction, button) in &interaction_query {
        if *interaction == Interaction::Pressed {
            data.type_choice_index = button.index;
            data.name_input.clear();
            next_state.set(GameScreen::NamingMonster);
            return;
        }
    }

    // Clavier
    if keyboard.just_pressed(KeyCode::ArrowUp) && data.type_choice_index > 0 {
        data.type_choice_index -= 1;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) && data.type_choice_index < type_count - 1 {
        data.type_choice_index += 1;
    }
    if keyboard.just_pressed(KeyCode::Enter) {
        data.name_input.clear();
        next_state.set(GameScreen::NamingMonster);
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameScreen::MainMenu);
        data.menu_index = 0;
    }
}
