//! Machine à états du jeu — miroir du `Screen` enum de la TUI.
//!
//! Chaque variante correspond à un écran de l'application mobile.

use bevy::prelude::*;
use bevy::state::state::{OnEnter, OnExit, States};

use monster_battle_core::Monster;
use monster_battle_core::battle::BattleState;
use monster_battle_core::minigame::MinigameType;
use monster_battle_core::minigame::memory::MemoryGame;
use monster_battle_core::minigame::reflex::ReflexGame;
use monster_battle_core::minigame::rps::RpsGame;
use monster_battle_core::minigame::tictactoe::TicTacToe;
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
    /// Sélection du type de mini-jeu.
    MinigameTypeSelect,
    /// Sélection de la difficulté du mini-jeu.
    MinigameSelect,
    /// Partie de morpion en cours.
    MinigamePlay,
    /// Partie de Memory en cours.
    MemoryPlay,
    /// Partie de Réflexe en cours.
    ReflexPlay,
    /// Partie de PPC élémentaire en cours.
    RpsPlay,
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
    Minigame,
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
    /// Drapeau signalant que l'UI de combat doit être reconstruite (polling réseau).
    pub battle_ui_dirty: bool,
    /// Type de mini-jeu sélectionné.
    pub minigame_type: Option<MinigameType>,
    /// État de la partie de morpion en cours.
    pub tictactoe: Option<TicTacToe>,
    /// État de la partie de Memory en cours.
    pub memory_game: Option<MemoryGame>,
    /// État de la partie de Réflexe en cours.
    pub reflex_game: Option<ReflexGame>,
    /// État de la partie de PPC en cours.
    pub rps_game: Option<RpsGame>,
    /// UUID du monstre participant au mini-jeu.
    pub minigame_monster_id: Option<uuid::Uuid>,
    /// Nom du monstre participant au mini-jeu.
    pub minigame_monster_name: Option<String>,
    /// Vrai si le popup de sélection de nourriture est affiché.
    pub food_selecting: bool,
    /// Index courant dans la liste des types de nourriture.
    pub food_select_index: usize,
    /// Message d'événement aléatoire à afficher.
    pub event_message: Option<String>,
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
            battle_ui_dirty: false,
            minigame_type: None,
            tictactoe: None,
            memory_game: None,
            reflex_game: None,
            rps_game: None,
            minigame_monster_id: None,
            minigame_monster_name: None,
            food_selecting: false,
            food_select_index: 0,
            event_message: None,
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
        // Le nettoyage des entités d'écran est géré automatiquement par
        // `StateScoped` + `enable_state_scoped_entities` (despawn_recursive).
        // Ne PAS utiliser de cleanup_screen manuel avec despawn() non-récursif
        // car l'ordre d'exécution avec StateScoped est ambigu dans ExitSchedules.
        app.insert_resource(GameData::new(data_dir))
            .insert_resource(crate::ui::screens::training::TrainingWild(false))
            .enable_state_scoped_entities::<GameScreen>()
            .add_systems(OnEnter(GameScreen::MainMenu), on_enter_main_menu)
            .add_systems(OnEnter(GameScreen::MonsterList), on_enter_monster_list)
            .add_systems(
                Update,
                crate::ui::common::sync_scroll_position.run_if(in_state(GameScreen::MonsterList)),
            )
            .add_systems(OnEnter(GameScreen::NewMonster), on_enter_new_monster)
            .add_systems(OnEnter(GameScreen::NamingMonster), on_enter_naming)
            .add_systems(OnEnter(GameScreen::NamingMonster), enable_ime)
            .add_systems(OnExit(GameScreen::NamingMonster), disable_ime)
            .add_systems(OnEnter(GameScreen::SelectMonster), on_enter_select_monster)
            .add_systems(OnEnter(GameScreen::Training), on_enter_training)
            .add_systems(OnEnter(GameScreen::Cemetery), on_enter_cemetery)
            .add_systems(OnEnter(GameScreen::Help), on_enter_help)
            .add_systems(OnEnter(GameScreen::Battle), on_enter_battle)
            .add_systems(OnEnter(GameScreen::PvpSearching), on_enter_pvp_searching)
            .add_systems(
                OnEnter(GameScreen::BreedingSearching),
                on_enter_breeding_searching,
            )
            .add_systems(
                OnEnter(GameScreen::BreedingNaming),
                on_enter_breeding_naming,
            )
            .add_systems(OnEnter(GameScreen::BreedingNaming), enable_ime)
            .add_systems(OnExit(GameScreen::BreedingNaming), disable_ime)
            .add_systems(
                OnEnter(GameScreen::BreedingResult),
                on_enter_breeding_result,
            )
            .add_systems(
                OnEnter(GameScreen::MinigameTypeSelect),
                on_enter_minigame_type_select,
            )
            .add_systems(
                OnEnter(GameScreen::MinigameSelect),
                on_enter_minigame_select,
            )
            .add_systems(OnEnter(GameScreen::MinigamePlay), on_enter_minigame_play)
            .add_systems(OnEnter(GameScreen::MemoryPlay), on_enter_memory_play)
            .add_systems(OnEnter(GameScreen::ReflexPlay), on_enter_reflex_play)
            .add_systems(OnEnter(GameScreen::RpsPlay), on_enter_rps_play);
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
//  Marqueur pour les entités d'écran (conservé pour compatibilité)
// ═══════════════════════════════════════════════════════════════════

