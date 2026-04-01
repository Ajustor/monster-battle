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
}

/// Événement : jouer un effet d'attaque sur la cible.
#[derive(Event)]
pub struct PlayAttackEffect {
    pub element: ElementType,
    /// Position écran 2D de la cible (centre de l'effet).
    pub position: Vec2,
}

// ── Mapping ElementType → sprites ──────────────────────────────────

/// Retourne la liste de chemins d'assets pour un type élémentaire.
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
        ElementType::Fire => Color::srgb(1.0, 0.5, 0.1),
        ElementType::Water => Color::srgb(0.3, 0.7, 1.0),
        ElementType::Electric => Color::srgb(1.0, 1.0, 0.2),
        ElementType::Earth => Color::srgb(0.7, 0.5, 0.2),
        ElementType::Wind => Color::srgb(0.6, 1.0, 0.8),
        ElementType::Shadow => Color::srgb(0.6, 0.2, 0.9),
        ElementType::Light => Color::srgb(1.0, 1.0, 0.8),
        ElementType::Plant => Color::srgb(0.3, 0.9, 0.3),
        ElementType::Normal => Color::srgb(0.8, 0.8, 0.8),
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

        // Spawne une entité par frame (on joue frame_01 d'abord, puis on switch via le timer)
        let first_sprite: Handle<Image> = asset_server.load(sprites[0]);

        commands.spawn((
            AttackEffect {
                frame_timer: Timer::from_seconds(0.12, TimerMode::Repeating),
                frame_count,
                current_frame: 0,
                lifetime: Timer::from_seconds(0.12 * frame_count as f32 * 2.0, TimerMode::Once),
            },
            // Stocke les chemins pour avancer les frames
            AttackFrames {
                paths: sprites.iter().map(|s| s.to_string()).collect(),
                element: event.element,
            },
            Sprite {
                image: first_sprite,
                color: tint,
                custom_size: Some(Vec2::splat(128.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(event.position.x, event.position.y, 10.0)),
        ));
    }
}

/// Composant interne : liste des chemins de frames.
#[derive(Component)]
struct AttackFrames {
    paths: Vec<String>,
    element: ElementType,
}

/// Avance les frames de chaque effet d'attaque.
pub fn animate_attack_effects(
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut query: Query<(&mut AttackEffect, &AttackFrames, &mut Sprite)>,
) {
    for (mut effect, frames, mut sprite) in query.iter_mut() {
        effect.frame_timer.tick(time.delta());
        if effect.frame_timer.just_finished() {
            effect.current_frame = (effect.current_frame + 1) % effect.frame_count;
            let path = &frames.paths[effect.current_frame];
            sprite.image = asset_server.load(path.as_str());
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
