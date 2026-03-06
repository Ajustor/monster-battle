pub mod battle;
pub mod breeding;
pub mod cemetery;
pub mod common;
pub mod help;
pub mod main_menu;
pub mod minigame;
pub mod monster_list;
pub mod naming;
pub mod new_monster;
pub mod pvp;
pub mod training;

use ratatui::Frame;

use crate::app::App;

/// Cible après la sélection d'un monstre.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectMonsterTarget {
    /// Entraînement contre un bot.
    Training,
    /// Combat PvP.
    CombatPvP,
    /// Reproduction.
    Breeding,
    /// Mini-jeu.
    Minigame,
}

/// Les différents écrans de l'application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    /// Menu principal.
    MainMenu,
    /// Liste des monstres vivants.
    MonsterList,
    /// Création d'un nouveau monstre starter — choix du type.
    NewMonster,
    /// Saisie du nom du monstre après choix du type.
    NamingMonster {
        /// Index du type élémentaire choisi.
        type_index: usize,
    },
    /// Sélection du monstre avant une action (entraînement, PvP, reproduction).
    SelectMonster(SelectMonsterTarget),
    /// Entraînement contre un bot.
    /// `wild` = false : docile (50% XP, pas de mort) / `wild` = true : sauvage (100% XP, mort possible).
    Training { wild: bool },
    /// Interface de combat PvP.
    Combat(pvp::PvpPhase),
    /// Interface de reproduction.
    Breeding(breeding::BreedPhase),
    /// Combat interactif style Pokémon.
    Battle,
    /// Cimetière (monstres morts).
    Cemetery,
    /// Écran d'aide / tutoriel.
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

/// Point d'entrée du rendu : dispatche vers l'écran courant.
pub fn draw(frame: &mut Frame, app: &App) {
    use ratatui::layout::{Constraint, Direction, Layout};

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header
            Constraint::Min(10),   // body
            Constraint::Length(3), // footer / message
        ])
        .split(frame.area());

    common::draw_header(frame, chunks[0]);

    match &app.current_screen {
        Screen::MainMenu => main_menu::draw(frame, chunks[1], app),
        Screen::MonsterList => monster_list::draw(frame, chunks[1], app),
        Screen::NewMonster => new_monster::draw(frame, chunks[1], app),
        Screen::NamingMonster { type_index } => naming::draw(frame, chunks[1], app, *type_index),
        Screen::Cemetery => cemetery::draw(frame, chunks[1], app),
        Screen::Help => help::draw(frame, chunks[1], app),
        Screen::MinigameTypeSelect => minigame::draw_select_game_type(frame, chunks[1], app),
        Screen::MinigameSelect => minigame::draw_select_difficulty(frame, chunks[1], app),
        Screen::MinigamePlay => {
            if let Some(ref game) = app.tictactoe {
                let name = app.minigame_monster_name.as_deref().unwrap_or("?");
                minigame::draw_game(frame, chunks[1], game, name);
            }
        }
        Screen::MemoryPlay => {
            if let Some(ref game) = app.memory_game {
                let name = app.minigame_monster_name.as_deref().unwrap_or("?");
                minigame::draw_memory(frame, chunks[1], game, name);
            }
        }
        Screen::ReflexPlay => {
            if let Some(ref game) = app.reflex_game {
                let name = app.minigame_monster_name.as_deref().unwrap_or("?");
                minigame::draw_reflex(frame, chunks[1], game, name);
            }
        }
        Screen::RpsPlay => {
            if let Some(ref game) = app.rps_game {
                let name = app.minigame_monster_name.as_deref().unwrap_or("?");
                minigame::draw_rps(frame, chunks[1], game, name);
            }
        }
        Screen::Battle => battle::draw(frame, chunks[1], app),
        Screen::SelectMonster(target) => common::draw_select_monster(frame, chunks[1], app, target),
        Screen::Training { wild } => training::draw_select(frame, chunks[1], app, *wild),
        Screen::Combat(phase) => match phase {
            pvp::PvpPhase::Searching => pvp::draw_searching(frame, chunks[1], app),
            pvp::PvpPhase::Matched { opponent_name } => {
                pvp::draw_matched(frame, chunks[1], app, opponent_name)
            }
            pvp::PvpPhase::Error(e) => pvp::draw_error(frame, chunks[1], e),
        },
        Screen::Breeding(phase) => match phase {
            breeding::BreedPhase::Searching => breeding::draw_searching(frame, chunks[1], app),
            breeding::BreedPhase::Matched { opponent_name } => {
                breeding::draw_matched(frame, chunks[1], app, opponent_name)
            }
            breeding::BreedPhase::NamingChild => breeding::draw_naming_child(frame, chunks[1], app),
            breeding::BreedPhase::Result => breeding::draw_breed_result(frame, chunks[1], app),
            breeding::BreedPhase::Error(e) => breeding::draw_error(frame, chunks[1], e),
        },
    }

    common::draw_footer(frame, chunks[2], app);
}
