use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

use crate::types::{ElementType, Stats, Trait};

/// Durée de vie maximale d'un monstre en jours (sans le trait Longévité).
const BASE_MAX_AGE_DAYS: i64 = 30;

/// Bonus de longévité en jours.
const LONGEVITY_BONUS_DAYS: i64 = 15;

/// Nombre de jours sans manger avant la mort de faim.
const STARVATION_DAYS: i64 = 3;

/// Nombre d'heures pendant lesquelles le monstre est rassasié après avoir mangé.
const SATISFIED_HOURS: i64 = 12;

/// Stade de vie d'un monstre, basé sur son âge relatif à sa durée de vie max.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgeStage {
    /// 0–15 % de la durée de vie : le monstre vient d'éclore.
    Baby,
    /// 15–40 % : il grandit et apprend.
    Young,
    /// 40–75 % : pleine maturité, pic de puissance.
    Adult,
    /// 75–100 % : le déclin s'installe.
    Old,
}

impl AgeStage {
    /// Icône emoji pour le stade de vie.
    pub fn icon(&self) -> &'static str {
        match self {
            AgeStage::Baby => "💳",
            AgeStage::Young => "🌱",
            AgeStage::Adult => "💪",
            AgeStage::Old => "🧓",
        }
    }

    /// Multiplicateur global de stats pour ce stade de vie.
    /// Bébé = plus faible, Adulte = pic, Vieux = déclin.
    pub fn stat_multiplier(&self) -> f64 {
        match self {
            AgeStage::Baby => 0.80,
            AgeStage::Young => 0.95,
            AgeStage::Adult => 1.10,
            AgeStage::Old => 0.85,
        }
    }
}

impl fmt::Display for AgeStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            AgeStage::Baby => "Bébé",
            AgeStage::Young => "Jeune",
            AgeStage::Adult => "Adulte",
            AgeStage::Old => "Vieux",
        };
        write!(f, "{}", name)
    }
}

/// Niveau de faim d'un monstre.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HungerLevel {
    /// Le monstre n'a pas mangé depuis longtemps — en danger de mort.
    Starving,
    /// Le monstre a faim — stats normales.
    Hungry,
    /// Le monstre est rassasié — bonus de stats.
    Satisfied,
    /// Le monstre a trop mangé — malus de stats.
    Overfed,
}

impl HungerLevel {
    /// Icône emoji pour le niveau de faim.
    pub fn icon(&self) -> &'static str {
        match self {
            HungerLevel::Starving => "💀",
            HungerLevel::Hungry => "🍽️",
            HungerLevel::Satisfied => "😊",
            HungerLevel::Overfed => "🤢",
        }
    }

    /// Multiplicateur de stats lié à la faim.
    /// Starving = gros malus, Hungry = neutre, Satisfied = boost, Overfed = malus.
    pub fn stat_multiplier(&self) -> f64 {
        match self {
            HungerLevel::Starving => 0.70,
            HungerLevel::Hungry => 1.00,
            HungerLevel::Satisfied => 1.15,
            HungerLevel::Overfed => 0.85,
        }
    }
}

impl fmt::Display for HungerLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            HungerLevel::Starving => "Affamé",
            HungerLevel::Hungry => "A faim",
            HungerLevel::Satisfied => "Rassasié",
            HungerLevel::Overfed => "Trop mangé",
        };
        write!(f, "{}", name)
    }
}

/// Représente un monstre unique et irremplaçable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monster {
    /// Identifiant unique (ne change jamais).
    pub id: Uuid,

    /// Nom donné par le joueur.
    pub name: String,

    /// Type élémentaire primaire.
    pub primary_type: ElementType,

    /// Type élémentaire secondaire (optionnel, obtenu par reproduction).
    pub secondary_type: Option<ElementType>,

    /// Statistiques de base (génétiques, ne changent pas avec le level).
    pub base_stats: Stats,

    /// Niveau actuel (1–100).
    pub level: u32,

    /// Points d'expérience actuels.
    pub xp: u32,

    /// Points de vie actuels (en combat ou hors combat).
    pub current_hp: u32,

    /// Traits génétiques (hérités ou mutés).
    pub traits: Vec<Trait>,

    /// Date et heure de naissance (UTC).
    pub born_at: DateTime<Utc>,

    /// Date et heure de mort, si le monstre est mort.
    pub died_at: Option<DateTime<Utc>>,

    /// Identifiants des parents (None si monstre "sauvage" / starter).
    pub parent_a: Option<Uuid>,
    pub parent_b: Option<Uuid>,

    /// Nombre de combats gagnés au total.
    pub wins: u32,

    /// Nombre de combats perdus au total.
    pub losses: u32,

    /// Génération (0 = starter, 1 = enfant de starters, etc.).
    pub generation: u32,

    /// Dernière fois que le monstre a été nourri (UTC). None = jamais nourri.
    #[serde(default)]
    pub last_fed: Option<DateTime<Utc>>,

    /// Nombre de repas pris dans les 12 dernières heures.
    #[serde(default)]
    pub meals_today: u32,

    /// Horodatage du premier repas de la fenêtre actuelle (pour reset du compteur).
    #[serde(default)]
    pub meals_window_start: Option<DateTime<Utc>>,
}

