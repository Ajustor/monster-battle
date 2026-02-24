use rand::Rng;

use crate::monster::Monster;
use crate::types::Trait;

/// Résultat d'un combat entre deux monstres.
#[derive(Debug)]
pub struct CombatResult {
    pub winner_id: uuid::Uuid,
    pub loser_id: uuid::Uuid,
    pub log: Vec<CombatEvent>,
    /// Le perdant est-il mort ?
    pub loser_died: bool,
}

/// Un événement dans le déroulement d'un combat.
#[derive(Debug, Clone)]
pub enum CombatEvent {
    TurnStart {
        turn: u32,
    },
    Attack {
        attacker: String,
        defender: String,
        damage: u32,
        is_critical: bool,
    },
    Regeneration {
        monster: String,
        amount: u32,
    },
    Evasion {
        monster: String,
    },
    ThornsDamage {
        monster: String,
        amount: u32,
    },
    BerserkActivated {
        monster: String,
    },
    TenacitySaved {
        monster: String,
    },
    MonsterFainted {
        monster: String,
    },
}

impl CombatEvent {
    /// Retourne une description textuelle de l'événement.
    pub fn describe(&self) -> String {
        match self {
            CombatEvent::TurnStart { turn } => format!("--- Tour {} ---", turn),
            CombatEvent::Attack {
                attacker,
                defender,
                damage,
                is_critical,
            } => {
                let crit = if *is_critical { " 💥 CRITIQUE !" } else { "" };
                format!(
                    "{} attaque {} pour {} dégâts !{}",
                    attacker, defender, damage, crit
                )
            }
            CombatEvent::Regeneration { monster, amount } => {
                format!("🩹 {} régénère {} PV", monster, amount)
            }
            CombatEvent::Evasion { monster } => format!("💨 {} esquive l'attaque !", monster),
            CombatEvent::ThornsDamage { monster, amount } => {
                format!("🌵 {} subit {} dégâts d'épines !", monster, amount)
            }
            CombatEvent::BerserkActivated { monster } => {
                format!("😡 {} entre en mode Berserk !", monster)
            }
            CombatEvent::TenacitySaved { monster } => {
                format!("💪 {} tient bon avec 1 PV !", monster)
            }
            CombatEvent::MonsterFainted { monster } => format!("💀 {} est K.O. !", monster),
        }
    }
}

/// Calcule les dégâts bruts d'une attaque.
fn calculate_damage(attacker: &Monster, defender: &Monster, rng: &mut impl Rng) -> (u32, bool) {
    let mut atk = attacker.effective_attack() as f64;

    // Berserk : +50% attaque si PV < 25%
    let is_berserk =
        attacker.traits.contains(&Trait::Berserk) && attacker.current_hp < attacker.max_hp() / 4;
    if is_berserk {
        atk *= 1.5;
    }

    let def = defender.effective_defense() as f64;

    // Formule de dégâts simple
    let base_damage = ((atk * 2.0) / (def + atk) * 20.0).max(1.0);

    // Efficacité de type
    let type_mult = attacker
        .primary_type
        .effectiveness_against(&defender.primary_type);
    let type_mult = match &defender.secondary_type {
        Some(sec) => type_mult * attacker.primary_type.effectiveness_against(sec),
        None => type_mult,
    };

    // Coup critique
    let crit_chance = if attacker.traits.contains(&Trait::CriticalStrike) {
        0.20
    } else {
        0.08
    };
    let is_critical = rng.gen_bool(crit_chance);
    let crit_mult = if is_critical { 1.5 } else { 1.0 };

    // Variance aléatoire (±10%)
    let variance = rng.gen_range(0.9..=1.1);

    let total = (base_damage * type_mult * crit_mult * variance) as u32;
    (total.max(1), is_critical)
}

