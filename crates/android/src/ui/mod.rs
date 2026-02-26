//! Plugin UI — rendu des écrans du jeu avec `bevy_ui`.
//!
//! Chaque sous-module correspond à un écran de l'application.

pub mod common;
pub mod screens;

use screens::*;

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
            // ── Nouveau monstre (choix du type) ─────────────────
            .add_systems(
                OnEnter(GameScreen::NewMonster),
                new_monster::spawn_new_monster,
            )
            .add_systems(
                Update,
                new_monster::handle_new_monster_input.run_if(in_state(GameScreen::NewMonster)),
            )
            // ── Nommage du monstre ──────────────────────────────
            .add_systems(OnEnter(GameScreen::NamingMonster), naming::spawn_naming)
            .add_systems(
                Update,
                naming::handle_naming_input.run_if(in_state(GameScreen::NamingMonster)),
            )
            // ── Sélection du monstre ────────────────────────────
            .add_systems(
                OnEnter(GameScreen::SelectMonster),
                select_monster::spawn_select_monster,
            )
            .add_systems(
                Update,
                select_monster::handle_select_monster_input
                    .run_if(in_state(GameScreen::SelectMonster)),
            )
            // ── Entraînement ────────────────────────────────────
            .add_systems(OnEnter(GameScreen::Training), training::spawn_training)
            .add_systems(
                Update,
                training::handle_training_input.run_if(in_state(GameScreen::Training)),
            )
            // ── Combat ──────────────────────────────────────────
            .add_systems(OnEnter(GameScreen::Battle), battle::spawn_battle_ui)
            .add_systems(
                Update,
                battle::handle_battle_input.run_if(in_state(GameScreen::Battle)),
            )
            // ── PvP Searching ───────────────────────────────────
            .add_systems(OnEnter(GameScreen::PvpSearching), pvp::spawn_pvp_searching)
            .add_systems(
                Update,
                pvp::handle_pvp_searching_input.run_if(in_state(GameScreen::PvpSearching)),
            )
            // ── Breeding Searching ──────────────────────────────
            .add_systems(
                OnEnter(GameScreen::BreedingSearching),
                breeding::spawn_breeding_searching,
            )
            .add_systems(
                Update,
                breeding::handle_breeding_searching_input
                    .run_if(in_state(GameScreen::BreedingSearching)),
            )
            // ── Breeding Naming ─────────────────────────────────
            .add_systems(
                OnEnter(GameScreen::BreedingNaming),
                breeding::spawn_breeding_naming,
            )
            .add_systems(
                Update,
                breeding::handle_breeding_naming_input.run_if(in_state(GameScreen::BreedingNaming)),
            )
            // ── Breeding Result ─────────────────────────────────
            .add_systems(
                OnEnter(GameScreen::BreedingResult),
                breeding::spawn_breeding_result,
            )
            .add_systems(
                Update,
                breeding::handle_breeding_result_input.run_if(in_state(GameScreen::BreedingResult)),
            )
            // ── Cimetière ───────────────────────────────────────
            .add_systems(OnEnter(GameScreen::Cemetery), cemetery::spawn_cemetery)
            .add_systems(
                Update,
                cemetery::handle_cemetery_input.run_if(in_state(GameScreen::Cemetery)),
            )
            // ── Aide ────────────────────────────────────────────
            .add_systems(OnEnter(GameScreen::Help), help::spawn_help)
            .add_systems(
                Update,
                help::handle_help_input.run_if(in_state(GameScreen::Help)),
            )
            // ── Caméra 2D ───────────────────────────────────────
            .add_systems(Startup, spawn_camera);
    }
}

/// Spawn la caméra 2D principale.
fn spawn_camera(mut commands: Commands) {
    log::info!("📷 spawn_camera — création Camera2d");
    commands.spawn(Camera2d);
}
