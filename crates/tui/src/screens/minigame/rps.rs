//! Rendu du PPC élémentaire (Pierre-Papier-Ciseaux) en TUI.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use monster_battle_core::minigame::rps::RpsGame;

/// Écran principal du PPC élémentaire en cours de partie.
pub fn draw_rps(frame: &mut Frame, area: Rect, game: &RpsGame, monster_name: &str) {
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
        format!("Score : {}", game.score_display())
    };

    let info = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("  🪨 PPC ({}) — ", game.difficulty.label()),
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
    draw_rps_main(frame, chunks[1], game);

    // ── Controls ────────────────────────────────────
    let controls = if game.is_over() {
        let reward = game.reward();
        if reward.is_empty() {
            "  Entrée : retour au menu".to_string()
        } else {
            format!("  Entrée : récolter les récompenses ({})", reward.summary())
        }
    } else if game.waiting_confirm {
        "  Entrée : continuer │ Esc : abandonner".to_string()
    } else {
        "  ←→ : choisir │ Entrée : jouer │ Esc : abandonner".to_string()
    };

    let footer = Paragraph::new(Line::from(Span::styled(
        controls,
        Style::default().fg(Color::DarkGray),
    )))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, chunks[2]);
}

fn draw_rps_main(frame: &mut Frame, area: Rect, game: &RpsGame) {
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(4)])
        .split(area);

    // ── Résultat du dernier round ───────────────────
    if let Some(ref round) = game.last_round {
        let (color, label) = match round.outcome {
            monster_battle_core::minigame::rps::RoundOutcome::PlayerWin => {
                (Color::Green, "Gagné !")
            }
            monster_battle_core::minigame::rps::RoundOutcome::AiWin => (Color::Red, "Perdu !"),
            monster_battle_core::minigame::rps::RoundOutcome::Draw => (Color::Yellow, "Nul !"),
        };
        let text = format!(
            "  {} {} vs {} {} — {}",
            round.player_choice.icon(),
            round.player_choice,
            round.ai_choice.icon(),
            round.ai_choice,
            label
        );
        let p = Paragraph::new(Line::from(Span::styled(
            text,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
        frame.render_widget(p, inner[0]);
    } else {
        let p = Paragraph::new(Line::from(Span::styled(
            "  Choisissez un élément !",
            Style::default().fg(Color::DarkGray),
        )))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
        frame.render_widget(p, inner[0]);
    }

    // ── Choix des 3 éléments ────────────────────────
    if game.is_over() {
        let text = format!(
            "\n  Score final : {}\n  {}",
            game.score_display(),
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
    } else if !game.waiting_confirm {
        let choices = game.choices();
        let choice_w: u16 = 16;
        let total_w = choice_w * 3 + 4;
        let cx = inner[1].x + inner[1].width.saturating_sub(total_w) / 2;

        for (i, elem) in choices.iter().enumerate() {
            let is_sel = i == game.cursor;
            let style = if is_sel {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else {
                Style::default().fg(Color::White)
            };

            let text = format!("{} {}", elem.icon(), elem);
            let rect = Rect::new(cx + i as u16 * (choice_w + 2), inner[1].y + 1, choice_w, 3);
            let p = Paragraph::new(text)
                .alignment(Alignment::Center)
                .style(style)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(if is_sel {
                            Style::default().fg(Color::Yellow)
                        } else {
                            Style::default().fg(Color::DarkGray)
                        }),
                );
            frame.render_widget(p, rect);
        }
    } else {
        let p = Paragraph::new("  Appuyez sur Entrée pour continuer...")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            );
        frame.render_widget(p, inner[1]);
    }
}
