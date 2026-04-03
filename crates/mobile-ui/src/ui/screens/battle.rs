//! Écran de combat interactif.

use bevy::prelude::*;
use bevy::state::state::NextState;

use crate::battle_effects::PlayAttackEffect;
use crate::game::{GameData, GameScreen, ScreenEntity};
use crate::net_task::{NetTask, NetTaskAction};
use crate::sprites;
use crate::ui::common::{self, colors, fonts, ScreenMetrics};
use monster_battle_core::battle::{AnimationType, BattlePhase, BattleState, MessageStyle};
use monster_battle_storage::MonsterStorage;

/// Marqueur pour les boutons d'attaque.
#[derive(Component)]
pub(crate) struct AttackButton {
    index: usize,
}

/// Marqueur pour le bouton « Fuir ».
#[derive(Component)]
pub(crate) struct FleeButton;

/// Marqueur pour le bouton « Continuer » / « Retour au menu ».
#[derive(Component)]
pub(crate) struct ContinueButton;

/// Marqueur pour le sprite du joueur.
#[derive(Component)]
pub(crate) struct PlayerSprite;

/// Marqueur pour le sprite de l'adversaire.
#[derive(Component)]
pub(crate) struct OpponentSprite;

/// Marqueur pour la barre de PV du joueur.
#[derive(Component)]
pub(crate) struct PlayerHpBar;

/// Marqueur pour la barre de PV de l'adversaire.
#[derive(Component)]
pub(crate) struct OpponentHpBar;

/// Marqueur pour le texte de PV du joueur.
#[derive(Component)]
pub(crate) struct PlayerHpText;

/// Marqueur pour le texte de PV de l'adversaire.
#[derive(Component)]
pub(crate) struct OpponentHpText;

/// Marqueur pour le texte « En attente... » avec animation de points.
#[derive(Component)]
pub(crate) struct WaitingDots {
    timer: Timer,
    dots: u8,
}

/// Ressource qui gère l'état global de l'animation de combat.
#[derive(Resource)]
pub(crate) struct BattleAnimTimer {
    /// Timer de l'animation en cours.
    pub timer: Timer,
    /// Type d'animation.
    pub anim: AnimationType,
}

/// Marqueur pour le flash d'impact (overlay plein écran temporaire).
/// Uniquement pour les coups critiques, déclenché après les particules.
/// Conteneur overlay pour les effets d'attaque — spawné dans l'écran de combat.
/// Les effets sont ajoutés comme enfants pour garantir le rendu sur Android.
#[derive(Component)]
pub struct BattleEffectsContainer;

#[derive(Component)]
pub(crate) struct AttackFlashOverlay {
    pub timer: Timer,
    /// Délai avant apparition du flash (laisse le temps aux particules de jouer).
    pub delay: Timer,
    pub started: bool,
}

/// Construit l'UI de combat.
pub(crate) fn spawn_battle_ui(
    mut commands: Commands,
    data: Res<GameData>,
    mut images: ResMut<Assets<Image>>,
    mut atlas: ResMut<sprites::MonsterSpriteAtlas>,
    metrics: Res<ScreenMetrics>) {
    let battle = match &data.battle_state {
        Some(b) => b,
        None => return,
    };
    spawn_battle_ui_inner(&mut commands, battle, &mut images, &mut atlas, metrics.safe_top, metrics.safe_bottom);
}

