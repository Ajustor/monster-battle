use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

use crate::app::App;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let has_monster = app.has_living_monster();

    let mut items: Vec<&str> = Vec::new();
    items.push("🐾 Mon Monstre");
    if !has_monster {
        items.push("🥚 Nouveau Monstre");
    }
    if has_monster {
        items.push("⚔️  Entraînement");
        items.push("🗡️  Combat PvP");
        items.push("🧬 Reproduction");
    }
    items.push("💀 Cimetière");
    items.push("🚪 Quitter");

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
            .title(" Menu Principal ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)),
    );

    frame.render_widget(list, area);
}
