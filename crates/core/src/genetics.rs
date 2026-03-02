use chrono::Utc;
use rand::Rng;
use uuid::Uuid;

use crate::monster::Monster;
use crate::types::{ElementType, Stats, Trait};

/// Résultat d'une reproduction entre deux monstres.
pub struct BreedingResult {
    pub child: Monster,
    /// Indique si une mutation a eu lieu.
    pub mutation_occurred: bool,
    /// Description textuelle de ce qui s'est passé.
    pub description: String,
}

/// Chance de mutation d'un trait (en pourcentage).
const MUTATION_CHANCE: f64 = 0.15;

/// Chance d'obtenir un type secondaire lors de la reproduction.
const DUAL_TYPE_CHANCE: f64 = 0.40;

/// Fait se reproduire deux monstres et retourne un enfant.
/// Les deux parents doivent être vivants.
pub fn breed(
    parent_a: &Monster,
    parent_b: &Monster,
    child_name: String,
) -> Result<BreedingResult, String> {
    if parent_a.is_dead() {
        return Err(format!(
            "{} est mort et ne peut pas se reproduire.",
            parent_a.name
        ));
    }
    if parent_b.is_dead() {
        return Err(format!(
            "{} est mort et ne peut pas se reproduire.",
            parent_b.name
        ));
    }
    if parent_a.id == parent_b.id {
        return Err("Un monstre ne peut pas se reproduire avec lui-même.".to_string());
    }

    let mut rng = rand::thread_rng();

    // --- Type primaire : hérité d'un des parents ---
    let primary_type = if rng.gen_bool(0.5) {
        parent_a.primary_type
    } else {
        parent_b.primary_type
    };

    // --- Type secondaire : chance d'obtenir le type de l'autre parent ---
    let secondary_type = if rng.gen_bool(DUAL_TYPE_CHANCE) {
        let other_type = if primary_type == parent_a.primary_type {
            parent_b.primary_type
        } else {
            parent_a.primary_type
        };
        // Pas de doublon
        if other_type != primary_type {
            Some(other_type)
        } else {
            // Tente avec les types secondaires des parents
            parent_a.secondary_type.or(parent_b.secondary_type)
        }
    } else {
        None
    };

    // --- Stats : moyenne des parents avec variance ---
    let base_stats = blend_stats(&parent_a.base_stats, &parent_b.base_stats, &mut rng);

    // --- Traits : héritage + mutation ---
    let (traits, mutation_occurred) = inherit_traits(parent_a, parent_b, &mut rng);

    let max_hp = base_stats.hp;
    let generation = parent_a.generation.max(parent_b.generation) + 1;

    let child = Monster {
        id: Uuid::new_v4(),
        name: child_name,
        primary_type,
        secondary_type,
        base_stats,
        level: 1,
        xp: 0,
        current_hp: max_hp,
        traits: traits.clone(),
        born_at: Utc::now(),
        died_at: None,
        parent_a: Some(parent_a.id),
        parent_b: Some(parent_b.id),
        wins: 0,
        losses: 0,
        generation,
        last_fed: Some(Utc::now()),
        meals_today: 0,
        meals_window_start: None,
    };

    let mut desc = format!(
        "{} est né de {} et {} ! (Gén. {})",
        child.name, parent_a.name, parent_b.name, child.generation
    );
    if mutation_occurred {
        desc.push_str(" 🧬 Une mutation génétique s'est produite !");
    }

    Ok(BreedingResult {
        child,
        mutation_occurred,
        description: desc,
    })
}