impl Monster {
    /// Crée un nouveau monstre starter (sans parents).
    pub fn new_starter(name: String, primary_type: ElementType, base_stats: Stats) -> Self {
        let max_hp = base_stats.hp;
        Self {
            id: Uuid::new_v4(),
            name,
            primary_type,
            secondary_type: None,
            base_stats,
            level: 1,
            xp: 0,
            current_hp: max_hp,
            traits: Vec::new(),
            born_at: Utc::now(),
            died_at: None,
            parent_a: None,
            parent_b: None,
            wins: 0,
            losses: 0,
            generation: 0,
            last_fed: Some(Utc::now()),
            meals_today: 0,
            meals_window_start: None,
        }
    }

    /// Retourne `true` si le monstre est mort (combat ou vieillesse).
    pub fn is_dead(&self) -> bool {
        self.died_at.is_some()
    }

    /// Retourne `true` si le monstre est vivant.
    pub fn is_alive(&self) -> bool {
        !self.is_dead()
    }

    /// Retourne l'âge du monstre en jours.
    pub fn age_days(&self) -> i64 {
        let end = self.died_at.unwrap_or_else(Utc::now);
        (end - self.born_at).num_days()
    }

    /// Retourne l'âge maximum en jours pour ce monstre.
    pub fn max_age_days(&self) -> i64 {
        let base = BASE_MAX_AGE_DAYS;
        if self.traits.contains(&Trait::Longevity) {
            base + LONGEVITY_BONUS_DAYS
        } else {
            base
        }
    }

    /// Retourne le stade de vie actuel du monstre.
    pub fn age_stage(&self) -> AgeStage {
        let ratio = self.age_days() as f64 / self.max_age_days() as f64;
        if ratio < 0.15 {
            AgeStage::Baby
        } else if ratio < 0.40 {
            AgeStage::Young
        } else if ratio < 0.75 {
            AgeStage::Adult
        } else {
            AgeStage::Old
        }
    }

    /// Retourne le pourcentage de vie écoulée (0.0 – 1.0).
    pub fn age_ratio(&self) -> f64 {
        (self.age_days() as f64 / self.max_age_days() as f64).clamp(0.0, 1.0)
    }

    /// Vérifie si le monstre devrait mourir de vieillesse et le tue le cas échéant.
    /// Retourne `true` si le monstre vient de mourir.
    pub fn check_aging(&mut self) -> bool {
        if self.is_dead() {
            return false;
        }
        if self.age_days() >= self.max_age_days() {
            self.died_at = Some(Utc::now());
            true
        } else {
            false
        }
    }

    // ── Système de faim ─────────────────────────────────────────

    /// Retourne le niveau de faim actuel du monstre.
    pub fn hunger_level(&self) -> HungerLevel {
        if self.is_dead() {
            return HungerLevel::Hungry;
        }

        let now = Utc::now();

        // Heures depuis le dernier repas
        let hours_since_fed = match self.last_fed {
            Some(fed) => (now - fed).num_hours(),
            None => {
                // Jamais nourri → compter depuis la naissance
                (now - self.born_at).num_hours()
            }
        };

        let days_since_fed = hours_since_fed / 24;

        // Mort de faim après STARVATION_DAYS jours
        if days_since_fed >= STARVATION_DAYS {
            return HungerLevel::Starving;
        }

        // Vérifier si on a trop mangé (3+ repas en 12h)
        if self.meals_today >= 3 {
            if let Some(window_start) = self.meals_window_start {
                if (now - window_start).num_hours() < SATISFIED_HOURS {
                    return HungerLevel::Overfed;
                }
            }
        }

        // Rassasié si nourri dans les SATISFIED_HOURS dernières heures
        if hours_since_fed < SATISFIED_HOURS {
            return HungerLevel::Satisfied;
        }

        // Sinon, il a faim
        HungerLevel::Hungry
    }

    /// Vérifie si le monstre devrait mourir de faim et le tue le cas échéant.
    /// Retourne `true` si le monstre vient de mourir de faim.
    pub fn check_hunger(&mut self) -> bool {
        if self.is_dead() {
            return false;
        }
        if self.hunger_level() == HungerLevel::Starving {
            self.died_at = Some(Utc::now());
            true
        } else {
            false
        }
    }

    /// Nourrit le monstre. Retourne le nouveau niveau de faim.
    pub fn feed(&mut self) -> HungerLevel {
        if self.is_dead() {
            return HungerLevel::Hungry;
        }

        let now = Utc::now();

        // Reset le compteur de repas si la fenêtre a expiré (12h)
        if let Some(window_start) = self.meals_window_start {
            if (now - window_start).num_hours() >= SATISFIED_HOURS {
                self.meals_today = 0;
                self.meals_window_start = None;
            }
        }

        // Démarrer une nouvelle fenêtre si nécessaire
        if self.meals_window_start.is_none() {
            self.meals_window_start = Some(now);
        }

        self.last_fed = Some(now);
        self.meals_today += 1;

        self.hunger_level()
    }

