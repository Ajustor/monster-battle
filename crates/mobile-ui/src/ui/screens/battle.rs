//! Écran de combat interactif.

use bevy::prelude::*;
use bevy::state::state::NextState;
use bevy::ui::widget::NodeImageMode;

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

/// Marqueur pour le nœud racine de l'écran de combat (utilisé pour le zoom).
#[derive(Component)]
pub(crate) struct BattleRootNode;


/// Ressource qui gère l'état global de l'animation de combat.
#[derive(Resource)]
pub(crate) struct BattleAnimTimer {
    /// Timer de l'animation en cours.
    pub timer: Timer,
    /// Type d'animation.
    pub anim: AnimationType,
}

#[derive(Component)]
pub(crate) struct AttackFlashOverlay {
    pub timer: Timer,
    /// Délai avant apparition du flash (laisse le temps aux particules de jouer).
    pub delay: Timer,
    pub started: bool,
    /// Alpha maximum du flash (0.35 pour hit normal, 0.7 pour critique).
    pub max_alpha: f32,
}

/// Filtre de couleur plein-écran déclenché par le type d'attaque (fade in → hold → fade out).
#[derive(Component)]
pub(crate) struct BattleColorFilter {
    pub timer: Timer,
    /// Couleur du filtre (alpha max inclus dans la couleur).
    pub color: Color,
}

/// Effet machine à écrire — révèle le texte du message caractère par caractère.
#[derive(Component)]
pub(crate) struct BattleTypewriter {
    /// Texte complet à révéler.
    pub full_text: String,
    /// Timer entre deux caractères.
    pub char_timer: Timer,
    /// Nombre de caractères actuellement affichés.
    pub displayed: usize,
    /// true quand tous les caractères sont visibles.
    pub done: bool,
}

/// Indicateur ▼ clignotant — affiché quand le message est entièrement révélé.
#[derive(Component)]
pub(crate) struct BattlePromptArrow {
    pub timer: Timer,
}

/// Animation d'entrée en combat : le sprite glisse depuis l'extérieur de l'écran.
#[derive(Component)]
pub(crate) struct BattleEntryAnim {
    pub timer: Timer,
    /// true = joueur (entre par le bas), false = adversaire (entre par le haut).
    pub is_player: bool,
}

/// Construit l'UI de combat.
pub(crate) fn spawn_battle_ui(
    mut commands: Commands,
    data: Res<GameData>,
    mut images: ResMut<Assets<Image>>,
    mut atlas: ResMut<sprites::MonsterSpriteAtlas>,
    asset_server: Res<AssetServer>,
    metrics: Res<ScreenMetrics>) {
    let battle = match &data.battle_state {
        Some(b) => b,
        None => return,
    };

    spawn_battle_ui_inner(&mut commands, battle, &mut images, &mut atlas, &asset_server, metrics.safe_top, metrics.safe_bottom, true);
}