/// Mélange les stats de deux parents avec une variance aléatoire.
fn blend_stats(a: &Stats, b: &Stats, rng: &mut impl Rng) -> Stats {
    let mut blend = |va: u32, vb: u32| -> u32 {
        let avg = (va + vb) / 2;
        let variance = (avg as f64 * 0.15) as u32;
        let min = avg.saturating_sub(variance).max(1);
        let max = avg + variance;
        rng.gen_range(min..=max)
    };

    Stats {
        hp: blend(a.hp, b.hp),
        attack: blend(a.attack, b.attack),
        defense: blend(a.defense, b.defense),
        speed: blend(a.speed, b.speed),
        special_attack: blend(a.special_attack, b.special_attack),
        special_defense: blend(a.special_defense, b.special_defense),
    }
}

/// Hérite les traits des parents avec possibilité de mutation.
fn inherit_traits(
    parent_a: &Monster,
    parent_b: &Monster,
    rng: &mut impl Rng,
) -> (Vec<Trait>, bool) {
    let mut traits = Vec::new();
    let mut mutation = false;

    // Chaque trait de chaque parent a 50% de chance d'être transmis
    let all_parent_traits: Vec<Trait> = parent_a
        .traits
        .iter()
        .chain(parent_b.traits.iter())
        .cloned()
        .collect();

    for t in &all_parent_traits {
        if rng.gen_bool(0.5) && !traits.contains(t) {
            traits.push(t.clone());
        }
    }

    // Chance de mutation : gagner un nouveau trait aléatoire
    if rng.gen_bool(MUTATION_CHANCE) {
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

        let new_trait = all_traits[rng.gen_range(0..all_traits.len())].clone();
        if !traits.contains(&new_trait) {
            traits.push(new_trait);
            mutation = true;
        }
    }

    // Max 3 traits par monstre
    traits.truncate(3);

    (traits, mutation)
}

/// Génère des stats de base aléatoires pour un starter selon son type.
pub fn generate_starter_stats(element: ElementType) -> Stats {
    let mut rng = rand::thread_rng();

    // Chaque type a une spécialité
    let (base_hp, base_atk, base_def, base_spd, base_spatk, base_spdef) = match element {
        ElementType::Normal => (45, 45, 45, 45, 45, 45),
        ElementType::Fire => (40, 55, 35, 50, 55, 35),
        ElementType::Water => (50, 40, 50, 40, 45, 50),
        ElementType::Plant => (55, 40, 45, 35, 50, 50),
        ElementType::Electric => (35, 40, 30, 60, 55, 35),
        ElementType::Earth => (55, 50, 55, 25, 30, 45),
        ElementType::Wind => (40, 35, 30, 60, 50, 40),
        ElementType::Shadow => (40, 55, 35, 50, 55, 30),
        ElementType::Light => (45, 35, 40, 45, 50, 55),
    };

    let mut vary = |base: u32| -> u32 {
        let v = (base as f64 * 0.1) as u32;
        rng.gen_range(base.saturating_sub(v)..=base + v)
    };

    Stats::new(
        vary(base_hp),
        vary(base_atk),
        vary(base_def),
        vary(base_spd),
        vary(base_spatk),
        vary(base_spdef),
    )
}

