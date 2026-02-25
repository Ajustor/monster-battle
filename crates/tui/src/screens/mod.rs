pub mod battle;
pub mod breeding;
pub mod cemetery;
pub mod common;
pub mod main_menu;
pub mod monster_list;
pub mod naming;
pub mod new_monster;
pub mod pvp;
pub mod training;

use ratatui::Frame;

use crate::app::App;

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
        Screen::Battle => battle::draw(frame, chunks[1], app),
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
