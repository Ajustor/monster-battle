use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Flex, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

/// Dessine la modale de mise à jour par-dessus tout.
pub fn draw(frame: &mut Frame, server_version: Option<&str>) {
    let area = frame.area();

    // Centrer la modale (56x14)
    let [modal_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(14)])
        .flex(Flex::Center)
        .areas(area);
    let [modal_area] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(56)])
        .flex(Flex::Center)
        .areas(modal_area);

    // Effacer le fond
    frame.render_widget(Clear, modal_area);

    let client_version = env!("CARGO_PKG_VERSION");
    let sv = server_version.unwrap_or("?");

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  ⚠️  Mise à jour requise !",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Votre version  : ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                client_version,
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Version serveur : ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                sv.to_string(),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Téléchargez la dernière version :",
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            "  https://ajustor.github.io/monster-battle",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::UNDERLINED),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  Appuyez sur Enter pour ouvrir le lien…",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false }).block(
        Block::default()
            .title(" 🔄 Mise à jour ")
            .title_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );

    frame.render_widget(paragraph, modal_area);
}
