use serde::{Deserialize, Serialize};
use std::fmt;

/// Types de nourriture disponibles pour nourrir un monstre.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FoodType {
    /// Baie basique — restaure la faim normalement.
    Berry,
    /// Viande — boost temporaire d'attaque.
    Meat,
    /// Poisson — boost temporaire de vitesse.
    Fish,
    /// Herbes — soigne un malus de bonheur.
    Herbs,
    /// Gâteau — gros boost de bonheur mais risque de suralimentation.
    Cake,
}

impl FoodType {
    /// Toutes les variantes de nourriture.
    pub fn all() -> &'static [FoodType] {
        &[
            FoodType::Berry,
            FoodType::Meat,
            FoodType::Fish,
            FoodType::Herbs,
            FoodType::Cake,
        ]
    }

    /// Icône emoji pour le type de nourriture.
    pub fn icon(&self) -> &'static str {
        match self {
            FoodType::Berry => "🫐",
            FoodType::Meat => "🥩",
            FoodType::Fish => "🐟",
            FoodType::Herbs => "🌿",
            FoodType::Cake => "🍰",
        }
    }

    /// Bonus de bonheur accordé par cet aliment.
    pub fn happiness_bonus(&self) -> i32 {
        match self {
            FoodType::Berry => 5,
            FoodType::Meat => 8,
            FoodType::Fish => 8,
            FoodType::Herbs => 15,
            FoodType::Cake => 25,
        }
    }

    /// Nombre de repas que cet aliment compte (pour le calcul overfed).
    pub fn meal_weight(&self) -> u32 {
        match self {
            FoodType::Berry => 1,
            FoodType::Meat => 1,
            FoodType::Fish => 1,
            FoodType::Herbs => 0, // Les herbes ne remplissent pas vraiment
            FoodType::Cake => 2,  // Le gâteau compte double
        }
    }
}

impl fmt::Display for FoodType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            FoodType::Berry => "Baie",
            FoodType::Meat => "Viande",
            FoodType::Fish => "Poisson",
            FoodType::Herbs => "Herbes",
            FoodType::Cake => "Gâteau",
        };
        write!(f, "{}", name)
    }
}

/// Niveau de bonheur d'un monstre.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HappinessLevel {
    /// Très malheureux (0–19).
    Miserable,
    /// Malheureux (20–39).
    Sad,
    /// Neutre (40–59).
    Neutral,
    /// Content (60–79).
    Happy,
    /// Très heureux (80–100).
    Joyful,
}

impl HappinessLevel {
    /// Calcule le niveau de bonheur à partir de la valeur brute (0–100).
    pub fn from_value(value: u32) -> Self {
        match value {
            0..=19 => HappinessLevel::Miserable,
            20..=39 => HappinessLevel::Sad,
            40..=59 => HappinessLevel::Neutral,
            60..=79 => HappinessLevel::Happy,
            _ => HappinessLevel::Joyful,
        }
    }

    /// Icône emoji pour le niveau de bonheur.
    pub fn icon(&self) -> &'static str {
        match self {
            HappinessLevel::Miserable => "😭",
            HappinessLevel::Sad => "😢",
            HappinessLevel::Neutral => "😐",
            HappinessLevel::Happy => "😊",
            HappinessLevel::Joyful => "🥰",
        }
    }

    /// Multiplicateur de stats lié au bonheur.
    pub fn stat_multiplier(&self) -> f64 {
        match self {
            HappinessLevel::Miserable => 0.85,
            HappinessLevel::Sad => 0.93,
            HappinessLevel::Neutral => 1.00,
            HappinessLevel::Happy => 1.05,
            HappinessLevel::Joyful => 1.10,
        }
    }

    /// Multiplicateur d'XP lié au bonheur.
    pub fn xp_multiplier(&self) -> f64 {
        match self {
            HappinessLevel::Miserable => 0.80,
            HappinessLevel::Sad => 0.90,
            HappinessLevel::Neutral => 1.00,
            HappinessLevel::Happy => 1.10,
            HappinessLevel::Joyful => 1.25,
        }
    }
}

impl fmt::Display for HappinessLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            HappinessLevel::Miserable => "Misérable",
            HappinessLevel::Sad => "Triste",
            HappinessLevel::Neutral => "Neutre",
            HappinessLevel::Happy => "Content",
            HappinessLevel::Joyful => "Joyeux",
        };
        write!(f, "{}", name)
    }
}

/// Niveau de lien/affection entre le joueur et son monstre.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BondLevel {
    /// Lien naissant (0–24).
    Stranger,
    /// Lien amical (25–49).
    Companion,
    /// Lien fort (50–74).
    Partner,
    /// Lien profond (75–99).
    SoulBond,
    /// Lien maximal (100).
    Eternal,
}

impl BondLevel {
    /// Calcule le niveau de lien à partir de la valeur brute (0–100).
    pub fn from_value(value: u32) -> Self {
        match value {
            0..=24 => BondLevel::Stranger,
            25..=49 => BondLevel::Companion,
            50..=74 => BondLevel::Partner,
            75..=99 => BondLevel::SoulBond,
            _ => BondLevel::Eternal,
        }
    }

