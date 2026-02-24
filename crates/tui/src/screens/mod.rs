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
    /// Détails d'un monstre spécifique.
    MonsterDetail,
    /// Entraînement contre un bot (50% XP).
    Training,
    /// Log de combat d'entraînement en cours.
    TrainingResult,
    /// Interface de combat PvP.
    Combat(pvp::PvpPhase),
    /// Résultat du combat PvP.
    CombatResult,
    /// Interface de reproduction.
    Breeding(breeding::BreedPhase),
    /// Résultat de la reproduction.
    BreedingResult,
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
        Screen::Training => training::draw_select(frame, chunks[1], app),
        Screen::TrainingResult => training::draw_result(frame, chunks[1], app),
        Screen::Combat(phase) => match phase {
            pvp::PvpPhase::Menu => pvp::draw_menu(frame, chunks[1], app),
            pvp::PvpPhase::WaitingForOpponent => pvp::draw_waiting(frame, chunks[1], app),
            pvp::PvpPhase::EnterAddress => pvp::draw_enter_address(frame, chunks[1], app),
            pvp::PvpPhase::Connecting => pvp::draw_connecting(frame, chunks[1], app),
            pvp::PvpPhase::WaitingForAccept => pvp::draw_waiting(frame, chunks[1], app),
            pvp::PvpPhase::ReceivedChallenge { opponent_name } => {
                pvp::draw_received_challenge(frame, chunks[1], app, opponent_name)
            }
            pvp::PvpPhase::Fighting => pvp::draw_fighting(frame, chunks[1], app),
            pvp::PvpPhase::Result => pvp::draw_pvp_result(frame, chunks[1], app),
            pvp::PvpPhase::Error(e) => pvp::draw_error(frame, chunks[1], e),
        },
        Screen::CombatResult => pvp::draw_pvp_result(frame, chunks[1], app),
        Screen::Breeding(phase) => match phase {
            breeding::BreedPhase::Menu => breeding::draw_menu(frame, chunks[1], app),
            breeding::BreedPhase::WaitingForPartner => {
                breeding::draw_waiting(frame, chunks[1], app)
            }
            breeding::BreedPhase::EnterAddress => {
                breeding::draw_enter_address(frame, chunks[1], app)
            }
            breeding::BreedPhase::Connecting => breeding::draw_connecting(frame, chunks[1], app),
            breeding::BreedPhase::ReceivedProposal {
                partner_monster_name,
            } => breeding::draw_received_proposal(frame, chunks[1], app, partner_monster_name),
            breeding::BreedPhase::WaitingForAccept => breeding::draw_waiting(frame, chunks[1], app),
            breeding::BreedPhase::NamingChild => breeding::draw_naming_child(frame, chunks[1], app),
            breeding::BreedPhase::Result => breeding::draw_breed_result(frame, chunks[1], app),
            breeding::BreedPhase::Error(e) => breeding::draw_error(frame, chunks[1], e),
        },
        Screen::BreedingResult => breeding::draw_breed_result(frame, chunks[1], app),
        _ => common::draw_placeholder(frame, chunks[1], "En construction..."),
    }

    common::draw_footer(frame, chunks[2], app);
}
