//! Plugin UI — rendu des écrans du jeu avec `bevy_ui`.
//!
//! Chaque sous-module correspond à un écran de l'application.

pub mod battle;
pub mod common;
pub mod main_menu;
pub mod monster_list;

use bevy::prelude::*;
use bevy::state::condition::in_state;
use bevy::state::state::OnEnter;

use crate::game::GameScreen;

/// Plugin regroupant tous les systèmes UI.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            // ── Menu principal ───────────────────────────────────
            .add_systems(OnEnter(GameScreen::MainMenu), main_menu::spawn_menu)
            .add_systems(
                Update,
                main_menu::handle_menu_input.run_if(in_state(GameScreen::MainMenu)),
            )
            // ── Liste des monstres ──────────────────────────────
            .add_systems(
                OnEnter(GameScreen::MonsterList),
                monster_list::spawn_monster_list,
            )
            .add_systems(
                Update,
                monster_list::handle_monster_list_input.run_if(in_state(GameScreen::MonsterList)),
            )
            // ── Combat ──────────────────────────────────────────
            .add_systems(OnEnter(GameScreen::Battle), battle::spawn_battle_ui)
            .add_systems(
                Update,
                battle::handle_battle_input.run_if(in_state(GameScreen::Battle)),
            )
            // ── Caméra 2D ───────────────────────────────────────
            .add_systems(Startup, spawn_camera);
    }
}

/// Spawn la caméra 2D principale.
fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
