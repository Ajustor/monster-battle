//! Écran de combat interactif.

use bevy::prelude::*;
use bevy::state::state::NextState;

use crate::game::{GameData, GameScreen, ScreenEntity};
use crate::net_task::{NetTask, NetTaskAction};
use crate::sprites;
use crate::ui::common::{self, SAFE_BOTTOM, SAFE_TOP, colors, fonts};
use monster_battle_core::battle::{BattlePhase, BattleState, MessageStyle};
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

/// Construit l'UI de combat.
pub(crate) fn spawn_battle_ui(
    mut commands: Commands,
    data: Res<GameData>,
    mut images: ResMut<Assets<Image>>,
    mut atlas: ResMut<sprites::MonsterSpriteAtlas>,
) {
    let battle = match &data.battle_state {
        Some(b) => b,
        None => return,
    };
    spawn_battle_ui_inner(&mut commands, battle, &mut images, &mut atlas);
}

/// Logique interne de création de l'UI de combat (réutilisable).
fn spawn_battle_ui_inner(
    commands: &mut Commands,
    battle: &BattleState,
    mut images: &mut Assets<Image>,
    atlas: &mut sprites::MonsterSpriteAtlas,
) {
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
                    Val::Px(SAFE_TOP),
                    Val::Px(SAFE_BOTTOM),
                ),
                ..default()
            },
            BackgroundColor(colors::BACKGROUND),
            ScreenEntity,
            bevy::state::state_scoped::StateScoped(GameScreen::Battle),
        ))
        .with_children(|root| {
            // ── Zone adversaire (haut) ───────────────────────────
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(12.0),
                margin: UiRect::bottom(Val::Px(16.0)),
                ..default()
            })
            .with_children(|top| {
                // Sprite adversaire (de face)
                let grid = sprites::get_pixel_sprite(
                    battle.opponent.element,
                    battle.opponent.secondary_element,
                );
                let handle = atlas.get_or_create_front(
                    battle.opponent.element,
                    battle.opponent.secondary_element,
                    grid,
                    &mut images,
                );

                top.spawn((
                    ImageNode::new(handle),
                    Node {
                        width: Val::Px(96.0),
                        height: Val::Px(96.0),
                        ..default()
                    },
                ));

                // Stats adversaire
                top.spawn(Node {
                    flex_direction: FlexDirection::Column,
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

                    let hp_color =
                        common::hp_color(battle.opponent.current_hp, battle.opponent.max_hp);
                    info.spawn((
                        Text::new(format!(
                            "PV {}/{}",
                            battle.opponent.current_hp, battle.opponent.max_hp,
                        )),
                        TextFont {
                            font_size: fonts::SMALL,
                            ..default()
                        },
                        TextColor(hp_color),
                    ));
                });
            });

            // ── Zone joueur (bas) ────────────────────────────────
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(12.0),
                justify_content: JustifyContent::FlexEnd,
                ..default()
            })
            .with_children(|bottom| {
                // Stats joueur
                bottom
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexEnd,
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

                        let hp_color =
                            common::hp_color(battle.player.current_hp, battle.player.max_hp);
                        info.spawn((
                            Text::new(format!(
                                "PV {}/{}",
                                battle.player.current_hp, battle.player.max_hp,
                            )),
                            TextFont {
                                font_size: fonts::SMALL,
                                ..default()
                            },
                            TextColor(hp_color),
                        ));
                    });

                // Sprite joueur (de dos)
                let grid = sprites::get_pixel_back_sprite(
                    battle.player.element,
                    battle.player.secondary_element,
                );
                let handle = atlas.get_or_create_back(
                    battle.player.element,
                    battle.player.secondary_element,
                    grid,
                    &mut images,
                );

                bottom.spawn((
                    ImageNode::new(handle),
                    Node {
                        width: Val::Px(96.0),
                        height: Val::Px(96.0),
                        ..default()
                    },
                ));
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
                    actions.spawn((
                        Text::new(msg.text.clone()),
                        TextFont {
                            font_size: fonts::BODY,
                            ..default()
                        },
                        TextColor(colors::TEXT_PRIMARY),
                        Node {
                            margin: UiRect::bottom(Val::Px(8.0)),
                            ..default()
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
        });
}

