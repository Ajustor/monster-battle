use ratatui::{
    Frame,
    layout::Rect,
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
            Screen::TrainingResult | Screen::CombatResult | Screen::BreedingResult => {
                "↑↓ Défiler | Enter ou q pour revenir".to_string()
            }
            Screen::Combat(phase) => match phase {
                crate::screens::pvp::PvpPhase::Menu => {
                    "↑↓ Naviguer | Enter Sélectionner | Esc Retour".to_string()
                }
                crate::screens::pvp::PvpPhase::EnterAddress => {
                    "Entrez l'adresse | Enter Valider | Esc Annuler".to_string()
                }
                crate::screens::pvp::PvpPhase::ReceivedChallenge { .. } => {
                    "Enter Accepter | Esc Refuser".to_string()
                }
                crate::screens::pvp::PvpPhase::Result => {
                    "↑↓ Défiler | Enter ou q pour revenir".to_string()
                }
                _ => "Esc Annuler".to_string(),
            },
            Screen::Breeding(phase) => match phase {
                crate::screens::breeding::BreedPhase::Menu => {
                    "↑↓ Naviguer | Enter Sélectionner | Esc Retour".to_string()
                }
                crate::screens::breeding::BreedPhase::EnterAddress => {
                    "Entrez l'adresse | Enter Valider | Esc Annuler".to_string()
                }
                crate::screens::breeding::BreedPhase::NamingChild => {
                    "Tapez le nom du bébé | Enter Valider | Esc Annuler".to_string()
                }
                crate::screens::breeding::BreedPhase::ReceivedProposal { .. } => {
                    "Enter Accepter | Esc Refuser".to_string()
                }
                crate::screens::breeding::BreedPhase::Result => {
                    "↑↓ Défiler | Enter ou q pour revenir".to_string()
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

    let footer = Paragraph::new(text)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(footer, area);
}

/// Dessine un placeholder pour les écrans non implémentés.
pub fn draw_placeholder(frame: &mut Frame, area: Rect, text: &str) {
    let p = Paragraph::new(text)
        .style(Style::default().fg(Color::DarkGray))
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(p, area);
}
