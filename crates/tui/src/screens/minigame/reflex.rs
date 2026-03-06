//! Rendu du jeu Réflexe (QTE) en TUI.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use monster_battle_core::minigame::reflex::{ReflexGame, RoundResult as ReflexRoundResult};

/// Écran principal du Réflexe en cours de partie.
pub fn draw_reflex(frame: &mut Frame, area: Rect, game: &ReflexGame, monster_name: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(area);

    // ── Info header ─────────────────────────────────
    let status = if game.is_over() {
        game.result_label().to_string()
    } else {
        format!(
            "Round {}/{} — Score : {}/{}",
            game.current_round + 1,
            game.total_rounds,
            game.score,
            game.current_round
        )
    };

    let info = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("  ⚡ Réflexe ({}) — ", game.difficulty.label()),
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

    // ── Main area ───────────────────────────────────
    draw_reflex_main(frame, chunks[1], game);

    // ── Controls ────────────────────────────────────
    let controls = if game.is_over() {
        let reward = game.reward();
        if reward.is_empty() {
            "  Entrée : retour au menu".to_string()
        } else {
            format!("  Entrée : récolter les récompenses ({})", reward.summary())
        }
    } else {
        "  ↑↓←→ : appuyer sur la direction │ Esc : abandonner".to_string()
    };

    let footer = Paragraph::new(Line::from(Span::styled(
        controls,
        Style::default().fg(Color::DarkGray),
    )))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, chunks[2]);
}

fn draw_reflex_main(frame: &mut Frame, area: Rect, game: &ReflexGame) {
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(4)])
        .split(area);

    // ── Historique des derniers rounds ───────────────
    let history_len = game.results.len();
    let display_count = 20.min(history_len);
    let start = history_len.saturating_sub(display_count);
    let history_spans: Vec<Span> = game.results[start..]
        .iter()
        .map(|r| match r {
            ReflexRoundResult::Correct => Span::styled("✓ ", Style::default().fg(Color::Green)),
            ReflexRoundResult::Wrong => Span::styled("✗ ", Style::default().fg(Color::Red)),
        })
        .collect();
    let history_line = if history_spans.is_empty() {
        Line::from(Span::styled(
            "  En attente...",
            Style::default().fg(Color::DarkGray),
        ))
    } else {
        Line::from(history_spans)
    };
    let history = Paragraph::new(history_line)
        .alignment(Alignment::Center)
        .block(Block::default());
    frame.render_widget(history, inner[0]);

    // ── Flèche à reproduire ─────────────────────────
    if game.is_over() {
        let text = format!(
            "\n  Score final : {}/{}\n  {}",
            game.score,
            game.total_rounds,
            game.result_label()
        );
        let p = Paragraph::new(text)
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            );
        frame.render_widget(p, inner[1]);
    } else if let Some(arrow) = game.current_arrow() {
        let big_arrow = format!("\n\n  {}  {}", arrow.symbol(), arrow.label());
        let p = Paragraph::new(big_arrow)
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Appuyez sur la bonne direction ! ")
                    .title_style(Style::default().fg(Color::Cyan))
                    .border_style(Style::default().fg(Color::Cyan)),
            );
        frame.render_widget(p, inner[1]);
    }
}
