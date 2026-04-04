//! Plugin d'animations d'effets de combat.
//! Chaque attaque déclenche un effet visuel basé sur le type élémentaire.
//! Les sprites viennent du Kenney Particle Pack (CC0).

use bevy::prelude::*;
use monster_battle_core::types::ElementType;

// ── Plugin ──────────────────────────────────────────────────────────

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

// ── Composants & événements ─────────────────────────────────────────

/// Marqueur d'un effet d'attaque en cours d'animation.
#[derive(Component)]
pub struct AttackEffect {
    /// Durée entre deux frames (secondes).
    pub frame_timer: Timer,
    /// Nombre total de frames (sprites dans la séquence).
    pub frame_count: usize,
    /// Frame courante (0-based).
    pub current_frame: usize,
    /// Timer de durée totale avant suppression.
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
    /// Position écran 2D de la cible (centre de l'effet).
    pub position: Vec2,
    /// Secondes d'attente avant d'afficher l'effet (correspond au moment d'impact).
    pub startup_delay: f32,
}

// ── Constantes ──────────────────────────────────────────────────────

const FRAME_DURATION: f32 = 0.11;
const HOLD_DURATION: f32 = 0.06;
const EFFECT_SIZE: f32 = 180.0;

// ── Mapping ElementType → sprites ──────────────────────────────────

fn sprites_for_element(element: ElementType) -> Vec<&'static str> {
    match element {
        ElementType::Fire => vec![
            "effects/fire_01.png",
            "effects/fire_02.png",
            "effects/fire_03.png",
        ],
        ElementType::Water => vec![
            "effects/water_01.png",
            "effects/water_02.png",
            "effects/water_03.png",
        ],
        ElementType::Electric => vec![
            "effects/electric_01.png",
            "effects/electric_02.png",
            "effects/electric_03.png",
        ],
        ElementType::Earth => vec![
            "effects/earth_01.png",
            "effects/earth_02.png",
            "effects/earth_03.png",
        ],
        ElementType::Wind => vec![
            "effects/wind_01.png",
            "effects/wind_02.png",
            "effects/wind_03.png",
        ],
        ElementType::Shadow => vec![
            "effects/shadow_01.png",
            "effects/shadow_02.png",
            "effects/shadow_03.png",
        ],
        ElementType::Light => vec![
            "effects/light_01.png",
            "effects/light_02.png",
            "effects/light_03.png",
        ],
        ElementType::Plant => vec![
            "effects/plant_01.png",
            "effects/plant_02.png",
            "effects/plant_03.png",
        ],
        ElementType::Normal => vec![
            "effects/normal_01.png",
            "effects/normal_02.png",
            "effects/normal_03.png",
        ],
    }
}

/// Teinte de couleur pour chaque type (pour renforcer l'identité visuelle).
fn tint_for_element(element: ElementType) -> Color {
    match element {
        ElementType::Fire     => Color::srgb(1.0, 0.4, 0.05),
        ElementType::Water    => Color::srgb(0.2, 0.6, 1.0),
        ElementType::Electric => Color::srgb(1.0, 0.95, 0.1),
        ElementType::Earth    => Color::srgb(0.75, 0.5, 0.15),
        ElementType::Wind     => Color::srgb(0.5, 0.95, 0.75),
        ElementType::Shadow   => Color::srgb(0.55, 0.1, 0.85),
        ElementType::Light    => Color::srgb(1.0, 1.0, 0.75),
        ElementType::Plant    => Color::srgb(0.2, 0.85, 0.2),
        ElementType::Normal   => Color::srgb(0.75, 0.75, 0.75),
    }
}

// ── Systèmes ────────────────────────────────────────────────────────

/// Spawne un effet d'attaque quand un `PlayAttackEffect` est reçu.
pub fn handle_play_attack_effect(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut events: EventReader<PlayAttackEffect>,
) {
    for event in events.read() {
        let sprites = sprites_for_element(event.element);
        let frame_count = sprites.len();
        let tint = tint_for_element(event.element);

        if frame_count == 0 {
            continue;
        }

        let first_sprite: Handle<Image> = asset_server.load(sprites[0]);
        let delay = event.startup_delay;
        let lifetime = delay + frame_count as f32 * FRAME_DURATION + HOLD_DURATION;

        commands.spawn((
            AttackEffect {
                frame_timer:   Timer::from_seconds(FRAME_DURATION, TimerMode::Repeating),
                frame_count,
                current_frame: 0,
                lifetime:      Timer::from_seconds(lifetime, TimerMode::Once),
                startup_delay: Timer::from_seconds(delay.max(0.001), TimerMode::Once),
                started:       delay <= 0.0,
                sequence_done: false,
            },
            AttackFrames {
                paths: sprites.iter().map(|s| s.to_string()).collect(),
                element: event.element,
            },
            Sprite {
                image: first_sprite,
                color: tint,
                custom_size: Some(Vec2::splat(EFFECT_SIZE)),
                ..default()
            },
            Transform::from_translation(Vec3::new(event.position.x, event.position.y, 10.0)),
            if delay > 0.0 { Visibility::Hidden } else { Visibility::Visible },
        ));
    }
}

/// Composant interne : liste des chemins de frames.
#[derive(Component)]
struct AttackFrames {
    paths: Vec<String>,
    element: ElementType,
}

/// Avance les frames et anime le scale de chaque effet d'attaque.
pub fn animate_attack_effects(
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut query: Query<(&mut AttackEffect, &AttackFrames, &mut Sprite, &mut Visibility, &mut Transform)>,
) {
    for (mut effect, frames, mut sprite, mut vis, mut transform) in query.iter_mut() {
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
            continue;
        }

        effect.frame_timer.tick(time.delta());

        // Animation de scale : expansion rapide puis stabilisation
        let sequence_progress = (effect.current_frame as f32 + effect.frame_timer.fraction())
            / effect.frame_count as f32;
        let scale = if sequence_progress < 0.3 {
            let t = sequence_progress / 0.3;
            0.4 + 0.7 * t
        } else if sequence_progress < 0.6 {
            let t = (sequence_progress - 0.3) / 0.3;
            1.1 - 0.1 * t
        } else {
            1.0
        };
        transform.scale = Vec3::splat(scale);
        sprite.custom_size = Some(Vec2::splat(EFFECT_SIZE));

        if effect.frame_timer.just_finished() {
            let next_frame = effect.current_frame + 1;
            if next_frame >= effect.frame_count {
                effect.sequence_done = true;
            } else {
                effect.current_frame = next_frame;
                let path = &frames.paths[effect.current_frame];
                sprite.image = asset_server.load(path.as_str());
            }
        }
    }
}

/// Supprime les effets dont la durée de vie est écoulée.
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
