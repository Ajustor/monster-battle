//! Validation des monstres poussés par les clients.
//!
//! Le serveur fait autorité : il ne peut pas rejouer toute l'histoire d'un
//! monstre, mais il refuse les états manifestement impossibles. L'objectif est
//! d'éliminer l'édition grossière de sauvegarde (niveau 9999, stats à 60000)
//! avant qu'un monstre n'entre dans l'arène, pas de prouver l'authenticité de
//! chaque point d'XP.

use monster_battle_core::Monster;
use monster_battle_core::types::Trait;

/// Niveau maximum atteignable.
const MAX_LEVEL: u32 = 100;

/// Plafond d'une statistique de base, marge de reproduction comprise.
const MAX_BASE_STAT: u32 = 255;

/// Plafond de bonheur et de lien, tels que définis par le cœur du jeu.
const MAX_HAPPINESS: u32 = 100;
const MAX_BOND: u32 = 100;

/// Nombre de traits distincts existants : un monstre ne peut pas en avoir plus.
const TRAIT_COUNT: usize = 9;

/// Tolérance d'horloge acceptée sur les dates envoyées par un client.
const CLOCK_SKEW_MINUTES: i64 = 5;

/// Vérifie qu'un monstre est dans un état plausible.
///
/// Retourne la liste des anomalies ; vide = le monstre est acceptable.
pub fn check(monster: &Monster) -> Vec<String> {
    let mut problems = Vec::new();

    if monster.name.trim().is_empty() {
        problems.push("le nom est vide".to_string());
    }
    if monster.name.chars().count() > 32 {
        problems.push("le nom dépasse 32 caractères".to_string());
    }

    if monster.level == 0 || monster.level > MAX_LEVEL {
        problems.push(format!(
            "niveau {} hors des bornes 1–{}",
            monster.level, MAX_LEVEL
        ));
    }

    // L'XP courant est un reliquat vers le niveau suivant : au-delà, le level
    // up aurait dû se produire.
    if monster.level <= MAX_LEVEL && monster.xp >= monster.xp_to_next_level().max(1) {
        problems.push(format!(
            "{} XP pour un palier de {} : le passage de niveau n'a pas été appliqué",
            monster.xp,
            monster.xp_to_next_level()
        ));
    }

    let stats = [
        ("PV", monster.base_stats.hp),
        ("attaque", monster.base_stats.attack),
        ("défense", monster.base_stats.defense),
        ("vitesse", monster.base_stats.speed),
        ("attaque spéciale", monster.base_stats.special_attack),
        ("défense spéciale", monster.base_stats.special_defense),
    ];
    for (label, value) in stats {
        if value == 0 || value > MAX_BASE_STAT {
            problems.push(format!(
                "statistique {} = {}, hors des bornes 1–{}",
                label, value, MAX_BASE_STAT
            ));
        }
    }

    if monster.current_hp > monster.max_hp() {
        problems.push(format!(
            "{} PV courants pour {} PV max",
            monster.current_hp,
            monster.max_hp()
        ));
    }

    // Un monstre vivant à 0 PV est incohérent : le combat aurait dû le tuer.
    if monster.is_alive() && monster.current_hp == 0 {
        problems.push("vivant avec 0 PV".to_string());
    }

    if monster.happiness > MAX_HAPPINESS {
        problems.push(format!("bonheur {} > {}", monster.happiness, MAX_HAPPINESS));
    }
    if monster.bond > MAX_BOND {
        problems.push(format!("lien {} > {}", monster.bond, MAX_BOND));
    }

    if monster.traits.len() > TRAIT_COUNT {
        problems.push(format!(
            "{} traits pour {} existants",
            monster.traits.len(),
            TRAIT_COUNT
        ));
    }
    let mut seen: Vec<&Trait> = Vec::new();
    for t in &monster.traits {
        if seen.contains(&t) {
            problems.push(format!("trait {} en double", t));
        } else {
            seen.push(t);
        }
    }

    let now = chrono::Utc::now();
    let skew = chrono::Duration::minutes(CLOCK_SKEW_MINUTES);

    if monster.born_at > now + skew {
        problems.push("date de naissance dans le futur".to_string());
    }
    if let Some(died_at) = monster.died_at {
        if died_at > now + skew {
            problems.push("date de mort dans le futur".to_string());
        }
        if died_at < monster.born_at {
            problems.push("mort avant sa naissance".to_string());
        }
    }

    if monster.parent_a.is_some() != monster.parent_b.is_some() {
        problems.push("un seul parent renseigné".to_string());
    }
    if monster.generation > 0 && monster.parent_a.is_none() {
        problems.push(format!("génération {} sans parents", monster.generation));
    }
    if monster.generation == 0 && monster.parent_a.is_some() {
        problems.push("génération 0 avec des parents".to_string());
    }

    problems
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use monster_battle_core::types::{ElementType, Stats};

    fn starter() -> Monster {
        Monster::new_starter(
            "Flamby".to_string(),
            ElementType::Fire,
            Stats::new(50, 45, 40, 35, 50, 40),
        )
    }

    #[test]
    fn un_starter_frais_est_valide() {
        assert!(check(&starter()).is_empty(), "{:?}", check(&starter()));
    }

    #[test]
    fn le_niveau_hors_bornes_est_refuse() {
        let mut m = starter();
        m.level = 9999;
        assert!(check(&m).iter().any(|p| p.contains("niveau")));

        let mut m = starter();
        m.level = 0;
        assert!(check(&m).iter().any(|p| p.contains("niveau")));
    }

    #[test]
    fn les_stats_gonflees_sont_refusees() {
        let mut m = starter();
        m.base_stats.attack = 60000;
        assert!(check(&m).iter().any(|p| p.contains("attaque")));
    }

    #[test]
    fn plus_de_pv_que_le_maximum_est_refuse() {
        let mut m = starter();
        m.current_hp = m.max_hp() + 1;
        assert!(check(&m).iter().any(|p| p.contains("PV courants")));
    }

    #[test]
    fn l_xp_non_consomme_est_refuse() {
        let mut m = starter();
        m.xp = m.xp_to_next_level() + 1;
        assert!(check(&m).iter().any(|p| p.contains("XP")));

        // Juste en dessous du palier : parfaitement normal.
        let mut m = starter();
        m.xp = m.xp_to_next_level() - 1;
        assert!(check(&m).is_empty());
    }

    #[test]
    fn les_traits_en_double_sont_refuses() {
        let mut m = starter();
        m.traits = vec![Trait::Berserk, Trait::Berserk];
        assert!(check(&m).iter().any(|p| p.contains("double")));
    }

    #[test]
    fn une_naissance_dans_le_futur_est_refusee() {
        let mut m = starter();
        m.born_at = Utc::now() + Duration::days(1);
        assert!(check(&m).iter().any(|p| p.contains("futur")));

        // Une petite dérive d'horloge reste tolérée.
        let mut m = starter();
        m.born_at = Utc::now() + Duration::minutes(1);
        assert!(check(&m).is_empty());
    }

    #[test]
    fn une_lignee_incoherente_est_refusee() {
        let mut m = starter();
        m.generation = 3;
        assert!(check(&m).iter().any(|p| p.contains("génération")));

        let mut m = starter();
        m.parent_a = Some(uuid::Uuid::new_v4());
        assert!(!check(&m).is_empty());
    }

    #[test]
    fn un_monstre_mort_au_combat_reste_valide() {
        let mut m = starter();
        m.current_hp = 0;
        m.died_at = Some(Utc::now());
        assert!(check(&m).is_empty(), "{:?}", check(&m));
    }
}