    /// Icône emoji pour le niveau de lien.
    pub fn icon(&self) -> &'static str {
        match self {
            BondLevel::Stranger => "🤝",
            BondLevel::Companion => "💛",
            BondLevel::Partner => "🧡",
            BondLevel::SoulBond => "❤️",
            BondLevel::Eternal => "💎",
        }
    }

    /// Bonus de survie à un coup fatal (en plus de Tenacity) grâce au lien.
    pub fn survival_chance(&self) -> f64 {
        match self {
            BondLevel::Stranger => 0.0,
            BondLevel::Companion => 0.0,
            BondLevel::Partner => 0.05,
            BondLevel::SoulBond => 0.10,
            BondLevel::Eternal => 0.15,
        }
    }

    /// Multiplicateur de stats en reproduction grâce au lien.
    pub fn breeding_bonus(&self) -> f64 {
        match self {
            BondLevel::Stranger => 1.0,
            BondLevel::Companion => 1.0,
            BondLevel::Partner => 1.02,
            BondLevel::SoulBond => 1.05,
            BondLevel::Eternal => 1.10,
        }
    }

    /// Titre affiché à côté du nom du monstre quand le lien est suffisant.
    pub fn title(&self) -> Option<&'static str> {
        match self {
            BondLevel::Stranger => None,
            BondLevel::Companion => Some("Compagnon"),
            BondLevel::Partner => Some("Partenaire"),
            BondLevel::SoulBond => Some("Âme liée"),
            BondLevel::Eternal => Some("Lien Éternel"),
        }
    }
}

impl fmt::Display for BondLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            BondLevel::Stranger => "Inconnu",
            BondLevel::Companion => "Compagnon",
            BondLevel::Partner => "Partenaire",
            BondLevel::SoulBond => "Âme liée",
            BondLevel::Eternal => "Lien Éternel",
        };
        write!(f, "{}", name)
    }
}

/// Événement aléatoire pouvant survenir lors de la consultation d'un monstre.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RandomEvent {
    /// Le monstre a trouvé de la nourriture.
    FoundFood(FoodType),
    /// Le monstre s'est entraîné seul (bonus de stats).
    SoloTraining,
    /// Le monstre fait un cauchemar (perte de bonheur).
    Nightmare,
    /// Le monstre est de très bonne humeur aujourd'hui.
    GoodMood,
    /// Le monstre a eu une illumination (bonus d'XP).
    Epiphany,
    /// Le monstre a trouvé un trésor (bonus de lien).
    TreasureFound,
}

impl RandomEvent {
    /// Icône emoji de l'événement.
    pub fn icon(&self) -> &'static str {
        match self {
            RandomEvent::FoundFood(_) => "🎁",
            RandomEvent::SoloTraining => "💪",
            RandomEvent::Nightmare => "😱",
            RandomEvent::GoodMood => "🌟",
            RandomEvent::Epiphany => "💡",
            RandomEvent::TreasureFound => "💎",
        }
    }

    /// Description de l'événement.
    pub fn description(&self, monster_name: &str) -> String {
        match self {
            RandomEvent::FoundFood(food) => {
                format!(
                    "{} {} a trouvé {} {} !",
                    self.icon(),
                    monster_name,
                    food.icon(),
                    food
                )
            }
            RandomEvent::SoloTraining => {
                format!(
                    "{} {} s'est entraîné seul et a gagné en puissance !",
                    self.icon(),
                    monster_name
                )
            }
            RandomEvent::Nightmare => {
                format!(
                    "{} {} a fait un cauchemar cette nuit...",
                    self.icon(),
                    monster_name
                )
            }
            RandomEvent::GoodMood => {
                format!(
                    "{} {} est de très bonne humeur aujourd'hui !",
                    self.icon(),
                    monster_name
                )
            }
            RandomEvent::Epiphany => {
                format!(
                    "{} {} a eu une illumination mystérieuse !",
                    self.icon(),
                    monster_name
                )
            }
            RandomEvent::TreasureFound => {
                format!(
                    "{} {} a trouvé un trésor caché !",
                    self.icon(),
                    monster_name
                )
            }
        }
    }
}

/// Types élémentaires des monstres.
/// Chaque monstre possède un type primaire et optionnellement un type secondaire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ElementType {
    /// Type neutre (utilisé uniquement pour les attaques).
    Normal,
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
    /// Retourne toutes les variantes du type élémentaire (hors Normal).
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
            // Normal : toujours neutre
            (Normal, _) | (_, Normal) => 1.0,

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
            ElementType::Normal => "⭐",
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
            ElementType::Normal => "Normal",
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
    /// Glouton : ignore l'état rassasié pour la dévoration.
    Gluttony,
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
            Trait::Gluttony => "Glouton",
        };
        write!(f, "{}", name)
    }
}
