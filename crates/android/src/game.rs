//! Machine à états du jeu — miroir du `Screen` enum de la TUI.
//!
//! Chaque variante correspond à un écran de l'application mobile.

use bevy::prelude::*;
use bevy::state::state::{OnEnter, OnExit, States};

use monster_battle_core::Monster;
use monster_battle_core::battle::BattleState;
use monster_battle_storage::{LocalStorage, MonsterStorage};

// ═══════════════════════════════════════════════════════════════════
//  États du jeu (Bevy States)
// ═══════════════════════════════════════════════════════════════════

/// Écrans principaux du jeu — pilotent les transitions Bevy.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameScreen {
    /// Menu principal.
    #[default]
    MainMenu,
    /// Liste des monstres vivants.
    MonsterList,
    /// Création d'un monstre starter — choix du type.
    NewMonster,
    /// Saisie du nom du monstre.
    NamingMonster,
    /// Sélection du monstre (entraînement / PvP / reproduction).
    SelectMonster,
    /// Menu d'entraînement (choix docile / sauvage).
    Training,
    /// Combat interactif (PvP ou entraînement).
    Battle,
    /// Recherche d'adversaire PvP.
    PvpSearching,
    /// Recherche de partenaire reproduction.
    BreedingSearching,
    /// Nommage du bébé après reproduction.
    BreedingNaming,
    /// Résultat de la reproduction.
    BreedingResult,
    /// Cimetière (monstres morts).
    Cemetery,
    /// Écran d'aide.
    Help,
}

// ═══════════════════════════════════════════════════════════════════
//  Sous-états / contexte
// ═══════════════════════════════════════════════════════════════════

/// Cible de la sélection de monstre — détermine l'action après sélection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource)]
pub enum SelectMonsterTarget {
    Training,
    CombatPvP,
    Breeding,
}

// ═══════════════════════════════════════════════════════════════════
//  Ressources globales du jeu
// ═══════════════════════════════════════════════════════════════════

/// Ressource principale contenant l'état du jeu.
#[derive(Resource)]
pub struct GameData {
    /// Stockage local des monstres (chiffré).
    pub storage: LocalStorage,
    /// Index du monstre actuellement sélectionné.
    pub monster_select_index: usize,
    /// Index du choix dans les menus.
    pub menu_index: usize,
    /// Saisie de texte (nom du monstre, etc.).
    pub name_input: String,
    /// Index du type élémentaire choisi lors de la création.
    pub type_choice_index: usize,
    /// Message temporaire affiché à l'utilisateur.
    pub message: Option<String>,
    /// État du combat en cours (si applicable).
    pub battle_state: Option<BattleState>,
    /// Monstre distant reçu du réseau (PvP / reproduction).
    pub remote_monster: Option<Monster>,
    /// Résultat de la dernière reproduction.
    pub breed_result: Option<Monster>,
    /// Décalage de scroll pour les écrans longs.
    pub scroll_offset: usize,
}

impl GameData {
    /// Crée les données de jeu avec un répertoire de stockage.
    pub fn new(data_dir: impl AsRef<std::path::Path>) -> Self {
        // S'assurer que le répertoire de données existe
        log::info!("📂 GameData::new — data_dir={:?}", data_dir.as_ref());
        let _ = std::fs::create_dir_all(data_dir.as_ref());
        let storage =
            LocalStorage::new(data_dir).expect("Impossible d'initialiser le stockage local");
        log::info!("✅ GameData — storage initialisé avec succès");
        Self {
            storage,
            monster_select_index: 0,
            menu_index: 0,
            name_input: String::new(),
            type_choice_index: 0,
            message: None,
            battle_state: None,
            remote_monster: None,
            breed_result: None,
            scroll_offset: 0,
        }
    }

    /// Vérifie s'il existe au moins un monstre vivant.
    pub fn has_living_monster(&self) -> bool {
        self.storage
            .list_alive()
            .map(|v| !v.is_empty())
            .unwrap_or(false)
    }
}

// ═══════════════════════════════════════════════════════════════════
//  Plugin principal
// ═══════════════════════════════════════════════════════════════════