/// Logique interne de création de l'UI de combat (réutilisable).
#[allow(clippy::too_many_arguments)]
fn spawn_battle_ui_inner(
    commands: &mut Commands,
    battle: &BattleState,
    images: &mut Assets<Image>,
    atlas: &mut sprites::MonsterSpriteAtlas,
    asset_server: &AssetServer,
    safe_top: f32,
    safe_bottom: f32,
    is_initial: bool,
) {
    let is_waiting = matches!(
        battle.phase,
        BattlePhase::Executing | BattlePhase::WaitingForOpponent
    ) && battle.current_message.is_none()
        && battle.message_queue.is_empty();

    // Fond de combat : entité root indépendante à z=-1, spawné une seule fois.
    if is_initial {
        spawn_battle_background(commands, battle.opponent.element, asset_server);
    }

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

            ScreenEntity,
            BattleRootNode,
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
                // Stats adversaire + barre PV (à gauche — encadré style Pokémon)
                top.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        flex_grow: 1.0,
                        padding: UiRect::all(Val::Px(8.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
                    BorderColor(Color::srgba(0.85, 0.85, 1.0, 0.3)),
                    BorderRadius::all(Val::Px(8.0)),
                ))
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
                // L'animation d'entrée ne joue qu'une seule fois, au tout premier spawn.
                let opp_entry_anim = !opponent_dead && is_initial;
                let mut opp_entry = top.spawn((
                    ImageNode::new(handle),
                    Node {
                        width: Val::Px(80.0),
                        height: if opponent_dead {
                            Val::Px(0.0)
                        } else {
                            Val::Px(80.0)
                        },
                        // Offset initial hors écran uniquement pour l'animation d'entrée
                        top: if opp_entry_anim { Val::Px(-120.0) } else { Val::Px(0.0) },
                        ..default()
                    },
                    if opponent_dead {
                        Visibility::Hidden
                    } else {
                        Visibility::Inherited
                    },
                    OpponentSprite,
                ));
                if opp_entry_anim {
                    opp_entry.insert(BattleEntryAnim {
                        timer: Timer::from_seconds(0.4, TimerMode::Once),
                        is_player: false,
                    });
                }
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
                // L'animation d'entrée ne joue qu'une seule fois, au tout premier spawn.
                let player_entry_anim = !player_dead && is_initial;
                let mut player_entry = bottom.spawn((
                    ImageNode::new(handle),
                    Node {
                        width: Val::Px(112.0),
                        height: if player_dead {
                            Val::Px(0.0)
                        } else {
                            Val::Px(112.0)
                        },
                        // Offset initial hors écran uniquement pour l'animation d'entrée
                        top: if player_entry_anim { Val::Px(140.0) } else { Val::Px(0.0) },
                        ..default()
                    },
                    if player_dead {
                        Visibility::Hidden
                    } else {
                        Visibility::Inherited
                    },
                    PlayerSprite,
                ));
                if player_entry_anim {
                    player_entry.insert(BattleEntryAnim {
                        timer: Timer::from_seconds(0.45, TimerMode::Once),
                        is_player: true,
                    });
                }

                // Stats joueur + barre PV (à droite — encadré style Pokémon)
                bottom
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::FlexEnd,
                            flex_grow: 1.0,
                            padding: UiRect::all(Val::Px(8.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
                        BorderColor(Color::srgba(0.85, 0.85, 1.0, 0.3)),
                        BorderRadius::all(Val::Px(8.0)),
                    ))
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

            // ── Zone actions / messages (bas) — style Pokémon ────
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.0),
                    row_gap: Val::Px(5.0),
                    ..default()
                },
            ))
            .with_children(|actions| {
                // === BOÎTE DE DIALOGUE style Pokémon ===
                // Visible quand il y a un message ou une attente PvP.
                let has_message = battle.current_message.is_some() || is_waiting;
                if has_message {
                    actions.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            width: Val::Percent(100.0),
                            min_height: Val::Px(82.0),
                            padding: UiRect::new(
                                Val::Px(14.0), Val::Px(14.0),
                                Val::Px(10.0), Val::Px(10.0),
                            ),
                            border: UiRect::all(Val::Px(3.0)),
                            position_type: PositionType::Relative,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.05, 0.05, 0.10)),
                        BorderColor(Color::srgb(0.80, 0.80, 0.90)),
                        BorderRadius::all(Val::Px(10.0)),
                    ))
                    .with_children(|msg_box| {
                        if let Some(ref msg) = battle.current_message {
                            let msg_color = match msg.style {
                                MessageStyle::PlayerAttack   => colors::ACCENT_BLUE,
                                MessageStyle::OpponentAttack => colors::ACCENT_RED,
                                MessageStyle::Victory        => colors::ACCENT_GREEN,
                                MessageStyle::Defeat         => colors::ACCENT_RED,
                                MessageStyle::Critical | MessageStyle::SuperEffective
                                    => colors::ACCENT_YELLOW,
                                _   => colors::TEXT_PRIMARY,
                            };
                            // Texte avec effet machine à écrire
                            msg_box.spawn((
                                Text::new(String::new()),
                                TextFont { font_size: fonts::BODY, ..default() },
                                TextColor(msg_color),
                                BattleTypewriter {
                                    full_text: msg.text.clone(),
                                    char_timer: Timer::from_seconds(0.033, TimerMode::Repeating),
                                    displayed: 0,
                                    done: false,
                                },
                            ));
                            // Indicateur ▼ clignotant en bas à droite
                            msg_box.spawn((
                                Text::new("▼"),
                                TextFont { font_size: 13.0, ..default() },
                                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.0)),
                                Node {
                                    position_type: PositionType::Absolute,
                                    right: Val::Px(10.0),
                                    bottom: Val::Px(8.0),
                                    ..default()
                                },
                                BattlePromptArrow {
                                    timer: Timer::from_seconds(0.55, TimerMode::Repeating),
                                },
                            ));
                        } else if is_waiting {
                            msg_box.spawn((
                                Text::new("En attente de l'adversaire".to_string()),
                                TextFont { font_size: fonts::BODY, ..default() },
                                TextColor(Color::srgba(0.7, 0.7, 0.7, 0.8)),
                                WaitingDots {
                                    timer: Timer::from_seconds(0.5, TimerMode::Repeating),
                                    dots: 0,
                                },
                            ));
                        }
                    });
                }

                // === BOUTONS ===
                match battle.phase {
                    BattlePhase::PlayerChooseAttack => {
                        // Boutons d'attaque avec badge de type coloré
                        for (i, attack) in battle.player.attacks.iter().enumerate() {
                            let selected = i == battle.attack_menu_index;
                            let (badge_bg, badge_txt) = element_badge_colors(attack.element);

                            actions
                                .spawn((
                                    Node {
                                        width: Val::Percent(100.0),
                                        flex_direction: FlexDirection::Row,
                                        align_items: AlignItems::Center,
                                        column_gap: Val::Px(10.0),
                                        padding: UiRect::axes(Val::Px(10.0), Val::Px(10.0)),
                                        border: UiRect::left(Val::Px(4.0)),
                                        ..default()
                                    },
                                    BackgroundColor(if selected {
                                        Color::srgb(0.18, 0.18, 0.28)
                                    } else {
                                        colors::PANEL
                                    }),
                                    BorderColor(badge_bg),
                                    BorderRadius::all(Val::Px(6.0)),
                                    AttackButton { index: i },
                                    Interaction::default(),
                                ))
                                .with_children(|btn| {
                                    // Badge type coloré
                                    btn.spawn((
                                        Node {
                                            padding: UiRect::axes(Val::Px(7.0), Val::Px(3.0)),
                                            ..default()
                                        },
                                        BackgroundColor(badge_bg),
                                        BorderRadius::all(Val::Px(4.0)),
                                    ))
                                    .with_children(|badge| {
                                        badge.spawn((
                                            Text::new(format!("{}", attack.element)),
                                            TextFont { font_size: 12.0, ..default() },
                                            TextColor(badge_txt),
                                        ));
                                    });

                                    // Nom de l'attaque (flex-grow)
                                    btn.spawn((
                                        Node { flex_grow: 1.0, ..default() },
                                    ))
                                    .with_children(|name_col| {
                                        name_col.spawn((
                                            Text::new(attack.name.clone()),
                                            TextFont { font_size: fonts::BODY, ..default() },
                                            TextColor(if selected {
                                                colors::ACCENT_YELLOW
                                            } else {
                                                colors::TEXT_PRIMARY
                                            }),
                                        ));
                                    });

                                    // Puissance à droite
                                    btn.spawn((
                                        Text::new(format!("Puiss. {}", attack.power)),
                                        TextFont { font_size: fonts::SMALL, ..default() },
                                        TextColor(colors::TEXT_SECONDARY),
                                    ));
                                });
                        }

                        // Bouton « Fuir » — discret
                        actions
                            .spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                                    margin: UiRect::top(Val::Px(2.0)),
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.96, 0.26, 0.21, 0.12)),
                                BorderRadius::all(Val::Px(6.0)),
                                FleeButton,
                                Interaction::default(),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Fuir"),
                                    TextFont { font_size: fonts::SMALL, ..default() },
                                    TextColor(colors::ACCENT_RED),
                                ));
                            });
                    }
                    BattlePhase::Victory if battle.is_over() => {
                        actions
                            .spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    padding: UiRect::axes(Val::Px(16.0), Val::Px(14.0)),
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
                                    TextFont { font_size: fonts::BODY, ..default() },
                                    TextColor(Color::BLACK),
                                ));
                            });
                    }
                    BattlePhase::Defeat if battle.is_over() => {
                        actions
                            .spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    padding: UiRect::axes(Val::Px(16.0), Val::Px(14.0)),
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
                                    Text::new("Défaite... Retour au menu"),
                                    TextFont { font_size: fonts::BODY, ..default() },
                                    TextColor(Color::WHITE),
                                ));
                            });
                    }
                    _ => {
                        // Bouton discret « Appuyer pour continuer »
                        actions
                            .spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
                                    justify_content: JustifyContent::Center,
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.12, 0.12, 0.20, 0.6)),
                                BorderColor(Color::srgba(0.5, 0.5, 0.6, 0.3)),
                                BorderRadius::all(Val::Px(6.0)),
                                ContinueButton,
                                Interaction::default(),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Appuyer pour continuer"),
                                    TextFont { font_size: fonts::SMALL, ..default() },
                                    TextColor(colors::TEXT_SECONDARY),
                                ));
                            });
                    }
                }
            });

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
    asset_server: Res<AssetServer>,
    net_task: Option<ResMut<NetTask>>,
    mut attack_effects: EventWriter<PlayAttackEffect>,
    attack_query: Query<(&Interaction, &AttackButton), Changed<Interaction>>,
    flee_query: Query<&Interaction, (Changed<Interaction>, With<FleeButton>)>,
    continue_query: Query<&Interaction, (Changed<Interaction>, With<ContinueButton>)>,
    screen_entities: Query<Entity, With<ScreenEntity>>,
    metrics: Res<ScreenMetrics>,
    anim_timer: Option<Res<BattleAnimTimer>>,
    mut typewriter_query: Query<(&mut Text, &mut BattleTypewriter)>,
) {
    // === Pré-vérification : machine à écrire ===
    // Si le joueur appuie et qu'un texte est en cours de révélation, on le complète
    // immédiatement plutôt que d'avancer au message suivant.
    {
        let any_pressed = keyboard.just_pressed(KeyCode::Enter)
            || keyboard.just_pressed(KeyCode::Space)
            || continue_query.iter().any(|i| *i == Interaction::Pressed);
        if any_pressed {
            let mut completed_any = false;
            for (mut text, mut tw) in &mut typewriter_query {
                if !tw.done {
                    tw.done = true;
                    *text = Text::new(tw.full_text.clone());
                    completed_any = true;
                }
            }
            if completed_any {
                return; // Le texte vient d'être complété — attendre le prochain tap
            }
        }
    }

    // === Bloquer les inputs pendant les animations de combat ===
    // (les animations doivent se terminer avant que le joueur puisse avancer)
    if anim_timer.is_some() {
        return;
    }
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
                                // Émettre l'effet de combat (particules) — délai = moment d'impact
                                attack_effects.send(PlayAttackEffect {
                                    element: attack_element,
                                    position: Vec2::new(0.65, 0.30), // Position adversaire (viewport %)
                                    startup_delay: 0.42,
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
                                    startup_delay: 0.42,
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
                        match anim {
                            AnimationType::PlayerAttack => {
                                // Filtre couleur de l'élément du joueur pendant l'attaque
                                let color = element_filter_color(battle.player.element);
                                commands.spawn((
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Percent(100.0),
                                        position_type: PositionType::Absolute,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                                    GlobalZIndex(40),
                                    BattleColorFilter {
                                        timer: Timer::from_seconds(duration, TimerMode::Once),
                                        color,
                                    },
                                ));
                            }
                            AnimationType::OpponentAttack => {
                                // Filtre couleur de l'élément adversaire + particules sur le joueur
                                let color = element_filter_color(battle.opponent.element);
                                commands.spawn((
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Percent(100.0),
                                        position_type: PositionType::Absolute,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                                    GlobalZIndex(40),
                                    BattleColorFilter {
                                        timer: Timer::from_seconds(duration, TimerMode::Once),
                                        color,
                                    },
                                ));
                                attack_effects.send(PlayAttackEffect {
                                    element: battle.opponent.element,
                                    position: Vec2::new(0.35, 0.65),
                                    startup_delay: 0.42,
                                });
                            }
                            AnimationType::PlayerHit | AnimationType::OpponentHit => {
                                // Flash léger sur hit normal
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
                                        timer: Timer::from_seconds(0.12, TimerMode::Once),
                                        delay: Timer::from_seconds(0.20, TimerMode::Once),
                                        started: false,
                                        max_alpha: 0.35,
                                    },
                                ));
                            }
                            AnimationType::PlayerHitCritical | AnimationType::OpponentHitCritical => {
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
                                        max_alpha: 0.7,
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
                spawn_battle_ui_inner(&mut commands, battle, &mut images, &mut atlas, &asset_server, metrics.safe_top, metrics.safe_bottom, false);
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
                spawn_battle_ui_inner(&mut commands, battle, &mut images, &mut atlas, &asset_server, metrics.safe_top, metrics.safe_bottom, false);
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
                spawn_battle_ui_inner(&mut commands, battle, &mut images, &mut atlas, &asset_server, metrics.safe_top, metrics.safe_bottom, false);
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
    asset_server: Res<AssetServer>,
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
        spawn_battle_ui_inner(&mut commands, battle, &mut images, &mut atlas, &asset_server, metrics.safe_top, metrics.safe_bottom, false);
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
            // Dash ample style Gen 5 (durée 0.9s) : charge-up → rush → impact → retour
            if let Ok((mut node, _)) = player_sprite.get_single_mut() {
                let (offset_y, offset_x) = if progress < 0.22 {
                    // Charge-up : recul vers le bas-gauche
                    let t = progress / 0.22;
                    let ease = t * t;
                    (8.0 * ease, -10.0 * ease)
                } else if progress < 0.47 {
                    // Rush très rapide vers l'adversaire (haut-droite)
                    let t = (progress - 0.22) / 0.25;
                    let ease = t * t; // ease-in accéléré
                    (8.0 - 65.0 * ease, -10.0 + 75.0 * ease)
                } else if progress < 0.57 {
                    // Pause à l'impact (plus longue avec 0.9s)
                    (-57.0, 65.0)
                } else {
                    // Retour fluide
                    let t = (progress - 0.57) / 0.43;
                    let ease = 1.0 - (1.0 - t) * (1.0 - t);
                    (-57.0 * (1.0 - ease), 65.0 * (1.0 - ease))
                };
                node.top = Val::Px(offset_y);
                node.left = Val::Px(offset_x);
            }
        }
        AnimationType::OpponentAttack => {
            // Dash ample de l'adversaire vers le joueur (miroir)
            if let Ok((mut node, _)) = opponent_sprite.get_single_mut() {
                let (offset_y, offset_x) = if progress < 0.22 {
                    let t = progress / 0.22;
                    let ease = t * t;
                    (-6.0 * ease, 8.0 * ease)
                } else if progress < 0.47 {
                    let t = (progress - 0.22) / 0.25;
                    let ease = t * t;
                    (-6.0 + 52.0 * ease, 8.0 - 60.0 * ease)
                } else if progress < 0.57 {
                    (46.0, -52.0)
                } else {
                    let t = (progress - 0.57) / 0.43;
                    let ease = 1.0 - (1.0 - t) * (1.0 - t);
                    (46.0 * (1.0 - ease), -52.0 * (1.0 - ease))
                };
                node.top = Val::Px(offset_y);
                node.left = Val::Px(offset_x);
            }
        }
        AnimationType::PlayerHit => {
            // Tremblement fort + clignotement rapide style Gen 5
            if let Ok((mut node, mut vis)) = player_sprite.get_single_mut() {
                // Shake latéral avec pic au début puis amplitude décroissante
                let shake_amp = 12.0 * (1.0 - progress).powf(0.6);
                let shake = (progress * 32.0 * std::f32::consts::PI).sin() * shake_amp;
                node.left = Val::Px(shake);

                // Clignotement rapide (5 cycles sur la durée)
                let blink_cycle = (progress * 22.0) as u32;
                *vis = if blink_cycle.is_multiple_of(2) {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }
        AnimationType::OpponentHit => {
            // Tremblement + clignotement (même logique côté adversaire)
            if let Ok((mut node, mut vis)) = opponent_sprite.get_single_mut() {
                let shake_amp = 12.0 * (1.0 - progress).powf(0.6);
                let shake = (progress * 32.0 * std::f32::consts::PI).sin() * shake_amp;
                node.left = Val::Px(shake);

                let blink_cycle = (progress * 22.0) as u32;
                *vis = if blink_cycle.is_multiple_of(2) {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }
        AnimationType::PlayerHitCritical => {
            // Coup critique : tremblement très fort + clignotement intense
            if let Ok((mut node, mut vis)) = player_sprite.get_single_mut() {
                let shake_amp = 18.0 * (1.0 - progress).powf(0.5);
                let shake = (progress * 40.0 * std::f32::consts::PI).sin() * shake_amp;
                node.left = Val::Px(shake);
                let blink_cycle = (progress * 28.0) as u32;
                *vis = if blink_cycle.is_multiple_of(2) { Visibility::Visible } else { Visibility::Hidden };
            }
        }
        AnimationType::OpponentHitCritical => {
            // Coup critique sur l'adversaire : tremblement très fort
            if let Ok((mut node, mut vis)) = opponent_sprite.get_single_mut() {
                let shake_amp = 18.0 * (1.0 - progress).powf(0.5);
                let shake = (progress * 40.0 * std::f32::consts::PI).sin() * shake_amp;
                node.left = Val::Px(shake);
                let blink_cycle = (progress * 28.0) as u32;
                *vis = if blink_cycle.is_multiple_of(2) { Visibility::Visible } else { Visibility::Hidden };
            }
        }
        AnimationType::PlayerFaint => {
            // K.O. style Gen 5 : le sprite tombe hors écran avec accélération + clignotement
            if let Ok((mut node, mut vis)) = player_sprite.get_single_mut() {
                let ease_in = progress * progress; // accélération gravitationnelle
                // Glisse vers le bas de manière accélérée
                node.top = Val::Px(160.0 * ease_in);
                // Clignotement à partir de 40% de l'animation
                if progress > 0.4 {
                    let blink_cycle = ((progress - 0.4) / 0.6 * 18.0) as u32;
                    *vis = if blink_cycle.is_multiple_of(2) { Visibility::Visible } else { Visibility::Hidden };
                }
                if progress > 0.95 {
                    *vis = Visibility::Hidden;
                }
            }
        }
        AnimationType::OpponentFaint => {
            // K.O. style Gen 5 : tombe hors écran avec accélération + clignotement
            if let Ok((mut node, mut vis)) = opponent_sprite.get_single_mut() {
                let ease_in = progress * progress;
                node.top = Val::Px(120.0 * ease_in);
                if progress > 0.4 {
                    let blink_cycle = ((progress - 0.4) / 0.6 * 18.0) as u32;
                    *vis = if blink_cycle.is_multiple_of(2) { Visibility::Visible } else { Visibility::Hidden };
                }
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
                    node.top = Val::Px(0.0);
                    node.left = Val::Px(0.0);
                    *vis = Visibility::Hidden;
                }
            }
            AnimationType::OpponentFaint => {
                if let Ok((mut node, mut vis)) = opponent_sprite.get_single_mut() {
                    node.height = Val::Px(0.0);
                    node.top = Val::Px(0.0);
                    node.left = Val::Px(0.0);
                    *vis = Visibility::Hidden;
                }
            }
            _ => {
                if let Ok((mut node, mut vis)) = player_sprite.get_single_mut() {
                    node.top = Val::Px(0.0);
                    node.left = Val::Px(0.0);
                    *vis = Visibility::Visible;
                }
                if let Ok((mut node, mut vis)) = opponent_sprite.get_single_mut() {
                    node.top = Val::Px(0.0);
                    node.left = Val::Px(0.0);
                    *vis = Visibility::Visible;
                }
            }
        }
        commands.remove_resource::<BattleAnimTimer>();
    }
}

/// Oscillation douce des sprites en attente d'une action (style Pokémon idle).
pub(crate) fn animate_idle_sprites(
    time: Res<Time>,
    battle_state: Option<Res<BattleAnimTimer>>,
    mut player_sprite: Query<&mut Node, With<PlayerSprite>>,
    mut opponent_sprite: Query<&mut Node, (With<OpponentSprite>, Without<PlayerSprite>)>,
    data: Res<GameData>,
) {
    // Pas d'oscillation pendant une animation en cours
    if battle_state.is_some() {
        return;
    }
    let Some(ref battle) = data.battle_state else {
        return;
    };
    if battle.phase != BattlePhase::PlayerChooseAttack {
        return;
    }

    let t = time.elapsed_secs();
    if let Ok(mut node) = player_sprite.get_single_mut() {
        // Bob plus prononcé style Gen 5 : amplitude 5px, rythme légèrement plus rapide
        node.top = Val::Px((t * 1.8 * std::f32::consts::TAU).sin() * 5.0);
    }
    if let Ok(mut node) = opponent_sprite.get_single_mut() {
        // Adversaire oscille en décalage de phase
        node.top = Val::Px((t * 1.5 * std::f32::consts::TAU + 1.2).sin() * 3.5);
    }
}

/// Zoom désactivé — garantit que UiScale et margin restent à leur valeur par défaut.
pub(crate) fn animate_attack_zoom(
    mut ui_scale: ResMut<UiScale>,
    mut root_query: Query<&mut Node, With<BattleRootNode>>,
) {
    if (ui_scale.0 - 1.0).abs() > 0.001 {
        ui_scale.0 = 1.0;
    }
    for mut node in &mut root_query {
        if node.margin != UiRect::ZERO {
            node.margin = UiRect::ZERO;
        }
    }
}

/// Fond de combat : entité racine indépendante à GlobalZIndex(-1) pour être
/// garantie derrière tous les nœuds UI (qui sont à z ≥ 0 par défaut).
fn spawn_battle_background(
    commands: &mut Commands,
    opponent_element: monster_battle_core::types::ElementType,
    asset_server: &AssetServer,
) {
    let (wall, ground) = battleback_assets(opponent_element);
    log::info!("spawn_battle_background: element={:?} wall={} ground={}", opponent_element, wall, ground);

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                right: Val::Px(0.0),
                bottom: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            GlobalZIndex(-1),
            bevy::state::state_scoped::StateScoped(GameScreen::Battle),
        ))
        .with_children(|bg| {
            // Mur (haut ~55%)
            bg.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(55.0),
                    ..default()
                },
                ImageNode::new(asset_server.load(wall)).with_mode(NodeImageMode::Stretch),
            ));
            // Sol (bas ~45%)
            bg.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(45.0),
                    ..default()
                },
                ImageNode::new(asset_server.load(ground)).with_mode(NodeImageMode::Stretch),
            ));
        });
}

