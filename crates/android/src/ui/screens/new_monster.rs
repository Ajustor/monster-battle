//! Écran de création d'un monstre — choix du type élémentaire.

use bevy::prelude::*;
use bevy::state::state::NextState;

use monster_battle_core::types::ElementType;

use crate::game::{GameData, GameScreen, ScreenEntity};
use crate::ui::common::{SAFE_TOP, colors, fonts};

/// Marqueur pour les boutons de type.
#[derive(Component)]
pub(crate) struct TypeButton {
    index: usize,
}

/// Marqueur pour le bouton retour.
#[derive(Component)]
pub(crate) struct NewMonsterBackButton;

/// Construit l'UI de sélection du type de starter.
pub(crate) fn spawn_new_monster(mut commands: Commands, data: Res<GameData>) {
    let types = ElementType::all();

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
                    Val::Px(16.0),
                ),
                ..default()
            },
            BackgroundColor(colors::BACKGROUND),
            ScreenEntity,
            bevy::state::state_scoped::StateScoped(GameScreen::NewMonster),
        ))
        .with_children(|parent| {
            // Titre
            parent.spawn((
                Text::new("Choisir un type de starter"),
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
                            Text::new(format!("{}", t)),
                            TextFont {
                                font_size: fonts::BODY,
                                ..default()
                            },
                            TextColor(txt_color),
                        ));
                    });
            }

            // Bouton retour (tactile)
            parent
                .spawn((
                    Node {
                        padding: UiRect::axes(Val::Px(24.0), Val::Px(12.0)),
                        margin: UiRect::top(Val::Px(16.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(colors::PANEL),
                    BorderRadius::all(Val::Px(8.0)),
                    NewMonsterBackButton,
                    Interaction::default(),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("< Retour"),
                        TextFont {
                            font_size: fonts::BODY,
                            ..default()
                        },
                        TextColor(colors::TEXT_PRIMARY),
                    ));
                });
        });
}

/// Gestion des entrées sur l'écran de choix du type.
pub(crate) fn handle_new_monster_input(
    mut data: ResMut<GameData>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameScreen>>,
    interaction_query: Query<(&Interaction, &TypeButton), Changed<Interaction>>,
    back_query: Query<&Interaction, (Changed<Interaction>, With<NewMonsterBackButton>)>,
) {
    let types = ElementType::all();
    let type_count = types.len();

    // Toucher retour
    for interaction in &back_query {
        if *interaction == Interaction::Pressed {
            next_state.set(GameScreen::MainMenu);
            data.menu_index = 0;
            return;
        }
    }

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
