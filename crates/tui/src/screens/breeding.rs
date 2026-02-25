use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use super::common::draw_placeholder;
use crate::app::App;

/// Sous-écrans de la reproduction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BreedPhase {
    /// Sélection du monstre à envoyer en reproduction.
    SelectMonster,
    /// Recherche d'un partenaire sur le serveur.
    Searching,
    /// Partenaire trouvé, échange en cours.
    Matched { opponent_name: String },
    /// Saisie du nom du bébé.
    NamingChild,
    /// Résultat de la reproduction.
    Result,
    /// Erreur.
    Error(String),
}

/// Écran de recherche d'un partenaire.
pub fn draw_searching(frame: &mut Frame, area: Rect, app: &App) {
    let text = format!(
        "⏳ Recherche d'un partenaire de reproduction...\n\n\
         Serveur : {}\n\n\
         La reproduction commencera automatiquement dès qu'un\n\
         autre joueur sera trouvé.\n\n\
         Appuyez sur Esc pour annuler.",
        app.server_address
    );

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::Magenta))
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(" 🧬 Reproduction ")
                .title_style(
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                )
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta)),
        );

    frame.render_widget(paragraph, area);
}

/// Écran de partenaire trouvé.
pub fn draw_matched(frame: &mut Frame, area: Rect, _app: &App, opponent_name: &str) {
    let text = format!(
        "🧬 Partenaire trouvé : {} !\n\n\
         Échange des données en cours...",
        opponent_name
    );

    let paragraph = Paragraph::new(text)
        .style(
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(" Reproduction ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta)),
        );

    frame.render_widget(paragraph, area);
}

/// Saisie du nom de l'enfant.
pub fn draw_naming_child(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(area);

    let help = Paragraph::new(
        "🧬 La reproduction a réussi !\n\
         Donnez un nom au nouveau monstre (max 20 caractères) :",
    )
    .style(Style::default().fg(Color::Magenta))
    .wrap(Wrap { trim: true })
    .block(
        Block::default()
            .title(" Nom du bébé ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta)),
    );

    frame.render_widget(help, chunks[0]);

    let cursor = if app.name_input_blink { "█" } else { " " };
    let input_text = format!("  {}{}", app.name_input, cursor);

    let input = Paragraph::new(input_text)
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .title(" Nom ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        );

    frame.render_widget(input, chunks[1]);
}

/// Résultat de la reproduction.
pub fn draw_breed_result(frame: &mut Frame, area: Rect, app: &App) {
    let log_text = app
        .breeding_log
        .iter()
        .map(|line| Line::from(Span::raw(line.clone())))
        .collect::<Vec<_>>();

    if log_text.is_empty() {
        draw_placeholder(frame, area, "Aucun résultat...");
        return;
    }

    let paragraph = Paragraph::new(log_text)
        .wrap(Wrap { trim: true })
        .scroll((app.scroll_offset as u16, 0))
        .block(
            Block::default()
                .title(" Résultat de la reproduction ")
                .title_style(
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                )
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta)),
        );

    frame.render_widget(paragraph, area);
}

/// Dessine l'écran de sélection du monstre pour la reproduction.
pub fn draw_select_monster(frame: &mut Frame, area: Rect, app: &App) {
    super::pvp::draw_monster_selection(
        frame,
        area,
        app,
        " 🧬 Choisir un monstre — Reproduction ",
        Color::Magenta,
    );
}

/// Erreur réseau.
pub fn draw_error(frame: &mut Frame, area: Rect, error: &str) {
    let text = format!(
        "❌ Erreur :\n\n{}\n\n\
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
