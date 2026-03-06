//! Mini-jeux pour améliorer les stats des monstres sans combattre.
//!
//! Chaque mini-jeu est une partie rapide contre une IA. La récompense
//! dépend de la difficulté et du résultat (victoire / nul / défaite).

pub mod memory;
pub mod reflex;
pub mod rps;
pub mod tictactoe;

use std::fmt;

use crate::types::Stats;

/// Types de mini-jeux disponibles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinigameType {
    /// Morpion (Tic-Tac-Toe).
    TicTacToe,
    /// Memory (paires).
    Memory,
    /// Réflexe (QTE).
    Reflex,
    /// Pierre-Papier-Ciseaux élémentaire.
    Rps,
}

impl MinigameType {
    /// Toutes les variantes.
    pub fn all() -> &'static [MinigameType] {
        &[
            MinigameType::TicTacToe,
            MinigameType::Memory,
            MinigameType::Reflex,
            MinigameType::Rps,
        ]
    }

    /// Nom du mini-jeu.
    pub fn label(self) -> &'static str {
        match self {
            MinigameType::TicTacToe => "Morpion",
            MinigameType::Memory => "Memory",
            MinigameType::Reflex => "Reflexe",
            MinigameType::Rps => "PPC Elementaire",
        }
    }

    /// Icône emoji.
    pub fn icon(self) -> &'static str {
        match self {
            MinigameType::TicTacToe => "❌",
            MinigameType::Memory => "🃏",
            MinigameType::Reflex => "⚡",
            MinigameType::Rps => "🪨",
        }
    }

    /// Description courte.
    pub fn description(self) -> &'static str {
        match self {
            MinigameType::TicTacToe => "Morpion classique contre une IA",
            MinigameType::Memory => "Retrouvez les paires de cartes",
            MinigameType::Reflex => "Appuyez sur la bonne direction",
            MinigameType::Rps => "Pierre-papier-ciseaux avec les types",
        }
    }

    /// Stats principalement boostées.
    pub fn stat_focus(self) -> &'static str {
        match self {
            MinigameType::TicTacToe => "PV, ATK, VIT",
            MinigameType::Memory => "DEF, DEF.S",
            MinigameType::Reflex => "VIT, ATK",
            MinigameType::Rps => "ATK, ATK.S",
        }
    }
}

impl fmt::Display for MinigameType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Résultat d'un mini-jeu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinigameResult {
    /// Le joueur a gagné.
    Win,
    /// Match nul.
    Draw,
    /// Le joueur a perdu.
    Loss,
}

/// Récompense d'un mini-jeu appliquée aux stats de base d'un monstre.
#[derive(Debug, Clone)]
pub struct StatReward {
    pub hp: u32,
    pub attack: u32,
    pub defense: u32,
    pub speed: u32,
    pub special_attack: u32,
    pub special_defense: u32,
    /// XP bonus accordée.
    pub xp: u32,
}

impl StatReward {
    /// Récompense vide (aucun bonus).
    pub fn none() -> Self {
        Self {
            hp: 0,
            attack: 0,
            defense: 0,
            speed: 0,
            special_attack: 0,
            special_defense: 0,
            xp: 0,
        }
    }

    /// Retourne `true` si la récompense est entièrement nulle.
    pub fn is_empty(&self) -> bool {
        self.hp == 0
            && self.attack == 0
            && self.defense == 0
            && self.speed == 0
            && self.special_attack == 0
            && self.special_defense == 0
            && self.xp == 0
    }

    /// Résumé textuel de la récompense.
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        if self.hp > 0 {
            parts.push(format!("PV+{}", self.hp));
        }
        if self.attack > 0 {
            parts.push(format!("ATK+{}", self.attack));
        }
        if self.defense > 0 {
            parts.push(format!("DEF+{}", self.defense));
        }
        if self.speed > 0 {
            parts.push(format!("VIT+{}", self.speed));
        }
        if self.special_attack > 0 {
            parts.push(format!("ATK.S+{}", self.special_attack));
        }
        if self.special_defense > 0 {
            parts.push(format!("DEF.S+{}", self.special_defense));
        }
        if self.xp > 0 {
            parts.push(format!("XP+{}", self.xp));
        }
        if parts.is_empty() {
            "Aucune récompense".to_string()
        } else {
            parts.join(", ")
        }
    }
}

/// Applique une récompense aux stats de base d'un monstre.
pub fn apply_reward(stats: &mut Stats, reward: &StatReward) {
    stats.hp += reward.hp;
    stats.attack += reward.attack;
    stats.defense += reward.defense;
    stats.speed += reward.speed;
    stats.special_attack += reward.special_attack;
    stats.special_defense += reward.special_defense;
}
