use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use monster_battle_core::types::ElementType;
use monster_battle_storage::MonsterStorage;

use crate::app::App;
use super::common::draw_placeholder;

/// Écran de sélection du type de bot adverse pour l'entraînement.
pub fn draw_select(frame: &mut Frame, area: Rect, app: &App) {
    let monsters = app.storage.list_alive().unwrap_or_default();
    if monsters.is_empty() {
        draw_placeholder(frame, area, "Vous n'avez pas de monstre vivant !");
        return;
    }

    let m = &monsters[0];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
        ])
        .split(area);

    // Info du monstre du joueur
    let player_info = Paragraph::new(Line::from(vec![
        Span::styled("  Votre monstre : ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{} {} — Nv.{} — PV {}/{}", m.primary_type.icon(), m.name, m.level, m.current_hp, m.max_hp()),
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(player_info, chunks[0]);

    // Liste des types adverses
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

            let effectiveness = m.primary_type.effectiveness_against(t);
            let indicator = if effectiveness > 1.0 {
                " ✅ avantagé"
            } else if effectiveness < 1.0 {
                " ❌ désavantagé"
            } else {
                ""
            };

            let line = format!("  {} Bot {}{}", t.icon(), t, indicator);
            ListItem::new(Line::from(Span::styled(line, style)))
        })
        .collect();

    let list = List::new(list_items).block(
        Block::default()
            .title(" Choisir un adversaire (Entraînement — 50% XP) ")
            .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red)),
    );

    frame.render_widget(list, chunks[1]);
}

/// Écran du résultat d'un combat d'entraînement.
pub fn draw_result(frame: &mut Frame, area: Rect, app: &App) {
    let log_text = app
        .training_log
        .iter()
        .map(|line| Line::from(Span::raw(line.clone())))
        .collect::<Vec<_>>();

    if log_text.is_empty() {
        draw_placeholder(frame, area, "Aucun combat...");
        return;
    }

    let paragraph = Paragraph::new(log_text)
        .wrap(Wrap { trim: true })
        .scroll((app.scroll_offset as u16, 0))
        .block(
            Block::default()
                .title(" Résultat du combat d'entraînement ")
                .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red)),
        );

    frame.render_widget(paragraph, area);
}
