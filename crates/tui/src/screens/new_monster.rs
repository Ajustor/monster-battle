use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

use monster_battle_core::types::ElementType;

use crate::app::App;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let types = ElementType::all();

    let list_items: Vec<ListItem> = types
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let style = if i == app.menu_index % types.len() {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else {
                Style::default().fg(Color::White)
            };

            let line = format!("  {} {}  ", t.icon(), t);
            ListItem::new(Line::from(Span::styled(line, style)))
        })
        .collect();

    let list = List::new(list_items).block(
        Block::default()
            .title(" Choisir un type de starter ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta)),
    );

    frame.render_widget(list, area);
}
