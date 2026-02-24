use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use monster_battle_storage::MonsterStorage;

use super::common::draw_placeholder;
use crate::app::App;

/// Sous-écrans du combat PvP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PvpPhase {
    /// Menu : Héberger ou Rejoindre.
    Menu,
    /// En attente de connexion (hôte).
    WaitingForOpponent,
    /// Saisie de l'adresse IP (client).
    EnterAddress,
    /// En cours de connexion au serveur.
    Connecting,
    /// En attente que l'adversaire accepte.
    WaitingForAccept,
    /// Proposition reçue — accepter ?
    ReceivedChallenge { opponent_name: String },
    /// Combat en cours (on attend le résultat).
    Fighting,
    /// Résultat du combat PvP (log).
    Result,
    /// Erreur réseau.
    Error(String),
}

/// Dessine le menu du combat PvP (héberger / rejoindre).
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

    // Info du monstre du joueur
    let player_info = Paragraph::new(Line::from(vec![
        Span::styled("  Votre monstre : ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!(
                "{} {} — Nv.{} — PV {}/{}",
                m.primary_type.icon(),
                m.name,
                m.level,
                m.current_hp,
                m.max_hp()
            ),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(player_info, chunks[0]);

    let items = vec![
        "🖥️  Héberger une partie (attendre un adversaire)",
        "🔗 Rejoindre une partie (entrer l'adresse IP)",
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
            .title(" ⚔️  Combat PvP ")
            .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red)),
    );

    frame.render_widget(list, chunks[1]);
}

/// Dessine l'écran d'attente de connexion (hôte).
pub fn draw_waiting(frame: &mut Frame, area: Rect, app: &App) {
    let port = app.pvp_port;
    let ips_display = app
        .local_ips
        .iter()
        .map(|ip| format!("  📡 {}:{}", ip, port))
        .collect::<Vec<_>>()
        .join("\n");

    let text = format!(
        "⏳ En attente d'un adversaire sur le port {}...\n\n\
         Votre adresse :\n{}\n\n\
         Communiquez une de ces adresses à votre adversaire.\n\n\
         Appuyez sur Esc pour annuler.",
        port, ips_display
    );

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::Yellow))
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(" Hébergement ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        );

    frame.render_widget(paragraph, area);
}

/// Dessine l'écran de saisie d'adresse IP (client).
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
        "Entrez l'adresse IP et le port du joueur hôte.\n\
         Format : adresse_ip:port  (ex: 192.168.1.42:7878)",
    )
    .style(Style::default().fg(Color::White))
    .wrap(Wrap { trim: true })
    .block(
        Block::default()
            .title(" Rejoindre une partie ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(help, chunks[0]);

    // Champ de saisie
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

/// Dessine l'écran de connexion en cours.
pub fn draw_connecting(frame: &mut Frame, area: Rect, _app: &App) {
    let text = "🔗 Connexion en cours...\n\nAppuyez sur Esc pour annuler.";

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::Cyan))
        .wrap(Wrap { trim: true })
        .block(Block::default().title(" Connexion ").borders(Borders::ALL));

    frame.render_widget(paragraph, area);
}

/// Dessine l'écran de challenge reçu.
pub fn draw_received_challenge(frame: &mut Frame, area: Rect, _app: &App, opponent_name: &str) {
    let text = format!(
        "⚔️  {} vous défie en combat !\n\n\
         Appuyez sur Enter pour accepter\n\
         Appuyez sur Esc pour refuser",
        opponent_name
    );

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(" Défi reçu ! ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red)),
        );

    frame.render_widget(paragraph, area);
}

/// Dessine l'écran de combat en cours.
pub fn draw_fighting(frame: &mut Frame, area: Rect, _app: &App) {
    let text = "⚔️  Combat en cours...\n\nVeuillez patienter.";

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::Red))
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(" Combat PvP ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red)),
        );

    frame.render_widget(paragraph, area);
}

/// Dessine le résultat du combat PvP.
pub fn draw_pvp_result(frame: &mut Frame, area: Rect, app: &App) {
    let log_text = app
        .pvp_log
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
                .title(" Résultat du combat PvP ")
                .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
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
