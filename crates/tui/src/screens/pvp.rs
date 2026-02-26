use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::App;

/// Sous-écrans du combat PvP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PvpPhase {
    /// Recherche d'un adversaire sur le serveur.
    Searching,
    /// Adversaire trouvé, combat en cours.
    Matched { opponent_name: String },
    /// Erreur réseau.
    Error(String),
}

/// Dessine l'écran de recherche d'adversaire.
pub fn draw_searching(frame: &mut Frame, area: Rect, app: &App) {
    let text = format!(
        "⏳ Recherche d'un adversaire...\n\n\
         Serveur : {}\n\n\
         Le combat commencera automatiquement dès qu'un\n\
         autre joueur sera trouvé.\n\n\
         Appuyez sur Esc pour annuler.",
        app.server_address
    );

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::Yellow))
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(" ⚔️  Combat PvP ")
                .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red)),
        );

    frame.render_widget(paragraph, area);
}

/// Dessine l'écran de combat en cours (adversaire trouvé).
pub fn draw_matched(frame: &mut Frame, area: Rect, _app: &App, opponent_name: &str) {
    let text = format!(
        "⚔️  Adversaire trouvé : {} !\n\n\
         Combat en cours... Veuillez patienter.",
        opponent_name
    );

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(" Combat PvP ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red)),
        );

    frame.render_widget(paragraph, area);
}

/// Dessine un écran d'erreur réseau.
pub fn draw_error(frame: &mut Frame, area: Rect, error: &str) {
    let text = format!(
        "❌ Erreur réseau :\n\n{}\n\n\
         Appuyez sur Enter ou Esc pour revenir.",
        error
    );

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::Red))
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(" Erreur ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red)),
        );

    frame.render_widget(paragraph, area);
}