/// Logique interne de création de l'UI de combat (réutilisable).
fn spawn_battle_ui_inner(
    commands: &mut Commands,
    battle: &BattleState,
    images: &mut Assets<Image>,
    atlas: &mut sprites::MonsterSpriteAtlas,
    safe_top: f32,
    safe_bottom: f32,
) {
    let is_waiting = matches!(
        battle.phase,
        BattlePhase::Executing | BattlePhase::WaitingForOpponent
    ) && battle.current_message.is_none()
        && battle.message_queue.is_empty();

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
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
            bevy::state::state_scoped::StateScoped(GameScreen::Battle),
        ))
        .with_children(|root| {
            // ── Zone adversaire (haut-droite, style Pokémon) ─────
            // Info (nom + barre PV) à gauche, sprite à droite
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(12.0),
                margin: UiRect::bottom(Val::Px(16.0)),
                ..default()
            })
            .with_children(|top| {
                // Stats adversaire + barre PV (à gauche)
                top.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    ..default()
                })
                .with_children(|info| {
                    info.spawn((
                        Text::new(format!(
                            "{} Nv.{}",
                            battle.opponent.name, battle.opponent.level,
                        )),
                        TextFont {
                            font_size: fonts::BODY,
                            ..default()
                        },
                        TextColor(colors::TEXT_PRIMARY),
                    ));

                    // Barre de PV graphique
                    let hp_pct = if battle.opponent.max_hp > 0 {
                        (battle.opponent.display_hp as f32 / battle.opponent.max_hp as f32)
                            .clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    let hp_color =
                        common::hp_color(battle.opponent.display_hp, battle.opponent.max_hp);

                    info.spawn(Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(10.0),
                        margin: UiRect::vertical(Val::Px(4.0)),
                        ..default()
                    })
                    .with_children(|bar_bg| {
                        // Fond gris
                        bar_bg.spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                position_type: PositionType::Absolute,
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 0.5)),
                            BorderRadius::all(Val::Px(5.0)),
                        ));
                        // Barre colorée
                        bar_bg.spawn((
                            Node {
                                width: Val::Percent(hp_pct * 100.0),
                                height: Val::Percent(100.0),
                                position_type: PositionType::Absolute,
                                ..default()
                            },
                            BackgroundColor(hp_color),
                            BorderRadius::all(Val::Px(5.0)),
                            OpponentHpBar,
                        ));
                    });

                    info.spawn((
                        Text::new(format!(
                            "PV {}/{}",
                            battle.opponent.display_hp, battle.opponent.max_hp,
                        )),
                        TextFont {
                            font_size: fonts::SMALL,
                            ..default()
                        },
                        TextColor(hp_color),
                        OpponentHpText,
                    ));
                });

                // Sprite adversaire (de face, à droite — style Pokémon)
                let grid = sprites::get_blended_sprite(
                    battle.opponent.element,
                    battle.opponent.secondary_element,
                    battle.opponent.age_stage,
                );
                let handle = atlas.get_or_create_front(
                    battle.opponent.element,
                    battle.opponent.secondary_element,
                    battle.opponent.age_stage,
                    &grid,
                    images,
                );

                // Le sprite reste visible tant que l'animation de K.O. n'a pas été
                // déclenchée (message courant ou encore dans la file).
                let faint_still_pending = battle.opponent.current_hp == 0
                    && (battle.current_message.as_ref().is_some_and(|m| {
                        matches!(m.anim_type, Some(AnimationType::OpponentFaint))
                    }) || battle
                        .message_queue
                        .iter()
                        .any(|m| matches!(m.anim_type, Some(AnimationType::OpponentFaint))));
                let opponent_dead = battle.opponent.current_hp == 0 && !faint_still_pending;
                top.spawn((
                    ImageNode::new(handle),
                    Node {
                        width: Val::Px(80.0),
                        height: if opponent_dead {
                            Val::Px(0.0)
                        } else {
                            Val::Px(80.0)
                        },
                        ..default()
                    },
                    if opponent_dead {
                        Visibility::Hidden
                    } else {
                        Visibility::Inherited
                    },
                    OpponentSprite,
                ));
            });

            // ── Zone joueur (bas-gauche, style Pokémon) ──────────
            // Sprite à gauche (plus grand, de dos), info à droite
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(12.0),
                ..default()
            })
            .with_children(|bottom| {
                // Sprite joueur (de dos, à gauche — style Pokémon, plus grand)
                let grid = sprites::get_blended_back_sprite(
                    battle.player.element,
                    battle.player.secondary_element,
                    battle.player.age_stage,
                );
                let handle = atlas.get_or_create_back(
                    battle.player.element,
                    battle.player.secondary_element,
                    battle.player.age_stage,
                    &grid,
                    images,
                );

                // Même logique : garder le sprite visible jusqu'à ce que
                // l'animation de K.O. ait été consommée.
                let faint_still_pending = battle.player.current_hp == 0
                    && (battle.current_message.as_ref().is_some_and(|m| {
                        matches!(m.anim_type, Some(AnimationType::PlayerFaint))
                    }) || battle
                        .message_queue
                        .iter()
                        .any(|m| matches!(m.anim_type, Some(AnimationType::PlayerFaint))));
                let player_dead = battle.player.current_hp == 0 && !faint_still_pending;
                bottom.spawn((
                    ImageNode::new(handle),
                    Node {
                        width: Val::Px(112.0),
                        height: if player_dead {
                            Val::Px(0.0)
                        } else {
                            Val::Px(112.0)
                        },
                        ..default()
                    },
                    if player_dead {
                        Visibility::Hidden
                    } else {
                        Visibility::Inherited
                    },
                    PlayerSprite,
                ));

                // Stats joueur + barre PV (à droite)
                bottom
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexEnd,
                        flex_grow: 1.0,
                        ..default()
                    })
                    .with_children(|info| {
                        info.spawn((
                            Text::new(
                                format!("{} Nv.{}", battle.player.name, battle.player.level,),
                            ),
                            TextFont {
                                font_size: fonts::BODY,
                                ..default()
                            },
                            TextColor(colors::TEXT_PRIMARY),
                        ));

                        // Barre de PV graphique
                        let hp_pct = if battle.player.max_hp > 0 {
                            (battle.player.display_hp as f32 / battle.player.max_hp as f32)
                                .clamp(0.0, 1.0)
                        } else {
                            0.0
                        };
                        let hp_color =
                            common::hp_color(battle.player.display_hp, battle.player.max_hp);

                        info.spawn(Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(10.0),
                            margin: UiRect::vertical(Val::Px(4.0)),
                            ..default()
                        })
                        .with_children(|bar_bg| {
                            bar_bg.spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    position_type: PositionType::Absolute,
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 0.5)),
                                BorderRadius::all(Val::Px(5.0)),
                            ));
                            bar_bg.spawn((
                                Node {
                                    width: Val::Percent(hp_pct * 100.0),
                                    height: Val::Percent(100.0),
                                    position_type: PositionType::Absolute,
                                    ..default()
                                },
                                BackgroundColor(hp_color),
                                BorderRadius::all(Val::Px(5.0)),
                                PlayerHpBar,
                            ));
                        });

                        info.spawn((
                            Text::new(format!(
                                "PV {}/{}",
                                battle.player.display_hp, battle.player.max_hp,
                            )),
                            TextFont {
                                font_size: fonts::SMALL,
                                ..default()
                            },
                            TextColor(hp_color),
                            PlayerHpText,
                        ));
                    });
            });

            // ── Zone actions / messages (bas) ────────────────────
            root.spawn(Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            })
            .with_children(|actions| {
                // Message courant
                if let Some(ref msg) = battle.current_message {
                    let msg_color = match msg.style {
                        MessageStyle::PlayerAttack => colors::ACCENT_BLUE,
                        MessageStyle::OpponentAttack => colors::ACCENT_RED,
                        MessageStyle::Victory => colors::ACCENT_GREEN,
                        MessageStyle::Defeat => colors::ACCENT_RED,
                        _ => colors::TEXT_PRIMARY,
                    };
                    actions.spawn((
                        Text::new(msg.text.clone()),
                        TextFont {
                            font_size: fonts::BODY,
                            ..default()
                        },
                        TextColor(msg_color),
                        Node {
                            margin: UiRect::bottom(Val::Px(8.0)),
                            ..default()
                        },
                    ));
                } else if is_waiting {
                    // Texte animé « En attente de l'adversaire... »
                    actions.spawn((
                        Text::new("En attente de l'adversaire".to_string()),
                        TextFont {
                            font_size: fonts::BODY,
                            ..default()
                        },
                        TextColor(Color::srgba(0.7, 0.7, 0.7, 0.8)),
                        Node {
                            margin: UiRect::bottom(Val::Px(8.0)),
                            ..default()
                        },
                        WaitingDots {
                            timer: Timer::from_seconds(0.5, TimerMode::Repeating),
                            dots: 0,
                        },
                    ));
                }

                match battle.phase {
                    BattlePhase::PlayerChooseAttack => {
                        // Boutons d'attaque
                        for (i, attack) in battle.player.attacks.iter().enumerate() {
                            let selected = i == battle.attack_menu_index;
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

                            actions
                                .spawn((
                                    Node {
                                        width: Val::Percent(100.0),
                                        padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
                                        margin: UiRect::bottom(Val::Px(4.0)),
                                        ..default()
                                    },
                                    BackgroundColor(bg),
                                    BorderRadius::all(Val::Px(6.0)),
                                    AttackButton { index: i },
                                    Interaction::default(),
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new(format!(
                                            "[{}] {}  (Puissance: {})",
                                            attack.element, attack.name, attack.power,
                                        )),
                                        TextFont {
                                            font_size: fonts::BODY,
                                            ..default()
                                        },
                                        TextColor(txt_color),
                                    ));
                                });
                        }

                        // Bouton « Fuir »
                        actions
                            .spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
                                    margin: UiRect::top(Val::Px(8.0)),
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                BackgroundColor(colors::ACCENT_RED),
                                BorderRadius::all(Val::Px(6.0)),
                                FleeButton,
                                Interaction::default(),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Fuir le combat"),
                                    TextFont {
                                        font_size: fonts::BODY,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            });
                    }
                    BattlePhase::Victory if battle.is_over() => {
                        // Bouton retour après victoire (tous les messages affichés)
                        actions
                            .spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    padding: UiRect::axes(Val::Px(16.0), Val::Px(14.0)),
                                    margin: UiRect::top(Val::Px(8.0)),
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                BackgroundColor(colors::ACCENT_GREEN),
                                BorderRadius::all(Val::Px(8.0)),
                                ContinueButton,
                                Interaction::default(),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Victoire ! Retour au menu"),
                                    TextFont {
                                        font_size: fonts::BODY,
                                        ..default()
                                    },
                                    TextColor(Color::BLACK),
                                ));
                            });
                    }
                    BattlePhase::Defeat if battle.is_over() => {
                        // Bouton retour après défaite (tous les messages affichés)
                        actions
                            .spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    padding: UiRect::axes(Val::Px(16.0), Val::Px(14.0)),
                                    margin: UiRect::top(Val::Px(8.0)),
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                BackgroundColor(colors::ACCENT_RED),
                                BorderRadius::all(Val::Px(8.0)),
                                ContinueButton,
                                Interaction::default(),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Defaite... Retour au menu"),
                                    TextFont {
                                        font_size: fonts::BODY,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            });
                    }
                    _ => {
                        // Autres phases : bouton « Continuer » pour avancer les messages
                        actions
                            .spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    padding: UiRect::axes(Val::Px(16.0), Val::Px(12.0)),
                                    margin: UiRect::top(Val::Px(8.0)),
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                BackgroundColor(colors::PANEL),
                                BorderRadius::all(Val::Px(6.0)),
                                ContinueButton,
                                Interaction::default(),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Continuer..."),
                                    TextFont {
                                        font_size: fonts::BODY,
                                        ..default()
                                    },
                                    TextColor(colors::TEXT_PRIMARY),
                                ));
                            });
                    }
                }
            });

            // Conteneur overlay pour les effets d'attaque (position absolute, plein écran)
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BattleEffectsContainer,
            ));
        });
}

