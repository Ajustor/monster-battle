//! Plugin d'animations d'effets de combat.
//! Les effets sont des nœuds UI root avec position absolute + TargetCamera.

use bevy::prelude::*;
use monster_battle_core::types::ElementType;


pub struct BattleEffectsPlugin;

impl Plugin for BattleEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayAttackEffect>().add_systems(
            Update,
            (
                handle_play_attack_effect,
                animate_attack_effects,
                cleanup_finished_effects,
            ),
        );
    }
}

#[derive(Component)]
pub struct AttackEffect {
    pub frame_timer: Timer,
    pub frame_count: usize,
    pub current_frame: usize,
    pub lifetime: Timer,
    /// Délai avant que l'effet devienne visible (synchronisé avec le moment d'impact).
    pub startup_delay: Timer,
    /// true une fois que le délai de démarrage est écoulé.
    pub started: bool,
    /// true une fois que toutes les frames ont été jouées (reste sur la dernière).
    pub sequence_done: bool,
}

/// Événement : jouer un effet d'attaque sur la cible.
#[derive(Event)]
pub struct PlayAttackEffect {
    pub element: ElementType,
    /// `true` = cibler le sprite adversaire, `false` = cibler le sprite joueur.
    pub target_opponent: bool,
    /// Secondes d'attente avant d'afficher l'effet (correspond au moment d'impact).
    pub startup_delay: f32,
}

#[derive(Component)]
pub struct AttackFrames {
    pub handles: Vec<Handle<Image>>,
}


fn tint_for_element(element: ElementType) -> Color {
    match element {
        ElementType::Fire     => Color::srgba(1.0, 0.4, 0.05, 1.0),
        ElementType::Water    => Color::srgba(0.2, 0.6, 1.0,  1.0),
        ElementType::Electric => Color::srgba(1.0, 0.95, 0.1, 1.0),
        ElementType::Earth    => Color::srgba(0.75, 0.5, 0.15, 1.0),
        ElementType::Wind     => Color::srgba(0.5, 0.95, 0.75, 1.0),
        ElementType::Shadow   => Color::srgba(0.55, 0.1, 0.85, 1.0),
        ElementType::Light    => Color::srgba(1.0,  1.0, 0.75, 1.0),
        ElementType::Plant    => Color::srgba(0.2, 0.85, 0.2,  1.0),
        ElementType::Normal   => Color::srgba(0.75, 0.75, 0.75, 1.0),
    }
}

/// Durée d'affichage de chaque frame (secondes).
const FRAME_DURATION: f32 = 0.11;
/// Durée de maintien sur la dernière frame avant disparition.
const HOLD_DURATION: f32 = 0.06;
/// Taille de l'effet en pixels.
const EFFECT_SIZE: f32 = 180.0;

pub fn handle_play_attack_effect(
    mut commands: Commands,
    battle_images: Res<crate::battle_images::BattleImages>,
    mut events: EventReader<PlayAttackEffect>,
    player_sprite: Query<Entity, With<crate::ui::screens::battle::PlayerSprite>>,
    opponent_sprite: Query<Entity, With<crate::ui::screens::battle::OpponentSprite>>,
) {
    for event in events.read() {
        let sprite_entity = if event.target_opponent {
            opponent_sprite.get_single().ok()
        } else {
            player_sprite.get_single().ok()
        };
        let Some(sprite_entity) = sprite_entity else { continue };

        let frames = battle_images.effect_frames(event.element);
        let frame_count = frames.len();
        let tint = tint_for_element(event.element);

        let first_image = frames[0].clone();
        let size = EFFECT_SIZE;
        let delay = event.startup_delay;
        let lifetime = delay + frame_count as f32 * FRAME_DURATION + HOLD_DURATION;

        let effect = commands.spawn((
            AttackEffect {
                frame_timer:    Timer::from_seconds(FRAME_DURATION, TimerMode::Repeating),
                frame_count,
                current_frame:  0,
                lifetime:       Timer::from_seconds(lifetime, TimerMode::Once),
                startup_delay:  Timer::from_seconds(delay.max(0.001), TimerMode::Once),
                started:        delay <= 0.0,
                sequence_done:  false,
            },
            AttackFrames {
                handles: frames.to_vec(),
            },
            Node {
                position_type: PositionType::Absolute,
                left:   Val::Percent(50.0),
                top:    Val::Percent(50.0),
                width:  Val::Px(size),
                height: Val::Px(size),
                margin: UiRect {
                    left:   Val::Px(-size / 2.0),
                    top:    Val::Px(-size / 2.0),
                    right:  Val::Auto,
                    bottom: Val::Auto,
                },
                ..default()
            },
            ImageNode {
                image: first_image,
                color: tint,
                ..default()
            },
            GlobalZIndex(50),
            if delay > 0.0 { Visibility::Hidden } else { Visibility::Visible },
        )).id();
        commands.entity(sprite_entity).add_child(effect);
    }
}

pub fn animate_attack_effects(
    time: Res<Time>,
    mut query: Query<(&mut AttackEffect, &AttackFrames, &mut ImageNode, &mut Visibility, &mut Node)>,
) {
    for (mut effect, frames, mut img_node, mut vis, mut node) in query.iter_mut() {
        // Phase de démarrage différé
        if !effect.started {
            effect.startup_delay.tick(time.delta());
            if effect.startup_delay.just_finished() {
                effect.started = true;
                *vis = Visibility::Visible;
            } else {
                continue;
            }
        }

        if effect.sequence_done {
            // Reste sur la dernière frame jusqu'à la fin du lifetime
            continue;
        }

        effect.frame_timer.tick(time.delta());

        // Animation de scale : l'effet grossit rapidement puis se stabilise
        // Basé sur la progression dans la séquence (0.0 → 1.0)
        let sequence_progress = (effect.current_frame as f32 + effect.frame_timer.fraction())
            / effect.frame_count as f32;
        let scale = if sequence_progress < 0.3 {
            // Expansion rapide depuis 0.4x jusqu'à 1.1x
            let t = sequence_progress / 0.3;
            0.4 + 0.7 * t
        } else if sequence_progress < 0.6 {
            // Légère compression (rebond) : 1.1x → 1.0x
            let t = (sequence_progress - 0.3) / 0.3;
            1.1 - 0.1 * t
        } else {
            1.0
        };
        let scaled_size = EFFECT_SIZE * scale;
        node.width = Val::Px(scaled_size);
        node.height = Val::Px(scaled_size);
        node.margin = UiRect {
            left:   Val::Px(-scaled_size / 2.0),
            top:    Val::Px(-scaled_size / 2.0),
            right:  Val::Auto,
            bottom: Val::Auto,
        };

        if effect.frame_timer.just_finished() {
            let next_frame = effect.current_frame + 1;
            if next_frame >= effect.frame_count {
                // Séquence terminée — reste sur la dernière frame
                effect.sequence_done = true;
            } else {
                effect.current_frame = next_frame;
                img_node.image = frames.handles[effect.current_frame].clone();
            }
        }
    }
}

pub fn cleanup_finished_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut AttackEffect)>,
) {
    for (entity, mut effect) in query.iter_mut() {
        effect.lifetime.tick(time.delta());
        if effect.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}
