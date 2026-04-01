use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

use rand::Rng;

use crate::types::{BondLevel, ElementType, FoodType, HappinessLevel, RandomEvent, Stats, Trait};

/// Durée de vie maximale d'un monstre en jours (sans le trait Longévité).
const BASE_MAX_AGE_DAYS: i64 = 30;

/// Bonus de longévité en jours.
const LONGEVITY_BONUS_DAYS: i64 = 15;

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

/// Résultat d'une dévoration après victoire en combat.
#[derive(Debug, Clone)]
pub struct DevourResult {
    /// Gains de stats obtenus.
    pub stat_gains: Stats,
    /// Vrai si une mutation de trait s'est produite.
    pub mutation_occurred: bool,
    /// Nouveau type secondaire acquis (si applicable).
    pub new_secondary_type: Option<ElementType>,
    /// Description narrative de la dévoration.
    pub description: String,
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

    /// Bonheur du monstre (0–100). Affecte les stats et l'XP.
    #[serde(default = "default_happiness")]
    pub happiness: u32,

    /// Lien / affection entre le joueur et le monstre (0–100). Ne descend jamais.
    #[serde(default)]
    pub bond: u32,

    /// Buff temporaire de nourriture actif (type + expiration).
    #[serde(default)]
    pub food_buff: Option<(FoodType, DateTime<Utc>)>,

    /// Dernière interaction avec le monstre (pour la baisse passive de bonheur).
    #[serde(default)]
    pub last_interaction: Option<DateTime<Utc>>,

    /// Dernier événement aléatoire vérifié (pour limiter la fréquence).
    #[serde(default)]
    pub last_event_check: Option<DateTime<Utc>>,

    /// Indices des 4 attaques actives dans la liste complète des attaques connues.
    /// Si vide, les 4 premières attaques connues sont utilisées.
    #[serde(default)]
    pub active_attack_indices: Vec<usize>,
}

