use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use monster_battle_core::battle::{
    AnimationType, BattlePhase, BattleState, MessageStyle,
};
use monster_battle_core::types::ElementType;

use super::common::draw_placeholder;
use crate::app::App;

// ── Couleur par élément ─────────────────────────────────────────────

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

// ── Point d'entrée ──────────────────────────────────────────────────

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let battle = match &app.battle_state {
        Some(b) => b,
        None => {
            draw_placeholder(frame, area, "Aucun combat en cours...");
            return;
        }
    };

    // Layout : terrain de combat + panneau d'actions
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(9)])
        .split(area);

    draw_battlefield(frame, chunks[0], battle);
    draw_action_panel(frame, chunks[1], battle);
}

// ── Terrain de combat ───────────────────────────────────────────────

fn draw_battlefield(frame: &mut Frame, area: Rect, battle: &BattleState) {
    let block = Block::default()
        .title(" ⚔️  Combat ")
        .title_style(
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Moitié haute : adversaire — moitié basse : joueur
    let halves = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    draw_opponent_side(frame, halves[0], battle);
    draw_player_side(frame, halves[1], battle);
}

fn draw_opponent_side(frame: &mut Frame, area: Rect, battle: &BattleState) {
    let opp = &battle.opponent;

    // Colonnes : sprite (gauche) + info (droite)
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // ── Sprite adversaire ───────────────────────────────────────
    let is_hit = matches!(battle.anim_type, Some(AnimationType::OpponentHit));
    let sprite_color = if is_hit {
        Color::Red
    } else {
        element_color(opp.element)
    };

    let sprite_lines = if is_hit && battle.anim_frame % 2 == 0 {
        // Flash : sprite disparaît brièvement
        vec![
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                "       💥",
                Style::default().fg(Color::Red),
            )),
        ]
    } else {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "      ╔════╗",
                Style::default().fg(sprite_color),
            )),
            Line::from(Span::styled(
                format!("      ║ {} ║", opp.element.icon()),
                Style::default().fg(sprite_color),
            )),
            Line::from(Span::styled(
                "      ╚════╝",
                Style::default().fg(sprite_color),
            )),
        ]
    };
    frame.render_widget(Paragraph::new(sprite_lines), cols[0]);

    // ── Info adversaire ─────────────────────────────────────────
    let bar_width = (cols[1].width as usize).saturating_sub(8).min(20);
    let hp_bar = hp_bar_spans(opp.display_hp, opp.max_hp, bar_width);

    let type_str = match opp.secondary_element {
        Some(sec) => format!("{}/{}", opp.element, sec),
        None => format!("{}", opp.element),
    };

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!(" {} {} ", opp.element.icon(), opp.name),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("Nv.{}", opp.level),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(format!(" ({})", type_str), Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(hp_bar),
    ];

    let info = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(info, cols[1]);
}