/// Gestion des entrées en combat.
pub(crate) fn handle_battle_input(
    mut commands: Commands,
    mut data: ResMut<GameData>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameScreen>>,
    mut images: ResMut<Assets<Image>>,
    mut atlas: ResMut<sprites::MonsterSpriteAtlas>,
    net_task: Option<ResMut<NetTask>>,
    attack_query: Query<(&Interaction, &AttackButton), Changed<Interaction>>,
    flee_query: Query<&Interaction, (Changed<Interaction>, With<FleeButton>)>,
    continue_query: Query<&Interaction, (Changed<Interaction>, With<ContinueButton>)>,
    screen_entities: Query<Entity, With<ScreenEntity>>,
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
                                battle.player_attack(btn.index);
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
                                battle.player_attack(idx);
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
            // Supprimer l'ancienne UI
            for entity in &screen_entities {
                commands.entity(entity).despawn_recursive();
            }
            // Reconstruire avec l'état mis à jour
            if let Some(ref battle) = data.battle_state {
                spawn_battle_ui_inner(&mut commands, battle, &mut images, &mut atlas);
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
            if let Some(ref net) = net_task {
                if let Some(ref tx) = net.attack_tx {
                    let _ = tx.try_send(idx);
                }
            }
            // Passer en mode "attente du serveur"
            if let Some(ref mut battle) = data.battle_state {
                battle.phase = BattlePhase::Executing;
                battle.current_message =
                    Some(monster_battle_core::battle::BattleMessage {
                        text: "En attente de l'adversaire...".to_string(),
                        style: MessageStyle::Info,
                        player_hp: None,
                        opponent_hp: None,
                        anim_type: None,
                    });
                battle.message_counter += 1;
            }
            // Rebuild l'UI
            for entity in &screen_entities {
                commands.entity(entity).despawn_recursive();
            }
            if let Some(ref battle) = data.battle_state {
                spawn_battle_ui_inner(&mut commands, battle, &mut images, &mut atlas);
            }
        }
        Action::PvpSendReady => {
            // Envoyer PvpReady au serveur (sentinel usize::MAX - 1)
            if let Some(ref net) = net_task {
                if let Some(ref tx) = net.attack_tx {
                    let _ = tx.try_send(usize::MAX - 1);
                }
            }
            // Afficher un message d'attente
            if let Some(ref mut battle) = data.battle_state {
                battle.current_message =
                    Some(monster_battle_core::battle::BattleMessage {
                        text: "En attente de l'adversaire...".to_string(),
                        style: MessageStyle::Info,
                        player_hp: None,
                        opponent_hp: None,
                        anim_type: None,
                    });
                battle.message_counter += 1;
            }
            for entity in &screen_entities {
                commands.entity(entity).despawn_recursive();
            }
            if let Some(ref battle) = data.battle_state {
                spawn_battle_ui_inner(&mut commands, battle, &mut images, &mut atlas);
            }
        }
        Action::PvpForfeit => {
            // Envoyer le forfait au serveur (sentinel usize::MAX)
            if let Some(ref net) = net_task {
                if let Some(ref tx) = net.attack_tx {
                    let _ = tx.try_send(usize::MAX);
                }
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
    } else {
        fighter.losses += 1;
        if battle.loser_died {
            fighter.died_at = Some(chrono::Utc::now());
        } else {
            // Entraînement docile ou fuite : soigner le monstre
            fighter.current_hp = fighter.max_hp();
        }
    }

    let _ = data.storage.save(fighter);

    if is_victory {
        data.message = Some(format!(
            "🏆 Victoire ! +{} XP{}",
            battle.xp_gained,
            if battle.is_training {
                " (entraînement docile)"
            } else {
                ""
            }
        ));
    } else if !battle.loser_died {
        if battle.is_training {
            data.message = Some("Défaite à l'entraînement docile — pas de pénalité !".to_string());
        }
    } else {
        data.message = Some("💀 Défaite... Votre monstre est mort.".to_string());
    }
}
