//! Écrans de recherche PvP (searching / matched / error).

use bevy::prelude::*;
use bevy::state::state::NextState;

use crate::game::{GameData, GameScreen, ScreenEntity};
use crate::ui::common::{SAFE_BOTTOM, SAFE_TOP, colors, fonts};

// ═══════════════════════════════════════════════════════════════════
//  PvP Searching
// ═══════════════════════════════════════════════════════════════════

/// Construit l'UI de recherche PvP.
pub(crate) fn spawn_pvp_searching(mut commands: Commands) {
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
            bevy::state::state_scoped::StateScoped(GameScreen::PvpSearching),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Combat PvP"),
                TextFont {
                    font_size: fonts::TITLE,
                    ..default()
                },
                TextColor(colors::ACCENT_RED),
                Node {
                    margin: UiRect::bottom(Val::Px(24.0)),
                    ..default()
                },
            ));

            parent.spawn((
                Text::new("Recherche d'un adversaire..."),
                TextFont {
                    font_size: fonts::BODY,
                    ..default()
                },
                TextColor(colors::ACCENT_YELLOW),
                Node {
                    margin: UiRect::bottom(Val::Px(16.0)),
                    ..default()
                },
            ));

            parent.spawn((
                Text::new(
                    "Le combat commencera automatiquement\n\
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

/// Marqueur du bouton d'annulation.
#[derive(Component)]
pub(crate) struct CancelButton;

/// Gestion des entrées en recherche PvP.
pub(crate) fn handle_pvp_searching_input(
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
            next_state.set(GameScreen::MainMenu);
            data.menu_index = 0;
            return;
        }
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        commands.remove_resource::<crate::net_task::NetTask>();
        data.message = None;
        next_state.set(GameScreen::MainMenu);
        data.menu_index = 0;
    }
}