fn default_happiness() -> u32 {
    50
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
            happiness: 50,
            bond: 0,
            food_buff: None,
            last_interaction: Some(Utc::now()),
            last_event_check: None,
            active_attack_indices: Vec::new(),
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

        // Starving après 3 jours sans manger (malus de stats, mais pas de mort)
        if days_since_fed >= 3 {
            return HungerLevel::Starving;
        }

        // Vérifier si on a trop mangé (3+ repas en 12h)
        if self.meals_today >= 3
            && let Some(window_start) = self.meals_window_start
                && (now - window_start).num_hours() < SATISFIED_HOURS {
                    return HungerLevel::Overfed;
                }

        // Rassasié si nourri dans les SATISFIED_HOURS dernières heures
        if hours_since_fed < SATISFIED_HOURS {
            return HungerLevel::Satisfied;
        }

        // Sinon, il a faim
        HungerLevel::Hungry
    }

    /// Vérifie le niveau de faim. Le monstre NE meurt plus de faim (malus de stats uniquement).
    /// Retourne `true` si le monstre est en état de famine (pour affichage d'avertissement).
    pub fn check_hunger(&mut self) -> bool {
        if self.is_dead() {
            return false;
        }
        self.hunger_level() == HungerLevel::Starving
    }

    // ── Système d'attaques actives ──────────────────────────────

    /// Retourne toutes les attaques connues par le monstre.
    pub fn known_attacks(&self) -> Vec<crate::attack::Attack> {
        crate::attack::Attack::all_attacks_for_type(self.primary_type, self.secondary_type)
    }

    /// Retourne les 4 attaques actives du monstre.
    /// Utilise `active_attack_indices` si défini, sinon les 4 premières.
    pub fn active_attacks(&self) -> Vec<crate::attack::Attack> {
        let known = self.known_attacks();
        if self.active_attack_indices.is_empty() {
            known.into_iter().take(4).collect()
        } else {
            self.active_attack_indices
                .iter()
                .filter_map(|&i| known.get(i).cloned())
                .take(4)
                .collect()
        }
    }

    /// Définit les indices des attaques actives (max 4, indices valides).
    /// Retourne une erreur si les indices sont invalides ou si plus de 4 sont fournis.
    pub fn set_active_attacks(&mut self, indices: Vec<usize>) -> Result<(), String> {
        if indices.len() > 4 {
            return Err(format!(
                "Trop d'attaques sélectionnées : {} (max 4)",
                indices.len()
            ));
        }
        let known_count = self.known_attacks().len();
        for &idx in &indices {
            if idx >= known_count {
                return Err(format!(
                    "Index d'attaque invalide : {} (max {})",
                    idx,
                    known_count.saturating_sub(1)
                ));
            }
        }
        self.active_attack_indices = indices;
        Ok(())
    }

    // ── Système de dévoration ───────────────────────────────────

    /// Résultat d'une dévoration après victoire en combat.
    pub fn devour(&mut self, prey: &Monster) -> DevourResult {
        let mut rng = rand::thread_rng();

        // Calcul des gains de stats (moitié des stats de prey, min 1 si > 0)
        let gain_stat = |v: u32| -> u32 {
            if v > 0 { (v / 2).max(1) } else { 0 }
        };
        let stat_gains = Stats {
            hp: gain_stat(prey.base_stats.hp),
            attack: gain_stat(prey.base_stats.attack),
            defense: gain_stat(prey.base_stats.defense),
            speed: gain_stat(prey.base_stats.speed),
            special_attack: gain_stat(prey.base_stats.special_attack),
            special_defense: gain_stat(prey.base_stats.special_defense),
        };

        // Appliquer les gains
        self.base_stats.hp += stat_gains.hp;
        self.base_stats.attack += stat_gains.attack;
        self.base_stats.defense += stat_gains.defense;
        self.base_stats.speed += stat_gains.speed;
        self.base_stats.special_attack += stat_gains.special_attack;
        self.base_stats.special_defense += stat_gains.special_defense;

        // Mutation de trait (20%)
        let mutation_occurred = rng.gen_bool(0.20);
        if mutation_occurred {
            // Tire un trait aléatoire de prey ou un trait aléatoire parmi tous les Trait
            let all_traits = [
                Trait::Regeneration,
                Trait::Evasion,
                Trait::CriticalStrike,
                Trait::Longevity,
                Trait::FastLearner,
                Trait::Thorns,
                Trait::Berserk,
                Trait::Tenacity,
            ];
            // 50% chance de prendre un trait du prey, 50% trait aléatoire global
            let new_trait = if !prey.traits.is_empty() && rng.gen_bool(0.5) {
                prey.traits[rng.gen_range(0..prey.traits.len())].clone()
            } else {
                all_traits[rng.gen_range(0..all_traits.len())].clone()
            };
            if !self.traits.contains(&new_trait) {
                self.traits.push(new_trait);
            }
        }

        // Type secondaire par dévoration (30%) — seulement si pas encore de type secondaire
        let new_secondary_type = if self.secondary_type.is_none()
            && prey.primary_type != self.primary_type
            && rng.gen_bool(0.30)
        {
            self.secondary_type = Some(prey.primary_type);
            Some(prey.primary_type)
        } else {
            None
        };

        self.record_interaction();

        // Description
        let mut desc_parts = vec![format!(
            "{} a dévoré {} et gagné +{} PV, +{} ATQ, +{} DEF, +{} VIT, +{} ATQ.SP, +{} DEF.SP !",
            self.name,
            prey.name,
            stat_gains.hp,
            stat_gains.attack,
            stat_gains.defense,
            stat_gains.speed,
            stat_gains.special_attack,
            stat_gains.special_defense,
        )];
        if mutation_occurred
            && let Some(t) = self.traits.last() {
                desc_parts.push(format!("🧬 Mutation ! Nouveau trait : {} !", t));
            }
        if let Some(st) = new_secondary_type {
            desc_parts.push(format!("🌀 {} absorbe le type {} !", self.name, st));
        }

        DevourResult {
            stat_gains,
            mutation_occurred,
            new_secondary_type,
            description: desc_parts.join("\n"),
        }
    }

    /// Nourrit le monstre avec un type de nourriture. Retourne le nouveau niveau de faim.
    pub fn feed_with(&mut self, food: FoodType) -> HungerLevel {
        if self.is_dead() {
            return HungerLevel::Hungry;
        }

        let now = Utc::now();

        // Reset le compteur de repas si la fenêtre a expiré (12h)
        if let Some(window_start) = self.meals_window_start
            && (now - window_start).num_hours() >= SATISFIED_HOURS {
                self.meals_today = 0;
                self.meals_window_start = None;
            }

        // Démarrer une nouvelle fenêtre si nécessaire
        if self.meals_window_start.is_none() {
            self.meals_window_start = Some(now);
        }

        self.last_fed = Some(now);
        self.meals_today += food.meal_weight();

        // Appliquer le buff temporaire de nourriture (dure 1h)
        match food {
            FoodType::Meat | FoodType::Fish => {
                self.food_buff = Some((food, now + chrono::Duration::hours(1)));
            }
            _ => {}
        }

        // Bonheur : la nourriture rend heureux !
        let happiness_bonus = food.happiness_bonus();
        // Mais si le monstre est suralimenté, le bonheur baisse
        let hunger = self.hunger_level();
        if hunger == HungerLevel::Overfed {
            self.adjust_happiness(-5);
        } else {
            self.adjust_happiness(happiness_bonus);
        }

        // Interaction → lien
        self.record_interaction();

        hunger
    }

    /// Nourrit le monstre (compatibilité ancienne — utilise une baie).
    pub fn feed(&mut self) -> HungerLevel {
        self.feed_with(FoodType::Berry)
    }

    /// Heures depuis le dernier repas.
    pub fn hours_since_fed(&self) -> i64 {
        let now = Utc::now();
        match self.last_fed {
            Some(fed) => (now - fed).num_hours(),
            None => (now - self.born_at).num_hours(),
        }
    }

    // ── Système de bonheur ──────────────────────────────────

    /// Retourne le niveau de bonheur actuel.
    pub fn happiness_level(&self) -> HappinessLevel {
        HappinessLevel::from_value(self.happiness)
    }

    /// Ajuste le bonheur (positif ou négatif), borné à 0–100.
    pub fn adjust_happiness(&mut self, delta: i32) {
        let new_val = (self.happiness as i32 + delta).clamp(0, 100);
        self.happiness = new_val as u32;
    }

    /// Applique la baisse passive de bonheur (appelé périodiquement).
    /// Le bonheur baisse de 1 pour chaque heure sans interaction.
    pub fn decay_happiness(&mut self) {
        if self.is_dead() {
            return;
        }
        let now = Utc::now();
        let hours_since = match self.last_interaction {
            Some(last) => (now - last).num_hours(),
            None => (now - self.born_at).num_hours(),
        };
        // Perd 1 de bonheur pour chaque 2h sans interaction, min 2
        let decay = ((hours_since / 2) as i32).clamp(0, 5);
        if decay > 0 {
            self.adjust_happiness(-decay);
        }

        // La faim rend malheureux
        match self.hunger_level() {
            HungerLevel::Starving => self.adjust_happiness(-10),
            HungerLevel::Hungry => self.adjust_happiness(-2),
            _ => {}
        }
    }

    // ── Système de lien ─────────────────────────────────────

    /// Retourne le niveau de lien actuel.
    pub fn bond_level(&self) -> BondLevel {
        BondLevel::from_value(self.bond)
    }

    /// Enregistre une interaction (nourrir, jouer, combattre...).
    /// Augmente le lien de 1 (ne descend jamais).
    pub fn record_interaction(&mut self) {
        self.last_interaction = Some(Utc::now());
        self.bond = (self.bond + 1).min(100);
    }

    /// Augmente le lien d'un montant spécifique.
    pub fn increase_bond(&mut self, amount: u32) {
        self.bond = (self.bond + amount).min(100);
    }

    // ── Buff de nourriture ──────────────────────────────────

    /// Retourne le buff de nourriture actif (s'il n'a pas expiré).
    pub fn active_food_buff(&self) -> Option<FoodType> {
        if let Some((food, expires)) = &self.food_buff {
            if Utc::now() < *expires {
                Some(*food)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Multiplicateur de buff de nourriture pour l'attaque.
    pub fn food_attack_multiplier(&self) -> f64 {
        match self.active_food_buff() {
            Some(FoodType::Meat) => 1.15,
            _ => 1.0,
        }
    }

    /// Multiplicateur de buff de nourriture pour la vitesse.
    pub fn food_speed_multiplier(&self) -> f64 {
        match self.active_food_buff() {
            Some(FoodType::Fish) => 1.15,
            _ => 1.0,
        }
    }

    // ── Événements aléatoires ───────────────────────────────

    /// Tente de déclencher un événement aléatoire.
    /// Retourne `Some(event)` si un événement se produit (max 1 par heure).
    pub fn try_random_event(&mut self) -> Option<RandomEvent> {
        if self.is_dead() {
            return None;
        }

        let now = Utc::now();

        // Limiter à 1 événement par heure
        if let Some(last_check) = self.last_event_check
            && (now - last_check).num_minutes() < 60 {
                return None;
            }

        self.last_event_check = Some(now);

        let mut rng = rand::thread_rng();
        use rand::Rng;

        // 30% de chance qu'un événement se produise
        if !rng.gen_bool(0.30) {
            return None;
        }

        // Choix pondéré de l'événement
        let roll: f64 = rng.r#gen();
        let event = if roll < 0.25 {
            // 25% : trouvé de la nourriture
            let foods = FoodType::all();
            let food = foods[rng.gen_range(0..foods.len())];
            RandomEvent::FoundFood(food)
        } else if roll < 0.40 {
            // 15% : entraînement solo
            RandomEvent::SoloTraining
        } else if roll < 0.55 {
            // 15% : cauchemar
            RandomEvent::Nightmare
        } else if roll < 0.70 {
            // 15% : bonne humeur
            RandomEvent::GoodMood
        } else if roll < 0.85 {
            // 15% : illumination
            RandomEvent::Epiphany
        } else {
            // 15% : trésor
            RandomEvent::TreasureFound
        };

        Some(event)
    }

    /// Applique les effets d'un événement aléatoire sur le monstre.
    /// Retourne un message descriptif.
    pub fn apply_event(&mut self, event: &RandomEvent) -> String {
        let desc = event.description(&self.name);
        match event {
            RandomEvent::FoundFood(food) => {
                self.feed_with(*food);
                format!("{} (nourri avec {} {})", desc, food.icon(), food)
            }
            RandomEvent::SoloTraining => {
                // Petit boost de stats aléatoire
                let mut rng = rand::thread_rng();
                use rand::Rng;
                match rng.gen_range(0..6) {
                    0 => self.base_stats.hp += 1,
                    1 => self.base_stats.attack += 1,
                    2 => self.base_stats.defense += 1,
                    3 => self.base_stats.speed += 1,
                    4 => self.base_stats.special_attack += 1,
                    _ => self.base_stats.special_defense += 1,
                }
                self.adjust_happiness(5);
                desc
            }
            RandomEvent::Nightmare => {
                self.adjust_happiness(-15);
                desc
            }
            RandomEvent::GoodMood => {
                self.adjust_happiness(20);
                desc
            }
            RandomEvent::Epiphany => {
                self.gain_xp(20);
                self.adjust_happiness(10);
                desc
            }
            RandomEvent::TreasureFound => {
                self.increase_bond(5);
                self.adjust_happiness(10);
                desc
            }
        }
    }

    /// PV max effectifs (stats de base × facteur de niveau × facteur d'âge).
    /// Note : les PV max ne sont PAS affectés par la faim/bonheur pour éviter une spirale de mort.
    pub fn max_hp(&self) -> u32 {
        let raw = (self.base_stats.hp + (self.level * 2)) * 2;
        (raw as f64 * self.age_stage().stat_multiplier()) as u32
    }

    /// Multiplicateur combiné de stats (âge × faim × bonheur).
    fn combined_stat_multiplier(&self) -> f64 {
        self.age_stage().stat_multiplier()
            * self.hunger_level().stat_multiplier()
            * self.happiness_level().stat_multiplier()
    }

    /// Attaque effective (stats de base × facteur de niveau × facteur d'âge × faim × bonheur × buff food).
    pub fn effective_attack(&self) -> u32 {
        let raw = self.base_stats.attack + (self.level / 2);
        (raw as f64 * self.combined_stat_multiplier() * self.food_attack_multiplier()) as u32
    }

    /// Défense effective (facteur d'âge × faim × bonheur inclus).
    pub fn effective_defense(&self) -> u32 {
        let raw = self.base_stats.defense + (self.level / 2);
        (raw as f64 * self.combined_stat_multiplier()) as u32
    }

    /// Vitesse effective (facteur d'âge × faim × bonheur × buff food inclus).
    pub fn effective_speed(&self) -> u32 {
        let raw = self.base_stats.speed + (self.level / 3);
        (raw as f64 * self.combined_stat_multiplier() * self.food_speed_multiplier()) as u32
    }

    /// Attaque spéciale effective (facteur d'âge × faim × bonheur inclus).
    pub fn effective_sp_attack(&self) -> u32 {
        let raw = self.base_stats.special_attack + (self.level / 2);
        (raw as f64 * self.combined_stat_multiplier()) as u32
    }

    /// Défense spéciale effective (facteur d'âge × faim × bonheur inclus).
    pub fn effective_sp_defense(&self) -> u32 {
        let raw = self.base_stats.special_defense + (self.level / 2);
        (raw as f64 * self.combined_stat_multiplier()) as u32
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

        let trait_multiplier = if self.traits.contains(&Trait::FastLearner) {
            1.5
        } else {
            1.0
        };

        let happiness_multiplier = self.happiness_level().xp_multiplier();

        self.xp += (amount as f64 * trait_multiplier * happiness_multiplier) as u32;
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
            // Tenacity : chance de survivre avec 1 PV
            if self.traits.contains(&Trait::Tenacity) && rand::random::<f64>() < 0.15 {
                let dmg = self.current_hp - 1;
                self.current_hp = 1;
                return dmg;
            }
            // Bond survival : chance de survivre grâce au lien
            let bond_chance = self.bond_level().survival_chance();
            if bond_chance > 0.0 && rand::random::<f64>() < bond_chance {
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
        let happiness = self.happiness_level();
        let bond = self.bond_level();
        let bond_title = bond
            .title()
            .map(|t| format!(" «{}»", t))
            .unwrap_or_default();
        format!(
            "{}{} [{}] — Nv.{} — {} — PV: {}/{} — {} {} ({}j/{}j) — {} {} — {} {} — {} {} — {}",
            self.name,
            bond_title,
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
            happiness.icon(),
            happiness,
            bond.icon(),
            bond,
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
