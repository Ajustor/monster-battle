use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use monster_battle_core::minigame::tictactoe::{Cell, Difficulty, TicTacToe};

use crate::app::App;

// ── Sélection de la difficulté ──────────────────────────────────

/// Écran de sélection de la difficulté du morpion.
pub fn draw_select_difficulty(frame: &mut Frame, area: Rect, app: &App) {
    let difficulties = Difficulty::all();

    let list_items: Vec<ListItem> = difficulties
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let style = if i == app.menu_index % difficulties.len() {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else {
                Style::default().fg(Color::White)
            };
            let desc = match d {
                Difficulty::Easy => "IA aléatoire — récompense faible",
                Difficulty::Medium => "IA mixte — récompense moyenne",
                Difficulty::Hard => "IA imbattable — récompense élevée",
            };
            let line = format!("  {} — {}", d.label(), desc);
            ListItem::new(Line::from(Span::styled(line, style)))
        })
        .collect();

    let list = List::new(list_items).block(
        Block::default()
            .title(" 🎮 Morpion — Choisir la difficulté ")
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(list, area);
}

// ── Plateau de jeu ──────────────────────────────────────────────

/// Écran principal du morpion en cours de partie.
pub fn draw_game(frame: &mut Frame, area: Rect, game: &TicTacToe, monster_name: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Info
            Constraint::Min(11),   // Board
            Constraint::Length(3), // Controls
        ])
        .split(area);

    // ── Info header ─────────────────────────────────
    let status = if game.is_over() {
        game.result_label().to_string()
    } else {
        format!(
            "À vous de jouer ! ({})",
            if game.current_turn == Cell::X {
                "X"
            } else {
                "O"
            }
        )
    };

    let info = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("  🎮 Morpion ({}) — ", game.difficulty.label()),
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
                        None => Color::White,
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
    draw_board(frame, chunks[1], game);

    // ── Controls ────────────────────────────────────
    let controls = if game.is_over() {
        let reward = game.reward();
        if reward.is_empty() {
            "  Entrée : retour au menu".to_string()
        } else {
            format!("  Entrée : récolter les récompenses ({})", reward.summary())
        }
    } else {
        "  ↑↓←→ : déplacer │ Entrée : jouer │ Esc : abandonner".to_string()
    };

    let footer = Paragraph::new(Line::from(Span::styled(
        controls,
        Style::default().fg(Color::DarkGray),
    )))
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(footer, chunks[2]);
}

/// Dessine la grille 3×3 du morpion centrée dans la zone.
fn draw_board(frame: &mut Frame, area: Rect, game: &TicTacToe) {
    // Each cell is 5 chars wide, 3 lines tall (avec séparateurs)
    // Total: 17 wide (5+1+5+1+5), 11 tall (3+1+3+1+3)
    let board_w: u16 = 17;
    let board_h: u16 = 11;

    // Centre le plateau
    let x = area.x + area.width.saturating_sub(board_w) / 2;
    let y = area.y + area.height.saturating_sub(board_h) / 2;

    let winning = game.winning_line.unwrap_or([usize::MAX; 3]);

    let mut lines: Vec<Line> = Vec::new();

    for row in 0..3 {
        // Ligne du haut de la rangée
        let mut top_spans: Vec<Span> = Vec::new();
        let mut mid_spans: Vec<Span> = Vec::new();
        let mut bot_spans: Vec<Span> = Vec::new();

        for col in 0..3 {
            let idx = row * 3 + col;
            let cell = game.board[idx];
            let is_cursor = idx == game.cursor && !game.is_over();
            let is_winning = winning.contains(&idx);

            let (fg, bg) = if is_winning {
                (Color::Black, Color::Green)
            } else if is_cursor {
                (Color::Yellow, Color::DarkGray)
            } else {
                let fg = match cell {
                    Cell::X => Color::Cyan,
                    Cell::O => Color::Red,
                    Cell::Empty => Color::DarkGray,
                };
                (fg, Color::Reset)
            };

            let style = Style::default().fg(fg).bg(bg);
            let sym = cell.symbol();

            // 5-wide cell: "     ", " sym ", "     " (3 tall)
            let top = format!("{:5}", "");
            let mid = format!("  {}  ", sym);
            let bot = format!("{:5}", "");

            top_spans.push(Span::styled(top, style));
            mid_spans.push(Span::styled(mid, style));
            bot_spans.push(Span::styled(bot, style));

            if col < 2 {
                let sep_style = Style::default().fg(Color::DarkGray);
                top_spans.push(Span::styled("│", sep_style));
                mid_spans.push(Span::styled("│", sep_style));
                bot_spans.push(Span::styled("│", sep_style));
            }
        }

        lines.push(Line::from(top_spans));
        lines.push(Line::from(mid_spans));
        lines.push(Line::from(bot_spans));

        if row < 2 {
            lines.push(Line::from(Span::styled(
                "─────┼─────┼─────",
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    let board_rect = Rect::new(x, y, board_w, board_h);
    let board = Paragraph::new(lines)
        .alignment(Alignment::Left)
        .block(Block::default());

    frame.render_widget(board, board_rect);
}