/// Retourne (wall_path, ground_path) selon le type de l'adversaire.
fn battleback_assets(
    element: monster_battle_core::types::ElementType,
) -> (&'static str, &'static str) {
    use monster_battle_core::types::ElementType;
    match element {
        ElementType::Fire => (
            "battlebacks/walls/LavaCave.png",
            "battlebacks/grounds/Lava2.png",
        ),
        ElementType::Water => (
            "battlebacks/walls/Clouds.png",
            "battlebacks/grounds/Clouds.png",
        ),
        ElementType::Electric => (
            "battlebacks/walls/RockCave.png",
            "battlebacks/grounds/RockCave.png",
        ),
        ElementType::Earth => (
            "battlebacks/walls/RockCave.png",
            "battlebacks/grounds/RockCave.png",
        ),
        ElementType::Wind => (
            "battlebacks/walls/Clouds.png",
            "battlebacks/grounds/Clouds.png",
        ),
        ElementType::Shadow => (
            "battlebacks/walls/LavaCave.png",
            "battlebacks/grounds/RockCave.png",
        ),
        ElementType::Light => (
            "battlebacks/walls/Clouds.png",
            "battlebacks/grounds/Grassland.png",
        ),
        ElementType::Plant => (
            "battlebacks/walls/Forest.png",
            "battlebacks/grounds/GrassMaze.png",
        ),
        ElementType::Normal => (
            "battlebacks/walls/GrassMaze.png",
            "battlebacks/grounds/Grassland.png",
        ),
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
                // Apparition immédiate au max alpha
                *bg = BackgroundColor(Color::srgba(1.0, 1.0, 1.0, flash.max_alpha));
            }
        } else {
            // Phase flash : fade out
            flash.timer.tick(time.delta());
            let alpha = flash.max_alpha * (1.0 - flash.timer.fraction());
            *bg = BackgroundColor(Color::srgba(1.0, 1.0, 1.0, alpha));
            if flash.timer.just_finished() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

/// Animation d'entrée style Gen 5 : les sprites glissent depuis l'extérieur de l'écran.
/// Joueur : arrive par le bas (offset top positif → 0).
/// Adversaire : arrive par le haut (offset top négatif → 0).
pub(crate) fn animate_entry_sprites(
    mut commands: Commands,
    time: Res<Time>,
    mut sprites: Query<(Entity, &mut Node, &mut BattleEntryAnim)>,
) {
    for (entity, mut node, mut entry) in &mut sprites {
        entry.timer.tick(time.delta());
        let progress = entry.timer.fraction();
        // ease-out quadratique : démarre vite, ralentit à l'arrivée
        let ease = 1.0 - (1.0 - progress) * (1.0 - progress);

        if entry.is_player {
            // Joueur : entre par le bas (140px → 0px)
            node.top = Val::Px(140.0 * (1.0 - ease));
        } else {
            // Adversaire : entre par le haut (-120px → 0px)
            node.top = Val::Px(-120.0 * (1.0 - ease));
        }

        if entry.timer.just_finished() {
            node.top = Val::Px(0.0);
            commands.entity(entity).remove::<BattleEntryAnim>();
        }
    }
}

/// Couleurs du badge de type sur les boutons d'attaque : (fond, texte).
fn element_badge_colors(element: monster_battle_core::types::ElementType) -> (Color, Color) {
    use monster_battle_core::types::ElementType;
    match element {
        ElementType::Fire     => (Color::srgb(0.85, 0.22, 0.05), Color::WHITE),
        ElementType::Water    => (Color::srgb(0.10, 0.42, 0.88), Color::WHITE),
        ElementType::Electric => (Color::srgb(0.95, 0.82, 0.05), Color::srgb(0.1, 0.1, 0.1)),
        ElementType::Earth    => (Color::srgb(0.55, 0.32, 0.08), Color::WHITE),
        ElementType::Wind     => (Color::srgb(0.20, 0.72, 0.60), Color::srgb(0.1, 0.1, 0.1)),
        ElementType::Shadow   => (Color::srgb(0.32, 0.05, 0.62), Color::WHITE),
        ElementType::Light    => (Color::srgb(0.92, 0.88, 0.35), Color::srgb(0.1, 0.1, 0.1)),
        ElementType::Plant    => (Color::srgb(0.12, 0.62, 0.12), Color::WHITE),
        ElementType::Normal   => (Color::srgb(0.40, 0.40, 0.48), Color::WHITE),
    }
}

/// Anime le texte caractère par caractère (effet machine à écrire style Pokémon).
pub(crate) fn animate_typewriter(
    time: Res<Time>,
    mut query: Query<(&mut Text, &mut BattleTypewriter)>,
) {
    for (mut text, mut tw) in &mut query {
        if tw.done {
            continue;
        }
        let total = tw.full_text.chars().count();
        if tw.displayed >= total {
            tw.done = true;
            *text = Text::new(tw.full_text.clone());
            continue;
        }
        tw.char_timer.tick(time.delta());
        let ticks = tw.char_timer.times_finished_this_tick() as usize;
        if ticks > 0 {
            tw.displayed = (tw.displayed + ticks).min(total);
            let revealed: String = tw.full_text.chars().take(tw.displayed).collect();
            *text = Text::new(revealed);
            if tw.displayed >= total {
                tw.done = true;
            }
        }
    }
}

/// Anime l'indicateur ▼ : visible et clignotant quand le message est entièrement révélé.
pub(crate) fn animate_prompt_arrow(
    time: Res<Time>,
    typewriter_query: Query<&BattleTypewriter>,
    mut arrow_query: Query<(&mut TextColor, &mut BattlePromptArrow)>,
) {
    // L'indicateur n'est visible que si TOUS les typewriters sont terminés
    let all_done = typewriter_query.iter().all(|tw| tw.done);

    for (mut color, mut arrow) in &mut arrow_query {
        if !all_done {
            *color = TextColor(Color::srgba(1.0, 1.0, 1.0, 0.0));
            continue;
        }
        arrow.timer.tick(time.delta());
        // Clignotement doux (sinusoïde : 0.0 → 1.0 → 0.0)
        let alpha = ((arrow.timer.fraction() * std::f32::consts::PI).sin()).max(0.0);
        *color = TextColor(Color::srgba(1.0, 1.0, 1.0, alpha));
    }
}

/// Retourne la couleur de filtre associée à un type élémentaire.
fn element_filter_color(element: monster_battle_core::types::ElementType) -> Color {
    use monster_battle_core::types::ElementType;
    match element {
        ElementType::Fire     => Color::srgba(1.00, 0.25, 0.00, 0.22),
        ElementType::Water    => Color::srgba(0.00, 0.35, 1.00, 0.22),
        ElementType::Electric => Color::srgba(1.00, 0.90, 0.00, 0.20),
        ElementType::Earth    => Color::srgba(0.60, 0.35, 0.05, 0.22),
        ElementType::Wind     => Color::srgba(0.25, 0.85, 0.65, 0.18),
        ElementType::Shadow   => Color::srgba(0.35, 0.00, 0.70, 0.25),
        ElementType::Light    => Color::srgba(1.00, 0.95, 0.55, 0.18),
        ElementType::Plant    => Color::srgba(0.05, 0.70, 0.10, 0.20),
        ElementType::Normal   => Color::srgba(0.55, 0.55, 0.55, 0.15),
    }
}

/// Anime le filtre de couleur plein-écran déclenché par le type d'attaque.
/// Enveloppe : fade-in rapide (0–20%) → maintien (20–70%) → fade-out (70–100%).
pub(crate) fn animate_color_filter(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut BattleColorFilter, &mut BackgroundColor)>,
) {
    for (entity, mut filter, mut bg) in &mut query {
        filter.timer.tick(time.delta());
        let p = filter.timer.fraction();

        // Extraire la couleur de base (RGBA) pour moduler l'alpha
        let base = filter.color.to_srgba();
        let alpha = if p < 0.20 {
            // Fade-in
            base.alpha * (p / 0.20)
        } else if p < 0.70 {
            // Maintien au max
            base.alpha
        } else {
            // Fade-out
            base.alpha * (1.0 - (p - 0.70) / 0.30)
        };

        *bg = BackgroundColor(Color::srgba(base.red, base.green, base.blue, alpha));

        if filter.timer.just_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
