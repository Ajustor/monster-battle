use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use monster_battle_storage::MonsterStorage;

use crate::app::App;

/// Sous-écrans du combat PvP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PvpPhase {
    /// Sélection du monstre à envoyer au combat.
    SelectMonster,
    /// Recherche d'un adversaire sur le serveur.
    Searching,
    /// Adversaire trouvé, combat en cours.
    Matched { opponent_name: String },
    /// Erreur réseau.
    Error(String),
}

/// Dessine l'écran de recherche d'adversaire.
pub fn draw_searching(frame: &mut Frame, area: Rect, app: &App) {
    let text = format!(
        "⏳ Recherche d'un adversaire...\n\n\
         Serveur : {}\n\n\
         Le combat commencera automatiquement dès qu'un\n\
         autre joueur sera trouvé.\n\n\
         Appuyez sur Esc pour annuler.",
        app.server_address
    );

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::Yellow))
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(" ⚔️  Combat PvP ")
                .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red)),
        );

    frame.render_widget(paragraph, area);
}

/// Dessine l'écran de combat en cours (adversaire trouvé).
pub fn draw_matched(frame: &mut Frame, area: Rect, _app: &App, opponent_name: &str) {
    let text = format!(
        "⚔️  Adversaire trouvé : {} !\n\n\
         Combat en cours... Veuillez patienter.",
        opponent_name
    );

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(" Combat PvP ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red)),
        );

    frame.render_widget(paragraph, area);
}

/// Dessine un écran d'erreur réseau.
pub fn draw_error(frame: &mut Frame, area: Rect, error: &str) {
    let text = format!(
        "❌ Erreur réseau :\n\n{}\n\n\
         Appuyez sur Enter ou Esc pour revenir.",
        error
    );

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::Red))
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(" Erreur ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red)),
        );

    frame.render_widget(paragraph, area);
}

/// Dessine l'écran de sélection du monstre pour le combat PvP.
pub fn draw_select_monster(frame: &mut Frame, area: Rect, app: &App) {
    draw_monster_selection(
        frame,
        area,
        app,
        " ⚔️  Choisir un monstre — Combat PvP ",
        Color::Red,
    );
}

/// Dessine la liste de sélection de monstre (partagé PvP / Reproduction).
pub fn draw_monster_selection(frame: &mut Frame, area: Rect, app: &App, title: &str, color: Color) {
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