/// Marqueur ajouté à toutes les entités créées par un écran.
/// Le nettoyage est géré par `StateScoped` + `enable_state_scoped_entities`
/// qui utilise `despawn_recursive` automatiquement.
#[derive(Component)]
pub struct ScreenEntity;

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
    data.food_selecting = false;
    data.food_select_index = 0;
    data.event_message = None;

    // Décroissance du bonheur des monstres vivants
    if let Ok(mut monsters) = data.storage.list_alive() {
        for m in monsters.iter_mut() {
            m.decay_happiness();
            let _ = data.storage.save(m);
        }
    }

    // Vérifier un événement aléatoire pour le monstre sélectionné
    if let Ok(mut monsters) = data.storage.list_alive() {
        let idx = data
            .monster_select_index
            .min(monsters.len().saturating_sub(1));
        if let Some(monster) = monsters.get_mut(idx) {
            if let Some(event) = monster.try_random_event() {
                let msg = monster.apply_event(&event);
                let _ = data.storage.save(monster);
                data.event_message = Some(msg);
            }
        }
    }
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

fn on_enter_pvp_searching(mut commands: Commands, mut data: ResMut<GameData>) {
    data.message = None;

    // Lancer la tâche réseau PvP
    let monsters = match data.storage.list_alive() {
        Ok(m) if !m.is_empty() => m,
        _ => {
            data.message = Some("Pas de monstre vivant !".to_string());
            return;
        }
    };

    let idx = data.monster_select_index.min(monsters.len() - 1);
    let monster = monsters[idx].clone();
    let fighter_id = monster.id;

    crate::net_task::start_pvp_task(&mut commands, monster, fighter_id);
}

fn on_enter_breeding_searching(mut commands: Commands, mut data: ResMut<GameData>) {
    data.message = None;
    data.remote_monster = None;

    // Lancer la tâche réseau de reproduction
    let monsters = match data.storage.list_alive() {
        Ok(m) if !m.is_empty() => m,
        _ => {
            data.message = Some("Pas de monstre vivant !".to_string());
            return;
        }
    };

    let idx = data.monster_select_index.min(monsters.len() - 1);
    let monster = monsters[idx].clone();
    let fighter_id = monster.id;

    crate::net_task::start_breeding_task(&mut commands, monster, fighter_id);
}

fn on_enter_breeding_naming(mut data: ResMut<GameData>) {
    data.name_input.clear();
    data.message = None;
}

fn on_enter_breeding_result(mut data: ResMut<GameData>) {
    data.scroll_offset = 0;
}

fn on_enter_minigame_type_select(mut data: ResMut<GameData>) {
    data.menu_index = 0;
}

fn on_enter_minigame_select(mut data: ResMut<GameData>) {
    data.menu_index = 0;
}

fn on_enter_minigame_play(_data: ResMut<GameData>) {
    // Le tictactoe est déjà configuré avant la transition.
}

fn on_enter_memory_play(_data: ResMut<GameData>) {
    // Le memory_game est déjà configuré avant la transition.
}

fn on_enter_reflex_play(_data: ResMut<GameData>) {
    // Le reflex_game est déjà configuré avant la transition.
}

fn on_enter_rps_play(_data: ResMut<GameData>) {
    // Le rps_game est déjà configuré avant la transition.
}

/// Active le clavier système pour les écrans de saisie de texte.
fn enable_ime(mut commands: Commands, mut windows: Query<&mut Window>) {
    if let Ok(mut window) = windows.get_single_mut() {
        window.ime_enabled = true;
    }
    // Insérer le timer de ré-essai (la vue peut ne pas être prête immédiatement)
    commands.insert_resource(crate::ui::common::KeyboardRetryTimer {
        frames_remaining: 10,
    });
    crate::ui::common::show_system_keyboard();
}

/// Désactive le clavier système.
fn disable_ime(mut commands: Commands, mut windows: Query<&mut Window>) {
    if let Ok(mut window) = windows.get_single_mut() {
        window.ime_enabled = false;
    }
    commands.remove_resource::<crate::ui::common::KeyboardRetryTimer>();
    crate::ui::common::hide_system_keyboard();
}
// Le BattleState est initialisé avant la transition (par training ou pvp).
