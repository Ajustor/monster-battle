//! Plugin d'animations d'effets de combat.
//! Chaque attaque déclenche un effet visuel basé sur le type élémentaire.
//! Les sprites viennent du Kenney Particle Pack (CC0).
//!
//! Les effets sont des nœuds UI root avec position absolute — ils s'affichent
//! par-dessus tout le reste grâce à GlobalZIndex.

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
}

/// Événement : jouer un effet d'attaque.
/// `position` est en coordonnées viewport normalisées (0.0–1.0, origin haut-gauche).
#[derive(Event)]
pub struct PlayAttackEffect {
    pub element: ElementType,
    /// Position dans le viewport : Vec2(x%, y%) ex: Vec2(0.65, 0.30) pour l'adversaire.
    pub position: Vec2,
}

#[derive(Component)]
pub struct AttackFrames {
    pub paths: Vec<String>,
}

fn sprites_for_element(element: ElementType) -> Vec<&'static str> {
    match element {
        ElementType::Fire     => vec!["effects/fire_01.png",     "effects/fire_02.png",     "effects/fire_03.png"],
        ElementType::Water    => vec!["effects/water_01.png",    "effects/water_02.png",    "effects/water_03.png"],
        ElementType::Electric => vec!["effects/electric_01.png", "effects/electric_02.png", "effects/electric_03.png"],
        ElementType::Earth    => vec!["effects/earth_01.png",    "effects/earth_02.png",    "effects/earth_03.png"],
        ElementType::Wind     => vec!["effects/wind_01.png",     "effects/wind_02.png",     "effects/wind_03.png"],
        ElementType::Shadow   => vec!["effects/shadow_01.png",   "effects/shadow_02.png",   "effects/shadow_03.png"],
        ElementType::Light    => vec!["effects/light_01.png",    "effects/light_02.png",    "effects/light_03.png"],
        ElementType::Plant    => vec!["effects/plant_01.png",    "effects/plant_02.png",    "effects/plant_03.png"],
        ElementType::Normal   => vec!["effects/normal_01.png",   "effects/normal_02.png",   "effects/normal_03.png"],
    }
}

fn tint_for_element(element: ElementType) -> Color {
    match element {
        ElementType::Fire     => Color::srgba(1.0, 0.5, 0.1, 0.9),
        ElementType::Water    => Color::srgba(0.3, 0.7, 1.0, 0.9),
        ElementType::Electric => Color::srgba(1.0, 1.0, 0.2, 0.9),
        ElementType::Earth    => Color::srgba(0.7, 0.5, 0.2, 0.9),
        ElementType::Wind     => Color::srgba(0.6, 1.0, 0.8, 0.9),
        ElementType::Shadow   => Color::srgba(0.6, 0.2, 0.9, 0.9),
        ElementType::Light    => Color::srgba(1.0, 1.0, 0.8, 0.9),
        ElementType::Plant    => Color::srgba(0.3, 0.9, 0.3, 0.9),
        ElementType::Normal   => Color::srgba(0.8, 0.8, 0.8, 0.9),
    }
}

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

        let first_image: Handle<Image> = asset_server.load(sprites[0]);
        let size = 128.0_f32;

        // Nœud UI root en position absolute, centré sur la cible via margins négatives
        commands.spawn((
            AttackEffect {
                frame_timer: Timer::from_seconds(0.10, TimerMode::Repeating),
                frame_count,
                current_frame: 0,
                lifetime: Timer::from_seconds(0.10 * frame_count as f32 * 3.0, TimerMode::Once),
            },
            AttackFrames {
                paths: sprites.iter().map(|s| s.to_string()).collect(),
            },
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(event.position.x * 100.0),
                top: Val::Percent(event.position.y * 100.0),
                width: Val::Px(size),
                height: Val::Px(size),
                margin: UiRect {
                    left: Val::Px(-size / 2.0),
                    top: Val::Px(-size / 2.0),
                    right: Val::Auto,
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
        ));
    }
}

pub fn animate_attack_effects(
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut query: Query<(&mut AttackEffect, &AttackFrames, &mut ImageNode)>,
) {
    for (mut effect, frames, mut img_node) in query.iter_mut() {
        effect.frame_timer.tick(time.delta());
        if effect.frame_timer.just_finished() {
            effect.current_frame = (effect.current_frame + 1) % effect.frame_count;
            img_node.image = asset_server.load(frames.paths[effect.current_frame].as_str());
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