/// Plugin central : insère les ressources et les systèmes de transition.
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        // Initialiser GameData avec le répertoire de données approprié
        let data_dir = dirs_data_dir();
        app.insert_resource(GameData::new(data_dir))
            .insert_resource(crate::ui::screens::training::TrainingWild(false))
            .enable_state_scoped_entities::<GameScreen>()
            .add_systems(OnEnter(GameScreen::MainMenu), on_enter_main_menu)
            .add_systems(OnExit(GameScreen::MainMenu), cleanup_screen)
            .add_systems(OnEnter(GameScreen::MonsterList), on_enter_monster_list)
            .add_systems(OnExit(GameScreen::MonsterList), cleanup_screen)
            .add_systems(OnEnter(GameScreen::NewMonster), on_enter_new_monster)
            .add_systems(OnExit(GameScreen::NewMonster), cleanup_screen)
            .add_systems(OnEnter(GameScreen::NamingMonster), on_enter_naming)
            .add_systems(OnExit(GameScreen::NamingMonster), cleanup_screen)
            .add_systems(OnEnter(GameScreen::SelectMonster), on_enter_select_monster)
            .add_systems(OnExit(GameScreen::SelectMonster), cleanup_screen)
            .add_systems(OnEnter(GameScreen::Training), on_enter_training)
            .add_systems(OnExit(GameScreen::Training), cleanup_screen)
            .add_systems(OnEnter(GameScreen::Cemetery), on_enter_cemetery)
            .add_systems(OnExit(GameScreen::Cemetery), cleanup_screen)
            .add_systems(OnEnter(GameScreen::Help), on_enter_help)
            .add_systems(OnExit(GameScreen::Help), cleanup_screen)
            .add_systems(OnEnter(GameScreen::Battle), on_enter_battle)
            .add_systems(OnExit(GameScreen::Battle), cleanup_screen)
            .add_systems(OnEnter(GameScreen::PvpSearching), on_enter_pvp_searching)
            .add_systems(OnExit(GameScreen::PvpSearching), cleanup_screen)
            .add_systems(
                OnEnter(GameScreen::BreedingSearching),
                on_enter_breeding_searching,
            )
            .add_systems(OnExit(GameScreen::BreedingSearching), cleanup_screen)
            .add_systems(
                OnEnter(GameScreen::BreedingNaming),
                on_enter_breeding_naming,
            )
            .add_systems(OnExit(GameScreen::BreedingNaming), cleanup_screen)
            .add_systems(
                OnEnter(GameScreen::BreedingResult),
                on_enter_breeding_result,
            )
            .add_systems(OnExit(GameScreen::BreedingResult), cleanup_screen);
    }
}

/// Répertoire de données selon la plateforme.
fn dirs_data_dir() -> std::path::PathBuf {
    // Sur Android : utiliser le dossier interne de l'app.
    // Le chemin est /data/data/<package>/files
    #[cfg(target_os = "android")]
    {
        // Utiliser le vrai chemin interne de l'app (correspond au package dans le manifest)
        std::path::PathBuf::from("/data/data/com.ajustor.monsterbattle/files")
    }
    #[cfg(not(target_os = "android"))]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        std::path::PathBuf::from(format!("{}/.local/share/monster-battle", home))
    }
}

// ═══════════════════════════════════════════════════════════════════
//  Marqueur pour le nettoyage des entités d'écran
// ═══════════════════════════════════════════════════════════════════

/// Marqueur ajouté à toutes les entités créées par un écran.
/// Détruites automatiquement lors de la sortie de l'écran.
#[derive(Component)]
pub struct ScreenEntity;

/// Système de nettoyage générique — supprime toutes les entités marquées.
fn cleanup_screen(mut commands: Commands, query: Query<Entity, With<ScreenEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

// ═══════════════════════════════════════════════════════════════════
//  Systèmes d'entrée d'écran (stubs — à implémenter par écran)
// ═══════════════════════════════════════════════════════════════════

fn on_enter_main_menu(mut data: ResMut<GameData>) {
    log::info!(
        "🎮 on_enter_main_menu — state=MainMenu, has_monster={}",
        data.has_living_monster()
    );
    data.menu_index = 0;
    data.message = None;
}

fn on_enter_monster_list(mut data: ResMut<GameData>) {
    data.monster_select_index = 0;
}

fn on_enter_new_monster(mut data: ResMut<GameData>) {
    data.type_choice_index = 0;
}

fn on_enter_cemetery(mut data: ResMut<GameData>) {
    data.scroll_offset = 0;
}

fn on_enter_help(mut data: ResMut<GameData>) {
    data.scroll_offset = 0;
}

fn on_enter_battle(_data: ResMut<GameData>) {
    // Le battle_state est déjà configuré avant la transition.
}

fn on_enter_naming(mut data: ResMut<GameData>) {
    data.message = None;
    // name_input est déjà vidé par l'écran précédent (NewMonster)
}

fn on_enter_select_monster(mut data: ResMut<GameData>) {
    data.monster_select_index = 0;
}

fn on_enter_training(mut data: ResMut<GameData>) {
    data.menu_index = 0;
}

fn on_enter_pvp_searching(mut data: ResMut<GameData>) {
    data.message = None;
}

fn on_enter_breeding_searching(mut data: ResMut<GameData>) {
    data.message = None;
    data.remote_monster = None;
}

fn on_enter_breeding_naming(mut data: ResMut<GameData>) {
    data.name_input.clear();
    data.message = None;
}

fn on_enter_breeding_result(mut data: ResMut<GameData>) {
    data.scroll_offset = 0;
}
// Le BattleState est initialisé avant la transition (par training ou pvp).
