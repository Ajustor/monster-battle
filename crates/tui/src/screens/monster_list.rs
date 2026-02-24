use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use monster_battle_core::types::ElementType;
use monster_battle_storage::MonsterStorage;

use super::common::draw_placeholder;
use crate::app::App;
use crate::sprites;

fn element_color(element: ElementType) -> Color {
    match element {
        ElementType::Normal => Color::Gray,
        ElementType::Fire => Color::Red,
        ElementType::Water => Color::Blue,
        ElementType::Plant => Color::Green,
        ElementType::Electric => Color::Yellow,
        ElementType::Earth => Color::Rgb(180, 120, 60),
        ElementType::Wind => Color::Cyan,
        ElementType::Shadow => Color::Magenta,
        ElementType::Light => Color::White,
    }
}

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let monsters = app.storage.list_alive().unwrap_or_default();

    if monsters.is_empty() {
        draw_placeholder(
            frame,
            area,
            "Aucun monstre vivant. Créez-en un depuis le menu principal !",
        );
        return;
    }

    // On n'a qu'un seul monstre, affichons ses détails
    let m = &monsters[0];

    let type_icon = m.primary_type.icon();
    let secondary = m
        .secondary_type
        .map(|t| format!(" / {} {}", t.icon(), t))
        .unwrap_or_default();

    let traits_str = if m.traits.is_empty() {
        "Aucun".to_string()
    } else {
        m.traits
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    };

    let xp_bar_width = 20;
    let xp_pct = if m.xp_to_next_level() > 0 {
        (m.xp as f64 / m.xp_to_next_level() as f64 * xp_bar_width as f64) as usize
    } else {
        0
    };
    let xp_bar = format!(
        "[{}{}] {}/{}",
        "█".repeat(xp_pct),
        "░".repeat(xp_bar_width - xp_pct),
        m.xp,
        m.xp_to_next_level()
    );

    let hp_bar_width = 20;
    let hp_pct = (m.current_hp as f64 / m.max_hp() as f64 * hp_bar_width as f64) as usize;
    let hp_color = if hp_pct > hp_bar_width / 2 {
        "💚"
    } else if hp_pct > hp_bar_width / 4 {
        "💛"
    } else {
        "❤️"
    };
    let hp_bar = format!(
        "{} [{}{}] {}/{}",
        hp_color,
        "█".repeat(hp_pct),
        "░".repeat(hp_bar_width - hp_pct),
        m.current_hp,
        m.max_hp()
    );

    let outer_block = Block::default()
        .title(format!(" {} — Fiche Monstre ", m.name))
        .title_style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));
    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    // Layout : sprite (gauche) + stats (droite)
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(18), Constraint::Min(30)])
        .split(inner);

    // ── Sprite ──────────────────────────────────────────────────
    let sprite_color = element_color(m.primary_type);
    let art = sprites::get_sprite(m.primary_type, m.secondary_type);
    let mut sprite_lines: Vec<Line> = vec![Line::from("")];
    for line in &art {
        sprite_lines.push(Line::from(Span::styled(
            *line,
            Style::default().fg(sprite_color),
        )));
    }
    sprite_lines.push(Line::from(""));
    sprite_lines.push(Line::from(Span::styled(
        format!("  {} {}", type_icon, m.primary_type),
        Style::default()
            .fg(sprite_color)
            .add_modifier(Modifier::BOLD),
    )));
    if let Some(sec) = m.secondary_type {
        sprite_lines.push(Line::from(Span::styled(
            format!("  {} {}", sec.icon(), sec),
            Style::default().fg(element_color(sec)),
        )));
    }
    let sprite_widget = Paragraph::new(sprite_lines).block(
        Block::default()
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(sprite_widget, cols[0]);

    // ── Stats ───────────────────────────────────────────────────
    let info = format!(
        r#"
  {} {}{}
  Niveau : {}
  XP     : {}
  PV     : {}

  ── Stats ──────────────
  ATK : {}   DEF : {}
  SPD : {}   S.ATK : {}
  S.DEF : {}

  ── Infos ──────────────
  Traits     : {}
  Âge        : {} / {} jours
  Génération : {}
  Victoires  : {}  |  Défaites : {}
"#,
        type_icon,
        m.name,
        secondary,
        m.level,
        xp_bar,
        hp_bar,
        m.effective_attack(),
        m.effective_defense(),
        m.effective_speed(),
        m.base_stats.special_attack,
        m.base_stats.special_defense,
        traits_str,
        m.age_days(),
        m.max_age_days(),
        m.generation,
        m.wins,
        m.losses,
    );

    let detail = Paragraph::new(info)
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: false });

    frame.render_widget(detail, cols[1]);
}
