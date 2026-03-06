//! Écrans des mini-jeux — un sous-module par jeu.

mod memory;
mod reflex;
mod rps;
mod tictactoe;

pub use memory::draw_memory;
pub use reflex::draw_reflex;
pub use rps::draw_rps;
pub use tictactoe::draw_game;

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

use monster_battle_core::minigame::MinigameType;

use crate::app::App;

// ── Sélection du type de mini-jeu ───────────────────────────────

/// Écran de sélection du type de mini-jeu.
pub fn draw_select_game_type(frame: &mut Frame, area: Rect, app: &App) {
    let types = MinigameType::all();

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
            let line = format!(
                "  {} {} — {}  ({})",
                t.icon(),
                t.label(),
                t.description(),
                t.stat_focus()
            );
            ListItem::new(Line::from(Span::styled(line, style)))
        })
        .collect();

    let name = app.minigame_monster_name.as_deref().unwrap_or("?");
    let title = format!(" 🎮 Mini-jeux — {} ", name);
    let list = List::new(list_items).block(
        Block::default()
            .title(title)
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

// ── Sélection de la difficulté ──────────────────────────────────

/// Écran de sélection de la difficulté (adapté au type de jeu sélectionné).
pub fn draw_select_difficulty(frame: &mut Frame, area: Rect, app: &App) {
    let game_type = app.minigame_type.unwrap_or(MinigameType::TicTacToe);

    let (title, labels): (String, Vec<(&str, &str)>) = match game_type {
        MinigameType::TicTacToe => {
            use monster_battle_core::minigame::tictactoe::Difficulty;
            let t = format!(" {} {} — Difficulté ", game_type.icon(), game_type.label());
            let l = Difficulty::all()
                .iter()
                .map(|d| {
                    let desc = match d {
                        Difficulty::Easy => "IA aléatoire — récompense faible",
                        Difficulty::Medium => "IA mixte — récompense moyenne",
                        Difficulty::Hard => "IA imbattable — récompense élevée",
                    };
                    (d.label(), desc)
                })
                .collect();
            (t, l)
        }
        MinigameType::Memory => {
            use monster_battle_core::minigame::memory::Difficulty;
            let t = format!(" {} {} — Difficulté ", game_type.icon(), game_type.label());
            let l = Difficulty::all()
                .iter()
                .map(|d| {
                    let desc = match d {
                        Difficulty::Easy => "Grille 4×3 — récompense faible",
                        Difficulty::Medium => "Grille 4×4 — récompense moyenne",
                        Difficulty::Hard => "Grille 4×5 — récompense élevée",
                    };
                    (d.label(), desc)
                })
                .collect();
            (t, l)
        }
        MinigameType::Reflex => {
            use monster_battle_core::minigame::reflex::Difficulty;
            let t = format!(" {} {} — Difficulté ", game_type.icon(), game_type.label());
            let l = Difficulty::all()
                .iter()
                .map(|d| {
                    let desc = match d {
                        Difficulty::Easy => "8 rounds — récompense faible",
                        Difficulty::Medium => "12 rounds — récompense moyenne",
                        Difficulty::Hard => "16 rounds — récompense élevée",
                    };
                    (d.label(), desc)
                })
                .collect();
            (t, l)
        }
        MinigameType::Rps => {
            use monster_battle_core::minigame::rps::Difficulty;
            let t = format!(" {} {} — Difficulté ", game_type.icon(), game_type.label());
            let l = Difficulty::all()
                .iter()
                .map(|d| {
                    let desc = match d {
                        Difficulty::Easy => "BO3 — récompense faible",
                        Difficulty::Medium => "BO5 — récompense moyenne",
                        Difficulty::Hard => "BO7 — récompense élevée",
                    };
                    (d.label(), desc)
                })
                .collect();
            (t, l)
        }
    };

    let list_items: Vec<ListItem> = labels
        .iter()
        .enumerate()
        .map(|(i, (label, desc))| {
            let style = if i == app.menu_index % labels.len() {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else {
                Style::default().fg(Color::White)
            };
            let line = format!("  {} — {}", label, desc);
            ListItem::new(Line::from(Span::styled(line, style)))
        })
        .collect();

    let list = List::new(list_items).block(
        Block::default()
            .title(title)
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