    /// Heures depuis le dernier repas.
    pub fn hours_since_fed(&self) -> i64 {
        let now = Utc::now();
        match self.last_fed {
            Some(fed) => (now - fed).num_hours(),
            None => (now - self.born_at).num_hours(),
        }
    }

    /// PV max effectifs (stats de base × facteur de niveau × facteur d'âge).
    /// Note : les PV max ne sont PAS affectés par la faim pour éviter une spirale de mort.
    pub fn max_hp(&self) -> u32 {
        let raw = self.base_stats.hp + (self.level * 2);
        (raw as f64 * self.age_stage().stat_multiplier()) as u32
    }

    /// Attaque effective (stats de base × facteur de niveau × facteur d'âge × faim).
    pub fn effective_attack(&self) -> u32 {
        let raw = self.base_stats.attack + (self.level / 2);
        (raw as f64 * self.age_stage().stat_multiplier() * self.hunger_level().stat_multiplier())
            as u32
    }

    /// Défense effective (facteur d'âge × faim inclus).
    pub fn effective_defense(&self) -> u32 {
        let raw = self.base_stats.defense + (self.level / 2);
        (raw as f64 * self.age_stage().stat_multiplier() * self.hunger_level().stat_multiplier())
            as u32
    }

    /// Vitesse effective (facteur d'âge × faim inclus).
    pub fn effective_speed(&self) -> u32 {
        let raw = self.base_stats.speed + (self.level / 3);
        (raw as f64 * self.age_stage().stat_multiplier() * self.hunger_level().stat_multiplier())
            as u32
    }

    /// Attaque spéciale effective (facteur d'âge × faim inclus).
    pub fn effective_sp_attack(&self) -> u32 {
        let raw = self.base_stats.special_attack + (self.level / 2);
        (raw as f64 * self.age_stage().stat_multiplier() * self.hunger_level().stat_multiplier())
            as u32
    }

    /// Défense spéciale effective (facteur d'âge × faim inclus).
    pub fn effective_sp_defense(&self) -> u32 {
        let raw = self.base_stats.special_defense + (self.level / 2);
        (raw as f64 * self.age_stage().stat_multiplier() * self.hunger_level().stat_multiplier())
            as u32
    }

    /// XP nécessaire pour passer au niveau suivant.
    pub fn xp_to_next_level(&self) -> u32 {
        self.level * self.level * 10
    }

    /// Ajoute de l'XP et gère le level up. Retourne le nombre de niveaux gagnés.
    pub fn gain_xp(&mut self, amount: u32) -> u32 {
        if self.is_dead() {
            return 0;
        }

        let xp_multiplier = if self.traits.contains(&Trait::FastLearner) {
            1.5
        } else {
            1.0
        };

        self.xp += (amount as f64 * xp_multiplier) as u32;
        let mut levels_gained = 0;

        while self.level < 100 && self.xp >= self.xp_to_next_level() {
            self.xp -= self.xp_to_next_level();
            self.level += 1;
            levels_gained += 1;
            // Restaure les PV au level up
            self.current_hp = self.max_hp();
        }

        levels_gained
    }

    /// Soigne le monstre de `amount` PV (sans dépasser le max).
    pub fn heal(&mut self, amount: u32) {
        if self.is_alive() {
            self.current_hp = (self.current_hp + amount).min(self.max_hp());
        }
    }

    /// Inflige des dégâts au monstre. Retourne les dégâts réellement infligés.
    /// Si les PV tombent à 0, le monstre meurt.
    pub fn take_damage(&mut self, raw_damage: u32) -> u32 {
        if self.is_dead() {
            return 0;
        }

        // Tenacity : chance de survivre avec 1 PV
        let actual_damage = raw_damage.min(self.current_hp);

        if actual_damage >= self.current_hp {
            if self.traits.contains(&Trait::Tenacity) && rand::random::<f64>() < 0.15 {
                // Survit avec 1 PV
                let dmg = self.current_hp - 1;
                self.current_hp = 1;
                return dmg;
            }
            self.current_hp = 0;
            self.died_at = Some(Utc::now());
            return actual_damage;
        }

        self.current_hp -= actual_damage;
        actual_damage
    }

    /// Retourne un résumé textuel du monstre pour affichage.
    pub fn summary(&self) -> String {
        let status = if self.is_dead() {
            "💀 MORT"
        } else {
            "💚 Vivant"
        };
        let types = match &self.secondary_type {
            Some(sec) => format!("{} / {}", self.primary_type, sec),
            None => format!("{}", self.primary_type),
        };
        let stage = self.age_stage();
        let hunger = self.hunger_level();
        format!(
            "{} [{}] — Nv.{} — {} — PV: {}/{} — {} {} ({}j/{}j) — {} {} — {}",
            self.name,
            types,
            self.level,
            status,
            self.current_hp,
            self.max_hp(),
            stage.icon(),
            stage,
            self.age_days(),
            self.max_age_days(),
            hunger.icon(),
            hunger,
            if self.traits.is_empty() {
                "Aucun trait".to_string()
            } else {
                self.traits
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        )
    }
}
