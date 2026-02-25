use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use monster_battle_core::types::ElementType;
use monster_battle_storage::MonsterStorage;

use super::common::draw_placeholder;
use crate::app::App;

/// Écran de sélection du mode + type de bot adverse pour l'entraînement.
pub fn draw_select(frame: &mut Frame, area: Rect, app: &App, wild: bool) {
    let monsters = app.storage.list_alive().unwrap_or_default();
    if monsters.is_empty() {
        draw_placeholder(frame, area, "Vous n'avez pas de monstre vivant !");
        return;
    }

    let m = &monsters[0];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Info monstre
            Constraint::Length(5), // Mode
            Constraint::Min(5),    // Liste des types
        ])
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

    // Mode d'entraînement
    let (mode_label, mode_color, mode_desc) = if wild {
        (
            "SAUVAGE",
            Color::Red,
            "100% XP — ⚠️ Défaite = mort du monstre",
        )
    } else {
        ("DOCILE", Color::Green, "50% XP — ✅ Pas de risque de mort")
    };

    let mode_block = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("  Mode : ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("◀ {} ▶", mode_label),
                Style::default().fg(mode_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  ({})", mode_desc),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(Span::styled(
            "  ←→ Changer de mode",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .block(
        Block::default()
            .title(" Mode d'entraînement ")
            .title_style(Style::default().fg(mode_color).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(mode_color)),
    );

    frame.render_widget(mode_block, chunks[1]);

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

    let title = if wild {
        " Choisir un adversaire (Entraînement sauvage — 100% XP) "
    } else {
        " Choisir un adversaire (Entraînement docile — 50% XP) "
    };

    let list = List::new(list_items).block(
        Block::default()
            .title(title)
            .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red)),
    );

    frame.render_widget(list, chunks[2]);
}
