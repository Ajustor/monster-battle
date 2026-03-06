//! Rendu du jeu Memory en TUI.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use monster_battle_core::minigame::memory::{CardState, MemoryGame};

/// Écran principal du Memory en cours de partie.
pub fn draw_memory(frame: &mut Frame, area: Rect, game: &MemoryGame, monster_name: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(area);

    // ── Info header ─────────────────────────────────
    let status = if game.is_over() {
        let res = game.result.map_or("?", |r| match r {
            monster_battle_core::minigame::MinigameResult::Win => "Victoire !",
            monster_battle_core::minigame::MinigameResult::Draw => "Correct.",
            monster_battle_core::minigame::MinigameResult::Loss => "Trop de tentatives...",
        });
        res.to_string()
    } else if game.needs_dismiss {
        "Pas de paire ! Entrée pour continuer".to_string()
    } else {
        format!(
            "Paires : {}/{} — Tentatives : {}",
            game.pairs_found, game.total_pairs, game.attempts
        )
    };

    let info = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("  🃏 Memory ({}) — ", game.difficulty.label()),
            Style::default().fg(Color::Cyan),
        ),
        Span::styled(
            &status,
            Style::default()
                .fg(if game.is_over() {
                    match game.result {
                        Some(monster_battle_core::minigame::MinigameResult::Win) => Color::Green,
                        Some(monster_battle_core::minigame::MinigameResult::Draw) => Color::Yellow,
                        Some(monster_battle_core::minigame::MinigameResult::Loss) => Color::Red,
                        _ => Color::White,
                    }
                } else {
                    Color::White
                })
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  — {}", monster_name),
            Style::default().fg(Color::DarkGray),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(info, chunks[0]);

    // ── Board ───────────────────────────────────────
    draw_memory_board(frame, chunks[1], game);

    // ── Controls ────────────────────────────────────
    let controls = if game.is_over() {
        let reward = game.reward();
        if reward.is_empty() {
            "  Entrée : retour au menu".to_string()
        } else {
            format!("  Entrée : récolter les récompenses ({})", reward.summary())
        }
    } else if game.needs_dismiss {
        "  Entrée : cacher les cartes │ Esc : abandonner".to_string()
    } else {
        "  ↑↓←→ : déplacer │ Entrée : retourner │ Esc : abandonner".to_string()
    };

    let footer = Paragraph::new(Line::from(Span::styled(
        controls,
        Style::default().fg(Color::DarkGray),
    )))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, chunks[2]);
}

fn draw_memory_board(frame: &mut Frame, area: Rect, game: &MemoryGame) {
    let cell_w: u16 = 4;
    let cell_h: u16 = 2;
    let board_w = game.cols as u16 * (cell_w + 1) - 1;
    let board_h = game.rows as u16 * (cell_h + 1) - 1;

    let x = area.x + area.width.saturating_sub(board_w) / 2;
    let y = area.y + area.height.saturating_sub(board_h) / 2;

    let mut lines: Vec<Line> = Vec::new();

    for row in 0..game.rows {
        let mut top_spans: Vec<Span> = Vec::new();
        let mut bot_spans: Vec<Span> = Vec::new();

        for col in 0..game.cols {
            let idx = row * game.cols + col;
            let is_cursor = idx == game.cursor && !game.is_over();

            let (fg, bg) = match game.states[idx] {
                CardState::Matched => (Color::Black, Color::Green),
                CardState::Revealed => {
                    if is_cursor {
                        (Color::Yellow, Color::Magenta)
                    } else {
                        (Color::White, Color::Magenta)
                    }
                }
                CardState::Hidden => {
                    if is_cursor {
                        (Color::Yellow, Color::DarkGray)
                    } else {
                        (Color::Gray, Color::Reset)
                    }
                }
            };

            let style = Style::default().fg(fg).bg(bg);
            let display = if game.is_visible(idx) {
                format!(" {} ", game.card_icon(idx))
            } else {
                " ?? ".to_string()
            };

            top_spans.push(Span::styled(
                format!("{:>cell_w$}", "", cell_w = cell_w as usize),
                style,
            ));
            bot_spans.push(Span::styled(display, style));

            if col < game.cols - 1 {
                let sep = Style::default().fg(Color::DarkGray);
                top_spans.push(Span::styled(" ", sep));
                bot_spans.push(Span::styled(" ", sep));
            }
        }

        lines.push(Line::from(top_spans));
        lines.push(Line::from(bot_spans));

        if row < game.rows - 1 {
            let sep_line = "─".repeat(board_w as usize);
            lines.push(Line::from(Span::styled(
                sep_line,
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    let rect = Rect::new(x, y, board_w, board_h);
    let board = Paragraph::new(lines)
        .alignment(Alignment::Left)
        .block(Block::default());
    frame.render_widget(board, rect);
}
