use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
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

    // Réserver de la place pour l'événement aléatoire si présent
    let has_event = app.event_message.is_some();
    let main_area = if has_event {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(10)])
            .split(area);

        // Afficher le bandeau d'événement
        if let Some(ref msg) = app.event_message {
            let event_block = Block::default()
                .title(" ✨ Événement ! ")
                .title_style(
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                )
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta));
            let event_text = Paragraph::new(Line::from(Span::styled(
                format!("  {}", msg),
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::ITALIC),
            )))
            .block(event_block);
            frame.render_widget(event_text, chunks[0]);
        }

        chunks[1]
    } else {
        area
    };

    let selected = app
        .monster_select_index
        .min(monsters.len().saturating_sub(1));

    if monsters.len() == 1 {
        draw_monster_detail(frame, main_area, &monsters[0]);
        draw_food_overlay(frame, main_area, app);
        return;
    }

    // Plusieurs monstres : liste à gauche + détail à droite
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(36), Constraint::Min(30)])
        .split(main_area);

    // ── Liste des monstres (gauche) ─────────────────────────
    let list_items: Vec<ListItem> = monsters
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let is_selected = i == selected;
            let cursor = if is_selected { "▸ " } else { "  " };
            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let secondary = m
                .secondary_type
                .map(|t| format!("/{}", t.icon()))
                .unwrap_or_default();

            let line = format!(
                "{}{}{} {} Nv.{} PV{}/{}",
                cursor,
                m.primary_type.icon(),
                secondary,
                m.name,
                m.level,
                m.current_hp,
                m.max_hp(),
            );
            ListItem::new(Line::from(Span::styled(line, style)))
        })
        .collect();

    let list = List::new(list_items).block(
        Block::default()
            .title(format!(" Mes Monstres ({}) ", monsters.len()))
            .title_style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)),
    );

    frame.render_widget(list, cols[0]);

    // ── Détail du monstre sélectionné (droite) ──────────────
    draw_monster_detail(frame, cols[1], &monsters[selected]);

    // ── Overlay de sélection de nourriture ──────────────────
    draw_food_overlay(frame, main_area, app);
}

