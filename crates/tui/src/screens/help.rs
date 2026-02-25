use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::App;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let lines = vec![
        Line::from(Span::styled(
            "  🐉 Bienvenue dans Monster Battle !",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  ── But du jeu ──────────────────────────",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  Élevez un monstre unique, nourrissez-le, entraînez-le"),
        Line::from("  et affrontez d'autres joueurs en combat PvP !"),
        Line::from("  Votre monstre est mortel : il vieillit et peut mourir"),
        Line::from("  de vieillesse, de faim ou au combat."),
        Line::from("  Reproduisez-le pour créer une lignée plus puissante."),
        Line::from(""),
        Line::from(Span::styled(
            "  ── Cycle de vie ────────────────────────",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  💿 Bébé (0-15%)     → Stats ×80%"),
        Line::from("  🌱 Jeune (15-40%)   → Stats ×95%"),
        Line::from("  💪 Adulte (40-75%)  → Stats ×110%  ← pic de puissance"),
        Line::from("  🧓 Vieux (75-100%)  → Stats ×85%"),
        Line::from("  💀 Mort au-delà de la durée de vie max (~30 jours)"),
        Line::from(""),
        Line::from(Span::styled(
            "  ── Système de faim ─────────────────────",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  🍽️ A faim           → Stats normales (×100%)"),
        Line::from("  😊 Rassasié (<12h)  → Boost ! (×115%)"),
        Line::from("  🤢 Trop mangé (3×)  → Malus (×85%)"),
        Line::from("  💀 Affamé (3+ jours) → Le monstre meurt de faim !"),
        Line::from(""),
        Line::from("  Nourrissez votre monstre depuis sa fiche (touche F)."),
        Line::from("  Attention : 3 repas en 12h = gavage → malus."),
        Line::from(""),
        Line::from(Span::styled(
            "  ── Combat ──────────────────────────────",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  ⚔️  Entraînement docile : 50% XP, pas de mort"),
        Line::from("  ⚔️  Entraînement sauvage : 100% XP, mort possible"),
        Line::from("  🗡️  PvP en ligne : 200% XP si KO, mort du perdant"),
        Line::from("  🏳️  Fuite PvP : pas de mort, l'adversaire gagne 100% XP"),
        Line::from(""),
        Line::from("  Les stats (ATK, DEF, SPD…) déterminent l'ordre"),
        Line::from("  d'attaque et les dégâts. Les types élémentaires"),
        Line::from("  créent des avantages/désavantages (super efficace !)."),
        Line::from(""),
        Line::from(Span::styled(
            "  ── Types élémentaires ──────────────────",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled("  🔥 Feu    > 🌿 Plante > 💧 Eau    > 🔥 Feu", Style::default().fg(Color::White))),
        Line::from(Span::styled("  ⚡ Électrique > 💧 Eau    🌍 Terre > ⚡ Électrique", Style::default().fg(Color::White))),
        Line::from(Span::styled("  🌀 Vent   > 🌍 Terre   🌑 Ombre > 🌟 Lumière > 🌑 Ombre", Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(Span::styled(
            "  ── Reproduction ────────────────────────",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  🧬 Croisez votre monstre avec celui d'un autre joueur."),
        Line::from("  Le bébé hérite des types, stats et traits des parents."),
        Line::from("  Des mutations peuvent apparaître ! Le type secondaire"),
        Line::from("  est transmis par reproduction uniquement."),
        Line::from(""),
        Line::from(Span::styled(
            "  ── Traits génétiques ───────────────────",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  🎯 CriticalStrike   → +crit (20% vs 8%)"),
        Line::from("  😡 Berserk          → ×1.5 ATK sous 25% PV"),
        Line::from("  💨 Evasion          → 12% d'esquive"),
        Line::from("  🌵 Thorns           → 15% dégâts renvoyés"),
        Line::from("  💪 Tenacity         → 15% de survie à 1 PV"),
        Line::from("  🩹 Regeneration     → 5% PV max régénérés/tour"),
        Line::from("  📚 FastLearner      → XP ×1.5"),
        Line::from("  🕰️  Longevity        → +15 jours de vie"),
        Line::from(""),
        Line::from(Span::styled(
            "  ── Commandes ───────────────────────────",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  ↑↓       Naviguer"),
        Line::from("  ←→       Docile / Sauvage (entraînement)"),
        Line::from("  Enter    Sélectionner / Confirmer"),
        Line::from("  Esc / q  Retour / Quitter"),
        Line::from("  f        Nourrir (fiche monstre)"),
        Line::from("  m        Activer / Couper la musique"),
        Line::from(""),
    ];

    let content = Paragraph::new(lines)
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: false })
        .scroll((app.scroll_offset as u16, 0))
        .block(
            Block::default()
                .title(" ❓ Aide — Comment jouer ")
                .title_style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

    frame.render_widget(content, area);
}