/// Gestion des entrées en combat.
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_battle_input(
    mut commands: Commands,
    mut data: ResMut<GameData>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameScreen>>,
    mut images: ResMut<Assets<Image>>,
    mut atlas: ResMut<sprites::MonsterSpriteAtlas>,
    net_task: Option<ResMut<NetTask>>,
    mut attack_effects: EventWriter<PlayAttackEffect>,
    attack_query: Query<(&Interaction, &AttackButton), Changed<Interaction>>,
    flee_query: Query<&Interaction, (Changed<Interaction>, With<FleeButton>)>,
    continue_query: Query<&Interaction, (Changed<Interaction>, With<ContinueButton>)>,
    screen_entities: Query<Entity, With<ScreenEntity>>,
    metrics: Res<ScreenMetrics>,
) {
    let is_pvp = net_task
        .as_ref()
        .map(|t| t.action == NetTaskAction::Pvp)
        .unwrap_or(false);

    // ── Résultat des interactions ──────────────────────────────────
    enum Action {
        None,
        Rebuild,
        EndBattle,
        Flee,
        PvpSendAttack(usize),
        PvpSendReady,
        PvpForfeit,
    }

    let action = {
        let battle = match data.battle_state.as_mut() {
            Some(b) => b,
            None => {
                next_state.set(GameScreen::MainMenu);
                return;
            }
        };

        if battle.is_over() {
            // Combat terminé — tout appui renvoie au menu
            let mut pressed = false;
            for interaction in &continue_query {
                if *interaction == Interaction::Pressed {
                    pressed = true;
                    break;
                }
            }
            if pressed || keyboard.just_pressed(KeyCode::Enter) {
                Action::EndBattle
            } else {
                Action::None
            }
        } else {
            match battle.phase {
                BattlePhase::PlayerChooseAttack => {
                    let attack_count = battle.player.attacks.len();
                    let mut act = Action::None;

                    // Toucher bouton attaque (mobile)
                    for (interaction, btn) in &attack_query {
                        if *interaction == Interaction::Pressed {
                            if is_pvp {
                                act = Action::PvpSendAttack(btn.index);
                            } else {
                                // Capture l'élément de l'attaque avant de l'exécuter
                                let attack_element = battle.player.attacks[btn.index].element;
                                battle.player_attack(btn.index);
                                // Émettre l'effet de combat (particules)
                                attack_effects.send(PlayAttackEffect {
                                    element: attack_element,
                                    position: Vec2::new(0.65, 0.30), // Position adversaire (viewport %)
                                });
                                act = Action::Rebuild;
                            }
                            break;
                        }
                    }

                    // Toucher bouton fuir (mobile)
                    if matches!(act, Action::None) {
                        for interaction in &flee_query {
                            if *interaction == Interaction::Pressed {
                                act = if is_pvp {
                                    Action::PvpForfeit
                                } else {
                                    Action::Flee
                                };
                                break;
                            }
                        }
                    }

                    // Clavier
                    if matches!(act, Action::None) {
                        if keyboard.just_pressed(KeyCode::ArrowUp) && battle.attack_menu_index > 0 {
                            battle.attack_menu_index -= 1;
                            act = Action::Rebuild;
                        }
                        if keyboard.just_pressed(KeyCode::ArrowDown)
                            && battle.attack_menu_index < attack_count.saturating_sub(1)
                        {
                            battle.attack_menu_index += 1;
                            act = Action::Rebuild;
                        }
                        if keyboard.just_pressed(KeyCode::Enter) {
                            let idx = battle.attack_menu_index;
                            if is_pvp {
                                act = Action::PvpSendAttack(idx);
                            } else {
                                // Capture l'élément avant l'attaque
                                let attack_element = battle.player.attacks[idx].element;
                                battle.player_attack(idx);
                                attack_effects.send(PlayAttackEffect {
                                    element: attack_element,
                                    position: Vec2::new(0.65, 0.30), // Position adversaire (viewport %)
                                });
                                act = Action::Rebuild;
                            }
                        }
                        if keyboard.just_pressed(KeyCode::Escape) {
                            act = if is_pvp {
                                Action::PvpForfeit
                            } else {
                                Action::Flee
                            };
                        }
                    }

                    act
                }
                BattlePhase::WaitingForOpponent => {
                    // PvP : le joueur a fini de lire les messages du tour
                    let mut act = Action::None;

                    for interaction in &continue_query {
                        if *interaction == Interaction::Pressed {
                            if !battle.advance_message() && battle.message_queue.is_empty() {
                                act = Action::PvpSendReady;
                            } else {
                                act = Action::Rebuild;
                            }
                            break;
                        }
                    }

                    if matches!(act, Action::None) {
                        if keyboard.just_pressed(KeyCode::Enter)
                            || keyboard.just_pressed(KeyCode::Space)
                        {
                            if !battle.advance_message() && battle.message_queue.is_empty() {
                                act = Action::PvpSendReady;
                            } else {
                                act = Action::Rebuild;
                            }
                        }
                        if keyboard.just_pressed(KeyCode::Escape) && is_pvp {
                            act = Action::PvpForfeit;
                        }
                    }

                    act
                }
                _ => {
                    // Intro, Executing, Victory/Defeat avec messages restants
                    let mut act = Action::None;

                    // En PvP, ne pas avancer si on attend la réponse du serveur
                    // (phase Executing + plus de messages = on attend)
                    let is_waiting_server = is_pvp
                        && battle.phase == BattlePhase::Executing
                        && battle.message_queue.is_empty()
                        && battle.current_message.is_none();

                    if is_waiting_server {
                        // Ne rien faire — la réponse viendra via poll_network_events
                        return;
                    }

                    // Toucher « Continuer » (mobile)
                    for interaction in &continue_query {
                        if *interaction == Interaction::Pressed {
                            battle.advance_message();
                            act = Action::Rebuild;
                            break;
                        }
                    }

                    // Clavier
                    if matches!(act, Action::None) {
                        if keyboard.just_pressed(KeyCode::Enter)
                            || keyboard.just_pressed(KeyCode::Space)
                        {
                            battle.advance_message();
                            act = Action::Rebuild;
                        }
                        if keyboard.just_pressed(KeyCode::Escape) {
                            act = if is_pvp {
                                Action::PvpForfeit
                            } else {
                                Action::Flee
                            };
                        }
                    }

                    act
                }
            }
        }
    };

    // ── Exécuter l'action ──────────────────────────────────────────
    match action {
        Action::None => {}
        Action::Rebuild => {
            // Déclencher une animation si le message courant en contient une
            if let Some(ref battle) = data.battle_state
                && let Some(ref msg) = battle.current_message
                    && let Some(ref anim) = msg.anim_type {
                        let duration = anim.duration();
                        commands.insert_resource(BattleAnimTimer {
                            timer: Timer::from_seconds(duration, TimerMode::Once),
                            anim: anim.clone(),
                        });
                        // Flash blanc uniquement sur coup critique, après les particules
                        match anim {
                            AnimationType::PlayerHitCritical | AnimationType::OpponentHitCritical => {
                                // Durée des particules ≈ 0.36s (3 frames × 0.12s × 2 loops)
                                // On déclenche le flash après ce délai
                                commands.spawn((
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Percent(100.0),
                                        position_type: PositionType::Absolute,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.0)),
                                    GlobalZIndex(100),
                                    AttackFlashOverlay {
                                        timer: Timer::from_seconds(0.18, TimerMode::Once),
                                        delay: Timer::from_seconds(0.36, TimerMode::Once),
                                        started: false,
                                    },
                                ));
                            }
                            _ => {}
                        }
                    }
            // Supprimer l'ancienne UI
            for entity in &screen_entities {
                commands.entity(entity).despawn_recursive();
            }
            // Reconstruire avec l'état mis à jour
            if let Some(ref battle) = data.battle_state {
                spawn_battle_ui_inner(&mut commands, battle, &mut images, &mut atlas, metrics.safe_top, metrics.safe_bottom);
            }
        }
        Action::EndBattle => {
            commands.remove_resource::<NetTask>();
            apply_battle_results(&mut data);
            next_state.set(GameScreen::MainMenu);
        }
        Action::Flee => {
            data.battle_state = None;
            data.message = Some("Vous avez fui le combat.".to_string());
            next_state.set(GameScreen::MainMenu);
        }
        Action::PvpSendAttack(idx) => {
            // Envoyer le choix au serveur via le canal
            if let Some(ref net) = net_task
                && let Some(ref tx) = net.attack_tx {
                    let _ = tx.try_send(idx);
                }
            // Passer en mode "attente du serveur"
            if let Some(ref mut battle) = data.battle_state {
                battle.phase = BattlePhase::Executing;
                battle.current_message = None;
                battle.message_queue.clear();
            }
            // Rebuild l'UI
            for entity in &screen_entities {
                commands.entity(entity).despawn_recursive();
            }
            if let Some(ref battle) = data.battle_state {
                spawn_battle_ui_inner(&mut commands, battle, &mut images, &mut atlas, metrics.safe_top, metrics.safe_bottom);
            }
        }
        Action::PvpSendReady => {
            // Envoyer PvpReady au serveur (sentinel usize::MAX - 1)
            if let Some(ref net) = net_task
                && let Some(ref tx) = net.attack_tx {
                    let _ = tx.try_send(usize::MAX - 1);
                }
            // Nettoyer le message et attendre PvpNextTurn
            if let Some(ref mut battle) = data.battle_state {
                battle.current_message = None;
                battle.message_queue.clear();
            }
            for entity in &screen_entities {
                commands.entity(entity).despawn_recursive();
            }
            if let Some(ref battle) = data.battle_state {
                spawn_battle_ui_inner(&mut commands, battle, &mut images, &mut atlas, metrics.safe_top, metrics.safe_bottom);
            }
        }
        Action::PvpForfeit => {
            // Envoyer le forfait au serveur (sentinel usize::MAX)
            if let Some(ref net) = net_task
                && let Some(ref tx) = net.attack_tx {
                    let _ = tx.try_send(usize::MAX);
                }
            commands.remove_resource::<NetTask>();
            data.battle_state = None;
            data.message = Some("Vous avez abandonné le combat PvP.".to_string());
            next_state.set(GameScreen::MainMenu);
        }
    }
}

