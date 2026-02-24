use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

use monster_battle_storage::MonsterStorage;

use super::common::draw_placeholder;
use crate::app::App;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let dead = app.storage.list_dead().unwrap_or_default();

    if dead.is_empty() {
        draw_placeholder(
            frame,
            area,
            "Le cimetière est vide. Vos monstres sont en sécurité... pour l'instant.",
        );
        return;
    }

    let list_items: Vec<ListItem> = dead
        .iter()
        .map(|m| {
            let line = format!(
                "  💀 {} {} — Nv.{} — Vécu {}j — {} victoires  ",
                m.primary_type.icon(),
                m.name,
                m.level,
                m.age_days(),
                m.wins,
            );
            ListItem::new(Line::from(Span::styled(
                line,
                Style::default().fg(Color::DarkGray),
            )))
        })
        .collect();

    let list = List::new(list_items).block(
        Block::default()
            .title(format!(" Cimetière ({}) ", dead.len()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(list, area);
}