fn draw_player_side(frame: &mut Frame, area: Rect, battle: &BattleState) {
    let p = &battle.player;

    // Colonnes : info (gauche) + sprite (droite)
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    // ── Info joueur ─────────────────────────────────────────────
    let bar_width = (cols[0].width as usize).saturating_sub(8).min(20);
    let hp_bar = hp_bar_spans(p.display_hp, p.max_hp, bar_width);

    let type_str = match p.secondary_element {
        Some(sec) => format!("{}/{}", p.element, sec),
        None => format!("{}", p.element),
    };

    let lines = vec![
        Line::from(vec![
            Span::styled(
                format!(" {} {} ", p.element.icon(), p.name),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("Nv.{}", p.level),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(format!(" ({})", type_str), Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(hp_bar),
    ];

    let info = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    frame.render_widget(info, cols[0]);

    // ── Sprite joueur ───────────────────────────────────────────
    let is_hit = matches!(battle.anim_type, Some(AnimationType::PlayerHit));
    let sprite_color = if is_hit {
        Color::Red
    } else {
        element_color(p.element)
    };

    let sprite_lines = if is_hit && battle.anim_frame % 2 == 0 {
        vec![
            Line::from(Span::styled(
                "  💥",
                Style::default().fg(Color::Red),
            )),
            Line::from(""),
            Line::from(""),
        ]
    } else {
        vec![
            Line::from(Span::styled(
                "  ╔════╗",
                Style::default().fg(sprite_color),
            )),
            Line::from(Span::styled(
                format!("  ║ {} ║", p.element.icon()),
                Style::default().fg(sprite_color),
            )),
            Line::from(Span::styled(
                "  ╚════╝",
                Style::default().fg(sprite_color),
            )),
        ]
    };
    frame.render_widget(Paragraph::new(sprite_lines), cols[1]);
}

// ── Barre de PV ─────────────────────────────────────────────────────

fn hp_bar_spans(current: u32, max: u32, width: usize) -> Vec<Span<'static>> {
    let pct = if max == 0 {
        0.0
    } else {
        current as f64 / max as f64
    };
    let filled = (pct * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);

    let color = if pct > 0.5 {
        Color::Green
    } else if pct > 0.25 {
        Color::Yellow
    } else {
        Color::Red
    };

    vec![
        Span::styled(
            " PV ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("█".repeat(filled), Style::default().fg(color)),
        Span::styled(
            "░".repeat(empty),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            format!(" {}/{}", current, max),
            Style::default().fg(Color::White),
        ),
    ]
}

// ── Panneau d'actions ───────────────────────────────────────────────

fn draw_action_panel(frame: &mut Frame, area: Rect, battle: &BattleState) {
    match &battle.phase {
        BattlePhase::PlayerChooseAttack => draw_attack_menu(frame, area, battle),
        _ => draw_message_panel(frame, area, battle),
    }
}

fn draw_attack_menu(frame: &mut Frame, area: Rect, battle: &BattleState) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(area);

    // ── Liste d'attaques (gauche) ───────────────────────────────
    let attacks = &battle.player.attacks;
    let selected = battle.attack_menu_index;

    let mut lines = vec![Line::from(Span::styled(
        format!(" Que doit faire {} ?", battle.player.name),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ))];

    for (i, atk) in attacks.iter().enumerate() {
        let prefix = if i == selected { " ▶ " } else { "   " };
        let style = if i == selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        lines.push(Line::from(Span::styled(
            format!("{}{} {}", prefix, atk.element.icon(), atk.name),
            style,
        )));
    }

    let list = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(" Attaques "),
    );
    frame.render_widget(list, cols[0]);

    // ── Détails de l'attaque sélectionnée (droite) ──────────────
    if selected < attacks.len() {
        let atk = &attacks[selected];
        let eff = atk
            .element
            .effectiveness_against(&battle.opponent.element);
        let eff_text = if eff > 1.2 {
            "💥 Super efficace !"
        } else if eff < 0.8 {
            "🔽 Pas très efficace..."
        } else {
            "➖ Efficacité normale"
        };
        let eff_color = if eff > 1.2 {
            Color::Green
        } else if eff < 0.8 {
            Color::Red
        } else {
            Color::White
        };
        let category = if atk.is_special {
            "Spéciale ✨"
        } else {
            "Physique 💪"
        };

        let detail_lines = vec![
            Line::from(Span::styled(
                format!(" {} {}", atk.element.icon(), atk.name),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                format!(" Puissance : {}", atk.power),
                Style::default().fg(Color::White),
            )),
            Line::from(Span::styled(
                format!(" Précision : {}%", atk.accuracy),
                Style::default().fg(Color::White),
            )),
            Line::from(Span::styled(
                format!(" Catégorie : {}", category),
                Style::default().fg(Color::White),
            )),
            Line::from(Span::styled(
                format!(" Type : {} {}", atk.element.icon(), atk.element),
                Style::default().fg(element_color(atk.element)),
            )),
            Line::from(Span::styled(
                format!(" {}", eff_text),
                Style::default().fg(eff_color),
            )),
        ];

        let details = Paragraph::new(detail_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(" Détails "),
        );
        frame.render_widget(details, cols[1]);
    }
}

fn draw_message_panel(frame: &mut Frame, area: Rect, battle: &BattleState) {
    let msg_text = match &battle.current_message {
        Some(msg) => msg.text.clone(),
        None => String::new(),
    };

    let style = match battle.current_message.as_ref().map(|m| &m.style) {
        Some(MessageStyle::Victory) => Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
        Some(MessageStyle::Defeat) => Style::default()
            .fg(Color::Red)
            .add_modifier(Modifier::BOLD),
        Some(MessageStyle::Critical) | Some(MessageStyle::SuperEffective) => Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
        Some(MessageStyle::NotEffective) => Style::default().fg(Color::DarkGray),
        Some(MessageStyle::PlayerAttack) => Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
        Some(MessageStyle::OpponentAttack) => Style::default()
            .fg(Color::Red)
            .add_modifier(Modifier::BOLD),
        Some(MessageStyle::Damage) => Style::default().fg(Color::White),
        Some(MessageStyle::Heal) => Style::default().fg(Color::Green),
        _ => Style::default().fg(Color::White),
    };

    let continue_hint = if battle.is_over() {
        " Enter pour terminer..."
    } else {
        " Enter pour continuer..."
    };

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(format!("  {}", msg_text), style)),
        Line::from(""),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            continue_hint,
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White)),
        );
    frame.render_widget(paragraph, area);
}
