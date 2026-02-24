use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use monster_battle_storage::MonsterStorage;

use crate::app::App;
use super::common::draw_placeholder;

/// Sous-écrans de la reproduction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BreedPhase {
    /// Menu : Héberger ou Rejoindre.
    Menu,
    /// En attente d'un partenaire (hôte).
    WaitingForPartner,
    /// Saisie d'adresse (client).
    EnterAddress,
    /// En cours de connexion.
    Connecting,
    /// Proposition de reproduction reçue.
    ReceivedProposal { partner_monster_name: String },
    /// En attente de réponse de l'autre joueur.
    WaitingForAccept,
    /// Saisie du nom du bébé.
    NamingChild,
    /// Résultat de la reproduction.
    Result,
    /// Erreur.
    Error(String),
}

/// Menu de la reproduction.
pub fn draw_menu(frame: &mut Frame, area: Rect, app: &App) {
    let monsters = app.storage.list_alive().unwrap_or_default();
    if monsters.is_empty() {
        draw_placeholder(frame, area, "Vous n'avez pas de monstre vivant !");
        return;
    }

    let m = &monsters[0];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(5)])
        .split(area);

    let player_info = Paragraph::new(Line::from(vec![
        Span::styled("  Votre monstre : ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!(
                "{} {} — Nv.{} — Gén.{}",
                m.primary_type.icon(),
                m.name,
                m.level,
                m.generation
            ),
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(player_info, chunks[0]);

    let items = vec![
        "🖥️  Héberger (attendre un partenaire)",
        "🔗 Rejoindre (entrer l'adresse IP)",
    ];

    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == app.menu_index % items.len() {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(Span::styled(format!("  {}  ", item), style)))
        })
        .collect();

    let list = List::new(list_items).block(
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

    frame.render_widget(list, chunks[1]);
}

/// Écran d'attente d'un partenaire (hôte).
pub fn draw_waiting(frame: &mut Frame, area: Rect, app: &App) {
    let port = app.pvp_port;
    let ips_display = app
        .local_ips
        .iter()
        .map(|ip| format!("  📡 {}:{}", ip, port))
        .collect::<Vec<_>>()
        .join("\n");

    let text = format!(
        "⏳ En attente d'un partenaire sur le port {}...\n\n\
         Votre adresse :\n{}\n\n\
         Communiquez une de ces adresses à l'autre joueur.\n\n\
         Appuyez sur Esc pour annuler.",
        port, ips_display
    );

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::Magenta))
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(" Hébergement — Reproduction ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta)),
        );

    frame.render_widget(paragraph, area);
}

/// Écran de saisie d'adresse (client).
pub fn draw_enter_address(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(area);

    let help = Paragraph::new(
        "Entrez l'adresse IP et le port de l'autre joueur.\n\
         Format : adresse_ip:port  (ex: 192.168.1.42:7878)",
    )
    .style(Style::default().fg(Color::White))
    .wrap(Wrap { trim: true })
    .block(
        Block::default()
            .title(" Rejoindre — Reproduction ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta)),
    );

    frame.render_widget(help, chunks[0]);

    let cursor = if app.name_input_blink { "█" } else { " " };
    let input_text = format!("  {}{}", app.pvp_address_input, cursor);

    let input = Paragraph::new(input_text)
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .title(" Adresse ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        );

    frame.render_widget(input, chunks[1]);
}

/// Écran de connexion en cours.
pub fn draw_connecting(frame: &mut Frame, area: Rect, _app: &App) {
    let text = "🔗 Connexion en cours...\n\nAppuyez sur Esc pour annuler.";

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::Magenta))
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(" Connexion ")
                .borders(Borders::ALL),
        );

    frame.render_widget(paragraph, area);
}

/// Proposition reçue.
pub fn draw_received_proposal(
    frame: &mut Frame,
    area: Rect,
    _app: &App,
    partner_monster_name: &str,
) {
    let text = format!(
        "🧬 L'autre joueur propose de faire reproduire son monstre : {}\n\n\
         Appuyez sur Enter pour accepter\n\
         Appuyez sur Esc pour refuser",
        partner_monster_name
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
                .title(" Proposition de reproduction ")
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