/// Applique les résultats du combat (XP, victoire/défaite, mort éventuelle).
fn apply_battle_results(data: &mut GameData) {
    let battle = match data.battle_state.take() {
        Some(b) => b,
        None => return,
    };

    let mut monsters = match data.storage.list_alive() {
        Ok(m) if !m.is_empty() => m,
        _ => return,
    };

    let idx = data.monster_select_index;
    let fighter = if let Some(f) = monsters.get_mut(idx) {
        f
    } else if let Some(f) = monsters.first_mut() {
        f
    } else {
        return;
    };

    let is_victory = battle.phase == BattlePhase::Victory;

    if is_victory {
        fighter.wins += 1;
        fighter.gain_xp(battle.xp_gained);
        fighter.current_hp = fighter.max_hp();
        // Victoire → bonheur + lien
        fighter.adjust_happiness(10);
        fighter.record_interaction();
        fighter.increase_bond(2);
    } else {
        fighter.losses += 1;
        // Défaite → perte de bonheur
        fighter.adjust_happiness(-5);
        if battle.loser_died {
            fighter.died_at = Some(chrono::Utc::now());
        } else {
            // Entraînement docile ou fuite : soigner le monstre
            fighter.current_hp = fighter.max_hp();
        }
    }

    // Dévoration de l'adversaire (uniquement en combat réel, pas en entraînement)
    let devour_msg = if is_victory && !battle.is_training {
        if let Some(ref prey) = battle.opponent_data {
            match fighter.try_devour(prey) {
                Some(result) => Some(result.description),
                None => Some(format!(
                    "{} est rassasié et refuse de dévorer le vaincu.",
                    fighter.name
                )),
            }
        } else {
            None
        }
    } else {
        None
    };

    let _ = data.storage.save(fighter);

    if is_victory {
        let xp_msg = format!(
            "🏆 Victoire ! +{} XP{}",
            battle.xp_gained,
            if battle.is_training {
                " (entraînement docile)"
            } else {
                ""
            }
        );
        data.message = Some(match devour_msg {
            Some(d) => format!("{}\n{}", xp_msg, d),
            None => xp_msg,
        });
    } else if !battle.loser_died {
        if battle.is_training {
            data.message = Some("Défaite à l'entraînement docile — pas de pénalité !".to_string());
        }
    } else {
        data.message = Some("💀 Défaite... Votre monstre est mort.".to_string());
    }
}

