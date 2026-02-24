use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use monster_battle_core::types::ElementType;

use crate::app::App;

pub fn draw(frame: &mut Frame, area: Rect, app: &App, type_index: usize) {
    let types = ElementType::all();
    let chosen = types[type_index % types.len()];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Min(0),
        ])
        .split(area);

    // Info type choisi
    let type_info = Paragraph::new(Line::from(vec![
        Span::styled("  Type choisi : ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{} {}", chosen.icon(), chosen),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(type_info, chunks[0]);

    // Champ de saisie du nom
    let cursor = if app.name_input_blink { "█" } else { " " };
    let input_text = format!("  {} {}", app.name_input, cursor);

    let input = Paragraph::new(Line::from(vec![
        Span::styled(input_text, Style::default().fg(Color::White)),
    ]))
    .block(
        Block::default()
            .title(" Nom de votre monstre ")
            .title_style(
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta)),
    );

    frame.render_widget(input, chunks[1]);

    // Indication
    let hint = Paragraph::new("  Choisissez un nom unique pour votre compagnon !")
        .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(hint, chunks[2]);
}
