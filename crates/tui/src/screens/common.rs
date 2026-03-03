use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::App;
use crate::screens::Screen;
use crate::screens::SelectMonsterTarget;

use monster_battle_storage::MonsterStorage;

/// Dessine le header commun à tous les écrans.
pub fn draw_header(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            " 🐉 Monster Battle ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            env!("CARGO_PKG_VERSION"),
            Style::default().fg(Color::DarkGray),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );

    frame.render_widget(title, area);
}

/// Dessine le footer commun à tous les écrans.
pub fn draw_footer(frame: &mut Frame, area: Rect, app: &App) {
    let text = if let Some(msg) = &app.message {
        msg.clone()
    } else {
        match &app.current_screen {
            Screen::MainMenu => "↑↓ Naviguer | Enter Sélectionner | q Quitter".to_string(),
            Screen::NamingMonster { .. } => {
                "Tapez le nom de votre monstre | Enter Valider | Esc Annuler".to_string()
            }
            Screen::Combat(phase) => match phase {
                crate::screens::pvp::PvpPhase::Error(_) => "Enter ou Esc pour revenir".to_string(),
                _ => "Esc Annuler".to_string(),
            },
            Screen::Breeding(phase) => match phase {
                crate::screens::breeding::BreedPhase::NamingChild => {
                    "Tapez le nom du bébé | Enter Valider | Esc Annuler".to_string()
                }
                crate::screens::breeding::BreedPhase::Result => {
                    "↑↓ Défiler | Enter ou q pour revenir".to_string()
                }
                crate::screens::breeding::BreedPhase::Error(_) => {
                    "Enter ou Esc pour revenir".to_string()
                }
                _ => "Esc Annuler".to_string(),
            },
            Screen::Battle => {
                if let Some(ref b) = app.battle_state {
                    match b.phase {
                        monster_battle_core::battle::BattlePhase::PlayerChooseAttack => {
                            "↑↓ Choisir | Enter Attaquer | Esc Fuir".to_string()
                        }
                        monster_battle_core::battle::BattlePhase::Victory
                        | monster_battle_core::battle::BattlePhase::Defeat => {
                            "Enter pour continuer...".to_string()
                        }
                        monster_battle_core::battle::BattlePhase::WaitingForOpponent => {
                            if b.message_queue.is_empty() && b.current_message.is_none() {
                                "⏳ En attente de l'adversaire...".to_string()
                            } else {
                                "Enter / Espace pour continuer… | Esc Fuir".to_string()
                            }
                        }
                        _ => "Enter / Espace pour continuer… | Esc Fuir".to_string(),
                    }
                } else {
                    String::new()
                }
            }
            Screen::MonsterList => "↑↓ Naviguer | f Nourrir | q Retour".to_string(),
            Screen::SelectMonster(_) => {
                "↑↓ Naviguer | Enter Sélectionner | Esc Annuler".to_string()
            }
            Screen::Help => "↑↓ Défiler | q Retour".to_string(),
            _ => "↑↓ Naviguer | Enter Sélectionner | q Retour".to_string(),
        }
    };

    // Statut serveur à gauche
    use crate::app::ServerStatus;
    let (status_icon, status_color) = match app.server_status {
        ServerStatus::Online => ("● En ligne", Color::Green),
        ServerStatus::Offline => ("● Hors ligne", Color::Red),
        ServerStatus::Unknown => ("● Connexion…", Color::DarkGray),
    };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(16), Constraint::Min(1)])
        .split(area);

    let status = Paragraph::new(Line::from(Span::styled(
        format!(" {}", status_icon),
        Style::default().fg(status_color),
    )))
    .block(Block::default().borders(Borders::ALL));

    let footer = Paragraph::new(text)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(status, chunks[0]);
    frame.render_widget(footer, chunks[1]);
}

/// Dessine un placeholder pour les écrans non implémentés.
pub fn draw_placeholder(frame: &mut Frame, area: Rect, text: &str) {
    let p = Paragraph::new(text)
        .style(Style::default().fg(Color::DarkGray))
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(p, area);
}

/// Titre et couleur selon la cible de sélection.
fn select_monster_style(target: &SelectMonsterTarget) -> (&'static str, Color) {
    match target {
        SelectMonsterTarget::Training => (" ⚔️  Choisir un monstre — Entraînement ", Color::Yellow),
        SelectMonsterTarget::CombatPvP => (" 🗡️  Choisir un monstre — Combat PvP ", Color::Red),
        SelectMonsterTarget::Breeding => (" 🧬 Choisir un monstre — Reproduction ", Color::Magenta),
        SelectMonsterTarget::Minigame => (" 🎮 Choisir un monstre — Mini-jeux ", Color::Cyan),
    }
}

/// Dessine l'écran mutualisé de sélection de monstre.
pub fn draw_select_monster(frame: &mut Frame, area: Rect, app: &App, target: &SelectMonsterTarget) {
    let (title, color) = select_monster_style(target);
    let monsters = app.storage.list_alive().unwrap_or_default();

    if monsters.is_empty() {
        let paragraph = Paragraph::new("Aucun monstre vivant !")
            .style(Style::default().fg(Color::Red))
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(color)),
            );
        frame.render_widget(paragraph, area);
        return;
    }

    let mut lines: Vec<Line> = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Choisissez le monstre à envoyer :",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    for (i, m) in monsters.iter().enumerate() {
        let is_selected = i == app.monster_select_index;
        let cursor = if is_selected { "▸ " } else { "  " };
        let style = if is_selected {
            Style::default().fg(color).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let secondary = m
            .secondary_type
            .map(|t| format!("/{}", t.icon()))
            .unwrap_or_default();

        let line = format!(
            "{}  {}{} {}  Nv.{}  PV {}/{}  ATK {} DEF {} SPD {}",
            cursor,
            m.primary_type.icon(),
            secondary,
            m.name,
            m.level,
            m.current_hp,
            m.max_hp(),
            m.effective_attack(),
            m.effective_defense(),
            m.effective_speed(),
        );

        lines.push(Line::from(Span::styled(line, style)));

        if is_selected {
            let traits_str = if m.traits.is_empty() {
                "Aucun".to_string()
            } else {
                m.traits
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            };

            lines.push(Line::from(Span::styled(
                format!(
                    "       S.ATK {} S.DEF {}  Traits: {}  W/L: {}/{}",
                    m.effective_sp_attack(),
                    m.effective_sp_defense(),
                    traits_str,
                    m.wins,
                    m.losses,
                ),
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  ↑↓ Naviguer | Enter Sélectionner | Esc Annuler",
        Style::default().fg(Color::DarkGray),
    )));

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title(title)
            .title_style(Style::default().fg(color).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(color)),
    );

    frame.render_widget(paragraph, area);
}
