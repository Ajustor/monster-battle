use serde::{Deserialize, Serialize};
use std::fmt;

/// Types élémentaires des monstres.
/// Chaque monstre possède un type primaire et optionnellement un type secondaire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ElementType {
    Fire,
    Water,
    Plant,
    Electric,
    Earth,
    Wind,
    Shadow,
    Light,
}

impl ElementType {
    /// Retourne toutes les variantes du type élémentaire.
    pub fn all() -> &'static [ElementType] {
        &[
            ElementType::Fire,
            ElementType::Water,
            ElementType::Plant,
            ElementType::Electric,
            ElementType::Earth,
            ElementType::Wind,
            ElementType::Shadow,
            ElementType::Light,
        ]
    }

    /// Retourne le multiplicateur de dégâts de `self` contre `defender`.
    /// > 1.0 = super efficace, < 1.0 = pas très efficace, 1.0 = neutre
    pub fn effectiveness_against(&self, defender: &ElementType) -> f64 {
        use ElementType::*;
        match (self, defender) {
            // Super efficace (×1.5)
            (Fire, Plant) | (Fire, Wind) => 1.5,
            (Water, Fire) | (Water, Earth) => 1.5,
            (Plant, Water) | (Plant, Earth) => 1.5,
            (Electric, Water) | (Electric, Wind) => 1.5,
            (Earth, Electric) | (Earth, Fire) => 1.5,
            (Wind, Plant) | (Wind, Earth) => 1.5,
            (Shadow, Light) => 1.5,
            (Light, Shadow) => 1.5,

            // Pas très efficace (×0.5)
            (Fire, Water) | (Fire, Earth) => 0.5,
            (Water, Plant) | (Water, Electric) => 0.5,
            (Plant, Fire) | (Plant, Wind) => 0.5,
            (Electric, Earth) => 0.5,
            (Earth, Plant) | (Earth, Wind) => 0.5,
            (Wind, Electric) | (Wind, Fire) => 0.5,

            // Neutre
            _ => 1.0,
        }
    }

    /// Emoji représentatif pour le TUI.
    pub fn icon(&self) -> &'static str {
        match self {
            ElementType::Fire => "🔥",
            ElementType::Water => "💧",
            ElementType::Plant => "🌿",
            ElementType::Electric => "⚡",
            ElementType::Earth => "🪨",
            ElementType::Wind => "🌪️",
            ElementType::Shadow => "🌑",
            ElementType::Light => "✨",
        }
    }
}

impl fmt::Display for ElementType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            ElementType::Fire => "Feu",
            ElementType::Water => "Eau",
            ElementType::Plant => "Plante",
            ElementType::Electric => "Électrique",
            ElementType::Earth => "Terre",
            ElementType::Wind => "Vent",
            ElementType::Shadow => "Ombre",
            ElementType::Light => "Lumière",
        };
        write!(f, "{}", name)
    }
}

/// Statistiques de base d'un monstre.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub hp: u32,
    pub attack: u32,
    pub defense: u32,
    pub speed: u32,
    pub special_attack: u32,
    pub special_defense: u32,
}

impl Stats {
    /// Crée des stats à partir de valeurs brutes.
    pub fn new(hp: u32, attack: u32, defense: u32, speed: u32, sp_atk: u32, sp_def: u32) -> Self {
        Self {
            hp,
            attack,
            defense,
            speed,
            special_attack: sp_atk,
            special_defense: sp_def,
        }
    }

    /// Retourne la somme totale des stats (indicateur de puissance).
    pub fn total(&self) -> u32 {
        self.hp
            + self.attack
            + self.defense
            + self.speed
            + self.special_attack
            + self.special_defense
    }
}

/// Trait génétique pouvant être hérité ou muté lors de la reproduction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Trait {
    /// Le monstre regagne des PV chaque tour.
    Regeneration,
    /// Le monstre a une chance d'esquiver les attaques.
    Evasion,
    /// Les attaques du monstre ont une chance de coup critique accrue.
    CriticalStrike,
    /// Le monstre résiste mieux au vieillissement.
    Longevity,
    /// Le monstre gagne plus d'XP.
    FastLearner,
    /// Le monstre inflige des dégâts en retour quand il est touché.
    Thorns,
    /// Le monstre est plus puissant quand ses PV sont bas.
    Berserk,
    /// Le monstre a une petite chance de survivre un coup fatal avec 1 PV.
    Tenacity,
}

impl fmt::Display for Trait {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Trait::Regeneration => "Régénération",
            Trait::Evasion => "Évasion",
            Trait::CriticalStrike => "Coup Critique+",
            Trait::Longevity => "Longévité",
            Trait::FastLearner => "Apprentissage Rapide",
            Trait::Thorns => "Épines",
            Trait::Berserk => "Berserk",
            Trait::Tenacity => "Ténacité",
        };
        write!(f, "{}", name)
    }
}
