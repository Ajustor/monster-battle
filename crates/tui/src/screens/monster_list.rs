use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use monster_battle_storage::MonsterStorage;

use crate::app::App;
use super::common::draw_placeholder;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let monsters = app.storage.list_alive().unwrap_or_default();

    if monsters.is_empty() {
        draw_placeholder(frame, area, "Aucun monstre vivant. Créez-en un depuis le menu principal !");
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
        m.traits.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(", ")
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
    let hp_color = if hp_pct > hp_bar_width / 2 { "💚" } else if hp_pct > hp_bar_width / 4 { "💛" } else { "❤️" };
    let hp_bar = format!(
        "{} [{}{}] {}/{}",
        hp_color,
        "█".repeat(hp_pct),
        "░".repeat(hp_bar_width - hp_pct),
        m.current_hp,
        m.max_hp()
    );

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
        type_icon, m.name, secondary,
        m.level,
        xp_bar,
        hp_bar,
        m.effective_attack(), m.effective_defense(),
        m.effective_speed(), m.base_stats.special_attack,
        m.base_stats.special_defense,
        traits_str,
        m.age_days(), m.max_age_days(),
        m.generation,
        m.wins, m.losses,
    );

    let detail = Paragraph::new(info)
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .title(format!(" {} — Fiche Monstre ", m.name))
                .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        );

    frame.render_widget(detail, area);
}