// ═══════════════════════════════════════════════════════════════════
//  Systèmes d'animation de combat
// ═══════════════════════════════════════════════════════════════════

/// Système qui reconstruit l'UI de combat quand `battle_ui_dirty` est posé
/// (par polling réseau, animations, etc.).
pub(crate) fn refresh_battle_ui(
    mut commands: Commands,
    mut data: ResMut<GameData>,
    mut images: ResMut<Assets<Image>>,
    mut atlas: ResMut<sprites::MonsterSpriteAtlas>,
    screen_entities: Query<Entity, With<ScreenEntity>>,
    metrics: Res<ScreenMetrics>,
) {
    if !data.battle_ui_dirty {
        return;
    }
    data.battle_ui_dirty = false;

    // Supprimer l'ancienne UI
    for entity in &screen_entities {
        commands.entity(entity).despawn_recursive();
    }

    // Reconstruire
    if let Some(ref battle) = data.battle_state {
        spawn_battle_ui_inner(&mut commands, battle, &mut images, &mut atlas, metrics.safe_top, metrics.safe_bottom);
    }
}

/// Anime les barres de PV et le texte associé (transition fluide vers la valeur cible).
#[allow(clippy::type_complexity)]
pub(crate) fn animate_hp_bars(
    mut data: ResMut<GameData>,
    time: Res<Time>,
    mut player_bar: Query<
        (&mut Node, &mut BackgroundColor),
        (With<PlayerHpBar>, Without<OpponentHpBar>),
    >,
    mut opponent_bar: Query<
        (&mut Node, &mut BackgroundColor),
        (With<OpponentHpBar>, Without<PlayerHpBar>),
    >,
    mut player_text: Query<
        (&mut Text, &mut TextColor),
        (With<PlayerHpText>, Without<OpponentHpText>),
    >,
    mut opponent_text: Query<
        (&mut Text, &mut TextColor),
        (With<OpponentHpText>, Without<PlayerHpText>),
    >,
) {
    let battle = match data.battle_state.as_mut() {
        Some(b) => b,
        None => return,
    };

    let dt = time.delta_secs();
    let speed = 60.0; // PV par seconde

    let mut changed = false;

    // Animer display_hp du joueur vers player_target_hp
    if battle.player.display_hp != battle.player_target_hp {
        let target = battle.player_target_hp as f32;
        let current = battle.player.display_hp as f32;
        let new_hp = if current > target {
            (current - speed * dt).max(target)
        } else {
            (current + speed * dt).min(target)
        };
        battle.player.display_hp = new_hp.round() as u32;
        changed = true;
    }

    // Animer display_hp de l'adversaire vers opponent_target_hp
    if battle.opponent.display_hp != battle.opponent_target_hp {
        let target = battle.opponent_target_hp as f32;
        let current = battle.opponent.display_hp as f32;
        let new_hp = if current > target {
            (current - speed * dt).max(target)
        } else {
            (current + speed * dt).min(target)
        };
        battle.opponent.display_hp = new_hp.round() as u32;
        changed = true;
    }

    if !changed {
        return;
    }

    // Mettre à jour la barre du joueur
    if let Ok((mut node, mut bg)) = player_bar.get_single_mut() {
        let pct = if battle.player.max_hp > 0 {
            (battle.player.display_hp as f32 / battle.player.max_hp as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };
        node.width = Val::Percent(pct * 100.0);
        *bg = BackgroundColor(common::hp_color(
            battle.player.display_hp,
            battle.player.max_hp,
        ));
    }
    if let Ok((mut text, mut color)) = player_text.get_single_mut() {
        *text = Text::new(format!(
            "PV {}/{}",
            battle.player.display_hp, battle.player.max_hp
        ));
        *color = TextColor(common::hp_color(
            battle.player.display_hp,
            battle.player.max_hp,
        ));
    }

    // Mettre à jour la barre de l'adversaire
    if let Ok((mut node, mut bg)) = opponent_bar.get_single_mut() {
        let pct = if battle.opponent.max_hp > 0 {
            (battle.opponent.display_hp as f32 / battle.opponent.max_hp as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };
        node.width = Val::Percent(pct * 100.0);
        *bg = BackgroundColor(common::hp_color(
            battle.opponent.display_hp,
            battle.opponent.max_hp,
        ));
    }
    if let Ok((mut text, mut color)) = opponent_text.get_single_mut() {
        *text = Text::new(format!(
            "PV {}/{}",
            battle.opponent.display_hp, battle.opponent.max_hp
        ));
        *color = TextColor(common::hp_color(
            battle.opponent.display_hp,
            battle.opponent.max_hp,
        ));
    }
}

/// Anime le texte « En attente de l'adversaire... » avec des points qui pulsent.
pub(crate) fn animate_waiting_dots(
    time: Res<Time>,
    mut query: Query<(&mut Text, &mut WaitingDots)>,
) {
    for (mut text, mut dots) in &mut query {
        dots.timer.tick(time.delta());
        if dots.timer.just_finished() {
            dots.dots = (dots.dots + 1) % 4;
            let suffix = ".".repeat(dots.dots as usize);
            *text = Text::new(format!("En attente de l'adversaire{}", suffix));
        }
    }
}

/// Anime les sprites en cas d'attaque ou de hit (style Pokémon).
#[allow(clippy::type_complexity)]
pub(crate) fn animate_battle_sprites(
    mut commands: Commands,
    time: Res<Time>,
    anim: Option<ResMut<BattleAnimTimer>>,
    mut player_sprite: Query<
        (&mut Node, &mut Visibility),
        (With<PlayerSprite>, Without<OpponentSprite>),
    >,
    mut opponent_sprite: Query<
        (&mut Node, &mut Visibility),
        (With<OpponentSprite>, Without<PlayerSprite>),
    >,
) {
    let mut anim = match anim {
        Some(a) => a,
        None => return,
    };

    anim.timer.tick(time.delta());
    let progress = anim.timer.fraction();

    match anim.anim {
        AnimationType::PlayerAttack => {
            // Lunge style Pokémon : dash rapide vers l'adversaire puis retour
            if let Ok((mut node, _)) = player_sprite.get_single_mut() {
                let (offset_y, offset_x) = if progress < 0.3 {
                    // Phase d'élan : recul léger
                    let t = progress / 0.3;
                    (5.0 * t, -3.0 * t)
                } else if progress < 0.5 {
                    // Phase d'attaque : dash vers l'adversaire (haut-droite)
                    let t = (progress - 0.3) / 0.2;
                    let ease = t * t; // ease-in
                    (5.0 - 40.0 * ease, -3.0 + 20.0 * ease)
                } else if progress < 0.55 {
                    // Pause à l'impact
                    (-35.0, 17.0)
                } else {
                    // Retour à la position d'origine
                    let t = (progress - 0.55) / 0.45;
                    let ease = 1.0 - (1.0 - t) * (1.0 - t); // ease-out
                    (-35.0 * (1.0 - ease), 17.0 * (1.0 - ease))
                };
                node.top = Val::Px(offset_y);
                node.left = Val::Px(offset_x);
            }
        }
        AnimationType::OpponentAttack => {
            // Lunge style Pokémon : dash rapide vers le joueur puis retour
            if let Ok((mut node, _)) = opponent_sprite.get_single_mut() {
                let (offset_y, offset_x) = if progress < 0.3 {
                    let t = progress / 0.3;
                    (-5.0 * t, 3.0 * t)
                } else if progress < 0.5 {
                    let t = (progress - 0.3) / 0.2;
                    let ease = t * t;
                    (-5.0 + 40.0 * ease, 3.0 - 20.0 * ease)
                } else if progress < 0.55 {
                    (35.0, -17.0)
                } else {
                    let t = (progress - 0.55) / 0.45;
                    let ease = 1.0 - (1.0 - t) * (1.0 - t);
                    (35.0 * (1.0 - ease), -17.0 * (1.0 - ease))
                };
                node.top = Val::Px(offset_y);
                node.left = Val::Px(offset_x);
            }
        }
        AnimationType::PlayerHit => {
            // Clignotement rapide + tremblement (style Pokémon : blink + shake)
            if let Ok((mut node, mut vis)) = player_sprite.get_single_mut() {
                // Tremblement avec amplitude décroissante
                let shake_amp = 8.0 * (1.0 - progress);
                let shake = (progress * 30.0 * std::f32::consts::PI).sin() * shake_amp;
                node.left = Val::Px(shake);

                // Clignotement rapide (toggle visible/invisible)
                let blink_cycle = (progress * 20.0) as u32;
                *vis = if blink_cycle.is_multiple_of(2) {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }
        AnimationType::OpponentHit => {
            // Clignotement rapide + tremblement (style Pokémon)
            if let Ok((mut node, mut vis)) = opponent_sprite.get_single_mut() {
                let shake_amp = 8.0 * (1.0 - progress);
                let shake = (progress * 30.0 * std::f32::consts::PI).sin() * shake_amp;
                node.left = Val::Px(shake);

                let blink_cycle = (progress * 20.0) as u32;
                *vis = if blink_cycle.is_multiple_of(2) {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }
        AnimationType::PlayerHitCritical => {
            // Coup critique : même tremblement que PlayerHit mais plus fort
            if let Ok((mut node, mut vis)) = player_sprite.get_single_mut() {
                let shake_amp = 14.0 * (1.0 - progress);
                let shake = (progress * 35.0 * std::f32::consts::PI).sin() * shake_amp;
                node.left = Val::Px(shake);
                let blink_cycle = (progress * 24.0) as u32;
                *vis = if blink_cycle.is_multiple_of(2) { Visibility::Visible } else { Visibility::Hidden };
            }
        }
        AnimationType::OpponentHitCritical => {
            // Coup critique sur l'adversaire : tremblement amplifié
            if let Ok((mut node, mut vis)) = opponent_sprite.get_single_mut() {
                let shake_amp = 14.0 * (1.0 - progress);
                let shake = (progress * 35.0 * std::f32::consts::PI).sin() * shake_amp;
                node.left = Val::Px(shake);
                let blink_cycle = (progress * 24.0) as u32;
                *vis = if blink_cycle.is_multiple_of(2) { Visibility::Visible } else { Visibility::Hidden };
            }
        }
        AnimationType::PlayerFaint => {
            // K.O. style Pokémon : le sprite rétrécit vers le bas et disparaît
            if let Ok((mut node, mut vis)) = player_sprite.get_single_mut() {
                let shrink = 1.0 - progress;
                node.height = Val::Px(112.0 * shrink);
                // Glisser vers le bas en rétrécissant
                node.top = Val::Px(112.0 * progress * 0.5);
                if progress > 0.95 {
                    *vis = Visibility::Hidden;
                }
            }
        }
        AnimationType::OpponentFaint => {
            // K.O. style Pokémon : le sprite rétrécit vers le bas et disparaît
            if let Ok((mut node, mut vis)) = opponent_sprite.get_single_mut() {
                let shrink = 1.0 - progress;
                node.height = Val::Px(80.0 * shrink);
                node.top = Val::Px(80.0 * progress * 0.5);
                if progress > 0.95 {
                    *vis = Visibility::Hidden;
                }
            }
        }
    }

    if anim.timer.just_finished() {
        // Remettre les sprites en place (sauf pour les faint où on les laisse cachés)
        match anim.anim {
            AnimationType::PlayerFaint => {
                if let Ok((mut node, mut vis)) = player_sprite.get_single_mut() {
                    node.height = Val::Px(0.0);
                    node.top = Val::Auto;
                    node.left = Val::Auto;
                    *vis = Visibility::Hidden;
                }
            }
            AnimationType::OpponentFaint => {
                if let Ok((mut node, mut vis)) = opponent_sprite.get_single_mut() {
                    node.height = Val::Px(0.0);
                    node.top = Val::Auto;
                    node.left = Val::Auto;
                    *vis = Visibility::Hidden;
                }
            }
            _ => {
                if let Ok((mut node, mut vis)) = player_sprite.get_single_mut() {
                    node.top = Val::Auto;
                    node.left = Val::Auto;
                    *vis = Visibility::Visible;
                }
                if let Ok((mut node, mut vis)) = opponent_sprite.get_single_mut() {
                    node.top = Val::Auto;
                    node.left = Val::Auto;
                    *vis = Visibility::Visible;
                }
            }
        }
        commands.remove_resource::<BattleAnimTimer>();
    }
}

/// Anime le flash d'impact critique (overlay blanc, déclenché après les particules).
pub(crate) fn animate_attack_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut flash_query: Query<(Entity, &mut AttackFlashOverlay, &mut BackgroundColor)>,
) {
    for (entity, mut flash, mut bg) in &mut flash_query {
        if !flash.started {
            // Phase délai : attendre la fin des particules
            flash.delay.tick(time.delta());
            if flash.delay.just_finished() {
                flash.started = true;
                // Apparition immédiate du flash
                *bg = BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.7));
            }
        } else {
            // Phase flash : fade out
            flash.timer.tick(time.delta());
            let alpha = 0.7 * (1.0 - flash.timer.fraction());
            *bg = BackgroundColor(Color::srgba(1.0, 1.0, 1.0, alpha));
            if flash.timer.just_finished() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