/// Affiche la fiche détaillée d'un monstre dans la zone donnée.
fn draw_monster_detail(frame: &mut Frame, area: Rect, m: &monster_battle_core::Monster) {
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
        ((m.xp as f64 / m.xp_to_next_level() as f64 * xp_bar_width as f64) as usize)
            .min(xp_bar_width)
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
    let hp_pct = hp_pct.min(hp_bar_width);
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
    let age = m.age_stage();
    let grid = sprites::pixel::get_blended_sprite(m.primary_type, m.secondary_type, age);
    let mut sprite_lines: Vec<Line> = vec![Line::from("")];
    sprite_lines.extend(sprites::pixel::render_pixel_sprite(
        &grid,
        m.primary_type,
        m.secondary_type,
        age,
    ));
    sprite_lines.push(Line::from(""));
    sprite_lines.push(Line::from(Span::styled(
        format!("  {} {}", type_icon, m.primary_type),
        Style::default()
            .fg(element_color(m.primary_type))
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
    let stage = m.age_stage();
    let age_bar_width = 20;
    let age_pct = (m.age_ratio() * age_bar_width as f64) as usize;
    let age_pct = age_pct.min(age_bar_width);
    let age_bar_color = match stage {
        monster_battle_core::AgeStage::Baby => "💿",
        monster_battle_core::AgeStage::Young => "🌱",
        monster_battle_core::AgeStage::Adult => "💪",
        monster_battle_core::AgeStage::Old => "🧓",
    };
    let age_bar = format!(
        "{} [{}{}] {}j/{}j",
        age_bar_color,
        "█".repeat(age_pct),
        "░".repeat(age_bar_width - age_pct),
        m.age_days(),
        m.max_age_days()
    );

    let hunger = m.hunger_level();
    let hunger_info = format!("{} {}", hunger.icon(), hunger);
    let hunger_modifier = format!("{}%", (hunger.stat_multiplier() * 100.0) as i32);

    let happiness = m.happiness_level();
    let happiness_info = format!("{} {} ({})", happiness.icon(), happiness, m.happiness);
    let happiness_modifier = format!("{}%", (happiness.stat_multiplier() * 100.0) as i32);

    let bond = m.bond_level();
    let bond_info = format!("{} {} ({}/100)", bond.icon(), bond, m.bond);
    let bond_title = bond
        .title()
        .map(|t| format!(" «{}»", t))
        .unwrap_or_default();

    let food_buff_info = match m.active_food_buff() {
        Some(monster_battle_core::FoodType::Meat) => "🥩 ATK+15%".to_string(),
        Some(monster_battle_core::FoodType::Fish) => "🐟 VIT+15%".to_string(),
        Some(other) => format!("{} {}", other.icon(), other),
        None => "Aucun".to_string(),
    };

    let stat_modifier = format!(
        "{}% ({} × {} × {})",
        (stage.stat_multiplier() * hunger.stat_multiplier() * happiness.stat_multiplier() * 100.0)
            as i32,
        stage,
        hunger,
        happiness,
    );

    let info = format!(
        r#"
  {} {}{}{}
  Niveau : {}
  XP     : {}
  PV     : {}

  ── Stats ──────────────
  ATK : {}   DEF : {}
  SPD : {}   S.ATK : {}
  S.DEF : {}

  ── Infos ──────────────
  Traits     : {}
  Stade      : {} {}
  Âge        : {}
  Faim       : {} (stats: {})
  Bonheur    : {} (stats: {})
  Lien       : {}
  Buff food  : {}
  Puissance  : {}
  Génération : {}
  Victoires  : {}  |  Défaites : {}

  ── Raccourcis ─────────
  [F] Nourrir  [Esc] Retour
"#,
        type_icon,
        m.name,
        bond_title,
        secondary,
        m.level,
        xp_bar,
        hp_bar,
        m.effective_attack(),
        m.effective_defense(),
        m.effective_speed(),
        m.effective_sp_attack(),
        m.effective_sp_defense(),
        traits_str,
        stage.icon(),
        stage,
        age_bar,
        hunger_info,
        hunger_modifier,
        happiness_info,
        happiness_modifier,
        bond_info,
        food_buff_info,
        stat_modifier,
        m.generation,
        m.wins,
        m.losses,
    );

    let detail = Paragraph::new(info)
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: false });

    frame.render_widget(detail, cols[1]);
}

/// Affiche une fenêtre modale de sélection de nourriture par-dessus tout.
fn draw_food_overlay(frame: &mut Frame, area: Rect, app: &App) {
    if !app.food_selecting {
        return;
    }

    use monster_battle_core::FoodType;

    let foods = FoodType::all();
    let popup_height = (foods.len() as u16) + 4;
    let popup_width = 36;

    // Centrer la popup
    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(
        x,
        y,
        popup_width.min(area.width),
        popup_height.min(area.height),
    );

    // Fond sombre
    let bg = Block::default().style(Style::default().bg(Color::Black));
    frame.render_widget(ratatui::widgets::Clear, popup_area);
    frame.render_widget(bg, popup_area);

    let items: Vec<ListItem> = foods
        .iter()
        .enumerate()
        .map(|(i, food)| {
            let is_selected = i == app.food_select_index;
            let cursor = if is_selected { "▸ " } else { "  " };
            let bonus = format!("+{} bonheur", food.happiness_bonus());
            let extra = match food {
                FoodType::Meat => " | ATK+15% 1h",
                FoodType::Fish => " | VIT+15% 1h",
                FoodType::Herbs => " | Soigne humeur",
                FoodType::Cake => " | x2 repas",
                FoodType::Berry => "",
            };
            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(Span::styled(
                format!("{}{} {} ({}{})", cursor, food.icon(), food, bonus, extra),
                style,
            )))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" 🍽️ Choisir un aliment ")
            .title_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );

    frame.render_widget(list, popup_area);
}
