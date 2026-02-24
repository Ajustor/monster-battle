use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{ElementType, Stats, Trait};

/// Durée de vie maximale d'un monstre en jours (sans le trait Longévité).
const BASE_MAX_AGE_DAYS: i64 = 30;

/// Bonus de longévité en jours.
const LONGEVITY_BONUS_DAYS: i64 = 15;

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

    /// PV max effectifs (stats de base × facteur de niveau).
    pub fn max_hp(&self) -> u32 {
        self.base_stats.hp + (self.level * 2)
    }

    /// Attaque effective (stats de base × facteur de niveau).
    pub fn effective_attack(&self) -> u32 {
        self.base_stats.attack + (self.level / 2)
    }

    /// Défense effective.
    pub fn effective_defense(&self) -> u32 {
        self.base_stats.defense + (self.level / 2)
    }

    /// Vitesse effective.
    pub fn effective_speed(&self) -> u32 {
        self.base_stats.speed + (self.level / 3)
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
        format!(
            "{} [{}] — Nv.{} — {} — PV: {}/{} — Âge: {}j/{}j — {}",
            self.name,
            types,
            self.level,
            status,
            self.current_hp,
            self.max_hp(),
            self.age_days(),
            self.max_age_days(),
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