/// Génère un monstre adverse pour l'entraînement.
///
/// - **Docile** (`wild = false`) : le bot est de niveau strictement inférieur
///   au joueur (1 à 3 niveaux en dessous, minimum 1). Cas spécial : si le joueur
///   est niveau 1, le bot est aussi niveau 1 (impossible de descendre plus bas).
/// - **Sauvage** (`wild = true`) : le bot est dans une fourchette de ±5 niveaux
///   autour du joueur (minimum 1, maximum 100).
pub fn generate_training_opponent(
    player_level: u32,
    opponent_type: ElementType,
    wild: bool,
) -> Monster {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let bot_level = if wild {
        // Sauvage : ±5 niveaux
        let min_level = player_level.saturating_sub(5).max(1);
        let max_level = (player_level + 5).min(100);
        rng.gen_range(min_level..=max_level)
    } else {
        // Docile : toujours strictement inférieur (1-3 niveaux en dessous)
        let sub = rng.gen_range(1..=3u32);
        player_level.saturating_sub(sub).max(1)
    };

    let bot_stats = generate_starter_stats(opponent_type);
    let mut bot = Monster::new_starter(format!("Bot {}", opponent_type), opponent_type, bot_stats);

    // Monter au niveau cible avec la bonne quantité d'XP
    if bot_level > 1 {
        let total_xp: u32 = (1..bot_level).map(|l| l * l * 10).sum();
        bot.gain_xp(total_xp);
    }

    bot
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breed_produces_child() {
        let mut parent_a = Monster::new_starter(
            "Flamby".to_string(),
            ElementType::Fire,
            generate_starter_stats(ElementType::Fire),
        );
        parent_a.traits.push(Trait::CriticalStrike);

        let mut parent_b = Monster::new_starter(
            "Aquara".to_string(),
            ElementType::Water,
            generate_starter_stats(ElementType::Water),
        );
        parent_b.traits.push(Trait::Regeneration);

        let result = breed(&parent_a, &parent_b, "Bébé".to_string());
        assert!(result.is_ok());

        let baby = result.unwrap().child;
        assert_eq!(baby.level, 1);
        assert_eq!(baby.generation, 1);
        assert!(baby.parent_a.is_some());
        assert!(baby.parent_b.is_some());
    }

    #[test]
    fn test_cannot_breed_dead_monster() {
        let mut parent_a = Monster::new_starter(
            "Mort".to_string(),
            ElementType::Shadow,
            generate_starter_stats(ElementType::Shadow),
        );
        parent_a.died_at = Some(Utc::now());

        let parent_b = Monster::new_starter(
            "Vivant".to_string(),
            ElementType::Light,
            generate_starter_stats(ElementType::Light),
        );

        let result = breed(&parent_a, &parent_b, "Impossible".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_training_docile_level_below_player() {
        // En mode docile, le bot doit être <= au joueur (strictement inférieur sauf niveau 1)
        for _ in 0..50 {
            let bot = generate_training_opponent(10, ElementType::Fire, false);
            assert!(bot.level < 10, "docile bot level {} should be < 10", bot.level);
            assert!(bot.level >= 1, "bot level should be >= 1");
        }
    }

    #[test]
    fn test_training_docile_level_1_edge_case() {
        // Cas spécial : joueur niveau 1, le bot ne peut pas descendre en dessous de 1
        for _ in 0..20 {
            let bot = generate_training_opponent(1, ElementType::Water, false);
            assert_eq!(bot.level, 1, "docile bot at player_level=1 should be 1");
        }
    }

    #[test]
    fn test_training_wild_level_range() {
        // En mode sauvage, le bot doit être dans ±5 niveaux du joueur
        for _ in 0..50 {
            let bot = generate_training_opponent(50, ElementType::Plant, true);
            assert!(
                bot.level >= 45 && bot.level <= 55,
                "wild bot level {} should be in 45..=55",
                bot.level
            );
        }
    }

    #[test]
    fn test_training_wild_level_clamp_low() {
        // Joueur niveau 1 en sauvage : bot entre 1 et 6
        for _ in 0..50 {
            let bot = generate_training_opponent(1, ElementType::Electric, true);
            assert!(bot.level >= 1 && bot.level <= 6, "bot level {} should be in 1..=6", bot.level);
        }
    }

    #[test]
    fn test_training_wild_level_clamp_high() {
        // Joueur niveau 100 en sauvage : bot entre 95 et 100
        for _ in 0..50 {
            let bot = generate_training_opponent(100, ElementType::Earth, true);
            assert!(
                bot.level >= 95 && bot.level <= 100,
                "bot level {} should be in 95..=100",
                bot.level
            );
        }
    }

    #[test]
    fn test_training_docile_level_2() {
        // Joueur niveau 2 : bot doit être 1
        for _ in 0..20 {
            let bot = generate_training_opponent(2, ElementType::Shadow, false);
            assert_eq!(bot.level, 1, "docile bot at player_level=2 should be 1");
        }
    }
}
