use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::App;
use crate::screens::Screen;

/// Dessine le header commun à tous les écrans.
pub fn draw_header(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            " 🐉 Monster Battle ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("v0.1.0", Style::default().fg(Color::DarkGray)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );

    frame.render_widget(title, area);
}

/// Dessine le footer commun à tous les écrans.
pub fn draw_footer(frame: &mut Frame, area: Rect, app: &App) {
    let text = if let Some(msg) = &app.message {
        msg.clone()
    } else {
        match &app.current_screen {
            Screen::MainMenu => "↑↓ Naviguer | Enter Sélectionner | q Quitter".to_string(),
            Screen::NamingMonster { .. } => {
                "Tapez le nom de votre monstre | Enter Valider | Esc Annuler".to_string()
            }
            Screen::Combat(phase) => match phase {
                crate::screens::pvp::PvpPhase::Error(_) => "Enter ou Esc pour revenir".to_string(),
                _ => "Esc Annuler".to_string(),
            },
            Screen::Breeding(phase) => match phase {
                crate::screens::breeding::BreedPhase::NamingChild => {
                    "Tapez le nom du bébé | Enter Valider | Esc Annuler".to_string()
                }
                crate::screens::breeding::BreedPhase::Result => {
                    "↑↓ Défiler | Enter ou q pour revenir".to_string()
                }
                crate::screens::breeding::BreedPhase::Error(_) => {
                    "Enter ou Esc pour revenir".to_string()
                }
                _ => "Esc Annuler".to_string(),
            },
            Screen::Battle => {
                if let Some(ref b) = app.battle_state {
                    match b.phase {
                        monster_battle_core::battle::BattlePhase::PlayerChooseAttack => {
                            "↑↓ Choisir | Enter Attaquer | Esc Fuir".to_string()
                        }
                        monster_battle_core::battle::BattlePhase::Victory
                        | monster_battle_core::battle::BattlePhase::Defeat => {
                            "Enter pour continuer...".to_string()
                        }
                        _ => "Enter / Espace pour continuer… | Esc Fuir".to_string(),
                    }
                } else {
                    String::new()
                }
            }
            _ => "↑↓ Naviguer | Enter Sélectionner | q Retour".to_string(),
        }
    };

    // Statut serveur à gauche
    use crate::app::ServerStatus;
    let (status_icon, status_color) = match app.server_status {
        ServerStatus::Online => ("● En ligne", Color::Green),
        ServerStatus::Offline => ("● Hors ligne", Color::Red),
        ServerStatus::Unknown => ("● Connexion…", Color::DarkGray),
    };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(16), Constraint::Min(1)])
        .split(area);

    let status = Paragraph::new(Line::from(Span::styled(
        format!(" {}", status_icon),
        Style::default().fg(status_color),
    )))
    .block(Block::default().borders(Borders::ALL));

    let footer = Paragraph::new(text)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(status, chunks[0]);
    frame.render_widget(footer, chunks[1]);
}

/// Dessine un placeholder pour les écrans non implémentés.
pub fn draw_placeholder(frame: &mut Frame, area: Rect, text: &str) {
    let p = Paragraph::new(text)
        .style(Style::default().fg(Color::DarkGray))
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(p, area);
}