/// Lance un combat tour par tour entre deux monstres.
/// Les monstres sont modifiés en place (PV, XP, wins/losses, mort éventuelle).
pub fn fight(monster_a: &mut Monster, monster_b: &mut Monster) -> Result<CombatResult, String> {
    if monster_a.is_dead() {
        return Err(format!(
            "{} est mort et ne peut pas combattre.",
            monster_a.name
        ));
    }
    if monster_b.is_dead() {
        return Err(format!(
            "{} est mort et ne peut pas combattre.",
            monster_b.name
        ));
    }

    let mut rng = rand::thread_rng();
    let mut log = Vec::new();
    let max_turns = 100;

    for turn in 1..=max_turns {
        log.push(CombatEvent::TurnStart { turn });

        // Détermine l'ordre d'attaque selon la vitesse
        let a_first = if monster_a.effective_speed() == monster_b.effective_speed() {
            rng.gen_bool(0.5)
        } else {
            monster_a.effective_speed() > monster_b.effective_speed()
        };

        let (first_idx, second_idx) = if a_first { (0, 1) } else { (1, 0) };

        // Tour du premier attaquant
        if let Some(event) = execute_turn(
            first_idx, second_idx, monster_a, monster_b, &mut rng, &mut log,
        ) {
            log.push(event);
            break;
        }

        // Tour du second attaquant (s'il est encore vivant)
        let defender_alive = if second_idx == 0 {
            monster_a.is_alive()
        } else {
            monster_b.is_alive()
        };
        if defender_alive {
            if let Some(event) = execute_turn(
                second_idx, first_idx, monster_a, monster_b, &mut rng, &mut log,
            ) {
                log.push(event);
                break;
            }
        }

        // Régénération en fin de tour
        let name_a = monster_a.name.clone();
        let name_b = monster_b.name.clone();
        for (monster, name_for_log) in [(&mut *monster_a, name_a), (&mut *monster_b, name_b)] {
            if monster.is_alive() && monster.traits.contains(&Trait::Regeneration) {
                let regen = (monster.max_hp() as f64 * 0.05) as u32;
                if regen > 0 {
                    monster.heal(regen);
                    log.push(CombatEvent::Regeneration {
                        monster: name_for_log,
                        amount: regen,
                    });
                }
            }
        }
    }

    // Déterminer le gagnant
    let (winner, loser, loser_died) = if monster_a.is_dead() || monster_a.current_hp == 0 {
        if monster_a.is_alive() {
            monster_a.died_at = Some(chrono::Utc::now());
        }
        (monster_b, monster_a, true)
    } else if monster_b.is_dead() || monster_b.current_hp == 0 {
        if monster_b.is_alive() {
            monster_b.died_at = Some(chrono::Utc::now());
        }
        (monster_a, monster_b, true)
    } else {
        // Timeout : celui avec le plus de PV% gagne
        let a_pct = monster_a.current_hp as f64 / monster_a.max_hp() as f64;
        let b_pct = monster_b.current_hp as f64 / monster_b.max_hp() as f64;
        if a_pct >= b_pct {
            (monster_a, monster_b, false)
        } else {
            (monster_b, monster_a, false)
        }
    };

    // XP et compteurs
    winner.wins += 1;
    loser.losses += 1;
    let xp_gain = 50 + (loser.level * 5);
    winner.gain_xp(xp_gain);

    Ok(CombatResult {
        winner_id: winner.id,
        loser_id: loser.id,
        log,
        loser_died,
    })
}

/// Exécute le tour d'un attaquant. Retourne Some(event) si le défenseur meurt.
fn execute_turn(
    attacker_idx: usize,
    _defender_idx: usize,
    monster_a: &mut Monster,
    monster_b: &mut Monster,
    rng: &mut impl Rng,
    log: &mut Vec<CombatEvent>,
) -> Option<CombatEvent> {
    // Récupérer les infos en lecture seule d'abord
    let (
        attacker_name,
        defender_name,
        defender_has_evasion,
        attacker_has_berserk,
        attacker_low_hp,
        damage,
        is_critical,
        defender_has_thorns,
    ) = {
        let (attacker, defender) = if attacker_idx == 0 {
            (&*monster_a as &Monster, &*monster_b as &Monster)
        } else {
            (&*monster_b as &Monster, &*monster_a as &Monster)
        };

        let a_name = attacker.name.clone();
        let d_name = defender.name.clone();
        let d_evasion = defender.traits.contains(&Trait::Evasion);
        let a_berserk = attacker.traits.contains(&Trait::Berserk);
        let a_low_hp = attacker.current_hp < attacker.max_hp() / 4;
        let (dmg, crit) = calculate_damage(attacker, defender, rng);
        let d_thorns = defender.traits.contains(&Trait::Thorns);

        (
            a_name, d_name, d_evasion, a_berserk, a_low_hp, dmg, crit, d_thorns,
        )
    };

    // Évasion
    if defender_has_evasion && rng.gen_bool(0.12) {
        log.push(CombatEvent::Evasion {
            monster: defender_name,
        });
        return None;
    }

    // Berserk notification
    if attacker_has_berserk && attacker_low_hp {
        log.push(CombatEvent::BerserkActivated {
            monster: attacker_name.clone(),
        });
    }

    log.push(CombatEvent::Attack {
        attacker: attacker_name.clone(),
        defender: defender_name.clone(),
        damage,
        is_critical,
    });

    // Applique les dégâts au défenseur
    let defender_mut = if attacker_idx == 0 {
        &mut *monster_b
    } else {
        &mut *monster_a
    };
    let actual = defender_mut.take_damage(damage);

    // Tenacity check
    if actual < damage && defender_mut.current_hp == 1 {
        log.push(CombatEvent::TenacitySaved {
            monster: defender_name.clone(),
        });
    }

    let defender_alive = defender_mut.is_alive();
    let defender_dead = defender_mut.is_dead() || defender_mut.current_hp == 0;

    // Thorns
    if defender_has_thorns && defender_alive {
        let thorn_dmg = (damage as f64 * 0.15) as u32;
        if thorn_dmg > 0 {
            let attacker_mut = if attacker_idx == 0 {
                &mut *monster_a
            } else {
                &mut *monster_b
            };
            attacker_mut.take_damage(thorn_dmg);
            log.push(CombatEvent::ThornsDamage {
                monster: attacker_name,
                amount: thorn_dmg,
            });
        }
    }

    if defender_dead {
        return Some(CombatEvent::MonsterFainted {
            monster: defender_name,
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genetics::generate_starter_stats;
    use crate::types::ElementType;

    #[test]
    fn test_combat_produces_winner() {
        let mut a = Monster::new_starter(
            "Flamby".to_string(),
            ElementType::Fire,
            generate_starter_stats(ElementType::Fire),
        );
        let mut b = Monster::new_starter(
            "Aquara".to_string(),
            ElementType::Water,
            generate_starter_stats(ElementType::Water),
        );

        let result = fight(&mut a, &mut b);
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_ne!(result.winner_id, result.loser_id);
        assert!(!result.log.is_empty());
    }
}
