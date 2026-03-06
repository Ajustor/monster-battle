//! Pierre-Papier-Ciseaux élémentaire — mini-jeu stratégique.
//!
//! Le joueur choisit un élément parmi 3 proposés (tirés des types du jeu).
//! L'IA choisit aléatoirement. Le vainqueur est déterminé par le triangle
//! des efficacités élémentaires (Feu > Plante > Eau > Feu, etc.).
//!
//! Best of 3/5/7 selon la difficulté.
//! Le type du monstre du joueur offre un bonus : si le type choisi correspond
//! au type du monstre, en cas de match nul le joueur gagne.
//!
//! Récompense en ATK / ATK.S selon la difficulté et le score.

use rand::Rng;

use crate::types::ElementType;

use super::{MinigameResult, StatReward};

// ── Types ───────────────────────────────────────────────────────

/// Difficulté de la partie (nombre de manches).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    /// Best of 3.
    Easy,
    /// Best of 5.
    Medium,
    /// Best of 7.
    Hard,
}

impl Difficulty {
    pub fn label(self) -> &'static str {
        match self {
            Difficulty::Easy => "Facile",
            Difficulty::Medium => "Moyen",
            Difficulty::Hard => "Difficile",
        }
    }

    pub fn all() -> &'static [Difficulty] {
        &[Difficulty::Easy, Difficulty::Medium, Difficulty::Hard]
    }

    /// Nombre total de manches (best of N).
    pub fn total_rounds(self) -> usize {
        match self {
            Difficulty::Easy => 3,
            Difficulty::Medium => 5,
            Difficulty::Hard => 7,
        }
    }

    /// Nombre de victoires nécessaires.
    pub fn wins_needed(self) -> usize {
        self.total_rounds() / 2 + 1
    }
}

/// Résultat d'une manche individuelle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundOutcome {
    PlayerWin,
    AiWin,
    Draw,
}

/// Triplet d'éléments utilisé pour une partie (triangle d'efficacité).
/// L'élément 0 bat l'élément 1, l'élément 1 bat l'élément 2, l'élément 2 bat l'élément 0.
#[derive(Debug, Clone)]
pub struct ElementTriple {
    pub elements: [ElementType; 3],
}

impl ElementTriple {
    /// Détermine le résultat : `a` contre `b` dans ce triangle.
    pub fn outcome(&self, a: ElementType, b: ElementType) -> RoundOutcome {
        if a == b {
            return RoundOutcome::Draw;
        }

        let idx_a = self.elements.iter().position(|&e| e == a);
        let idx_b = self.elements.iter().position(|&e| e == b);

        match (idx_a, idx_b) {
            (Some(ia), Some(ib)) => {
                // Le triangle : 0 bat 1, 1 bat 2, 2 bat 0
                if (ia + 1) % 3 == ib {
                    RoundOutcome::PlayerWin
                } else {
                    RoundOutcome::AiWin
                }
            }
            _ => RoundOutcome::Draw,
        }
    }
}

/// Triplets prédéfinis d'éléments.
const TRIPLES: &[[ElementType; 3]] = &[
    [ElementType::Fire, ElementType::Plant, ElementType::Water],
    [
        ElementType::Electric,
        ElementType::Water,
        ElementType::Earth,
    ],
    [ElementType::Wind, ElementType::Earth, ElementType::Plant],
    [ElementType::Shadow, ElementType::Light, ElementType::Fire],
    [ElementType::Light, ElementType::Shadow, ElementType::Water],
];

/// Choisit un triplet aléatoire.
fn random_triple() -> ElementTriple {
    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..TRIPLES.len());
    ElementTriple {
        elements: TRIPLES[idx],
    }
}

/// Historique d'une manche.
#[derive(Debug, Clone)]
pub struct RoundHistory {
    pub player_choice: ElementType,
    pub ai_choice: ElementType,
    pub outcome: RoundOutcome,
}

/// État complet d'une partie de PPC élémentaire.
#[derive(Debug, Clone)]
pub struct RpsGame {
    /// Triplet d'éléments pour cette partie.
    pub triple: ElementTriple,
    /// Historique des manches jouées.
    pub history: Vec<RoundHistory>,
    /// Score du joueur.
    pub player_wins: usize,
    /// Score de l'IA.
    pub ai_wins: usize,
    /// Nombre de manches jouées.
    pub rounds_played: usize,
    /// Difficulté.
    pub difficulty: Difficulty,
    /// Type élémentaire du monstre du joueur (bonus en cas de match nul).
    pub monster_type: ElementType,
    /// Index de sélection du joueur (0–2) dans le triple.
    pub cursor: usize,
    /// Résultat final (None si en cours).
    pub result: Option<MinigameResult>,
    /// Dernier résultat de manche (pour affichage).
    pub last_round: Option<RoundHistory>,
    /// Indique si on attend la confirmation du joueur après un round.
    pub waiting_confirm: bool,
}

impl RpsGame {
    /// Crée une nouvelle partie.
    pub fn new(difficulty: Difficulty, monster_type: ElementType) -> Self {
        Self {
            triple: random_triple(),
            history: Vec::new(),
            player_wins: 0,
            ai_wins: 0,
            rounds_played: 0,
            difficulty,
            monster_type,
            cursor: 0,
            result: None,
            last_round: None,
            waiting_confirm: false,
        }
    }

    /// La partie est-elle terminée ?
    pub fn is_over(&self) -> bool {
        self.result.is_some()
    }

    /// Les 3 éléments disponibles pour le choix.
    pub fn choices(&self) -> &[ElementType; 3] {
        &self.triple.elements
    }

    /// Le joueur soumet son choix (index 0–2).
    pub fn play(&mut self, choice_index: usize) -> Option<RoundOutcome> {
        if self.is_over() || self.waiting_confirm {
            return None;
        }

        let player_choice = self.triple.elements[choice_index % 3];

        // L'IA choisit aléatoirement
        let mut rng = rand::thread_rng();
        let ai_index = rng.gen_range(0..3);
        let ai_choice = self.triple.elements[ai_index];

        let mut outcome = self.triple.outcome(player_choice, ai_choice);

        // Bonus du type monstre : en cas de match nul, si le joueur a choisi
        // le type de son monstre, il gagne la manche
        if outcome == RoundOutcome::Draw && player_choice == self.monster_type {
            outcome = RoundOutcome::PlayerWin;
        }

        match outcome {
            RoundOutcome::PlayerWin => self.player_wins += 1,
            RoundOutcome::AiWin => self.ai_wins += 1,
            RoundOutcome::Draw => {} // Les nuls ne comptent pas
        }

        self.rounds_played += 1;

        let round = RoundHistory {
            player_choice,
            ai_choice,
            outcome,
        };
        self.history.push(round.clone());
        self.last_round = Some(round);
        self.waiting_confirm = true;

        // Vérifier si la partie est terminée
        let wins_needed = self.difficulty.wins_needed();
        if self.player_wins >= wins_needed {
            self.result = Some(MinigameResult::Win);
        } else if self.ai_wins >= wins_needed {
            self.result = Some(MinigameResult::Loss);
        } else if self.rounds_played >= self.difficulty.total_rounds() {
            // Toutes les manches jouées — déterminer le résultat
            if self.player_wins > self.ai_wins {
                self.result = Some(MinigameResult::Win);
            } else if self.ai_wins > self.player_wins {
                self.result = Some(MinigameResult::Loss);
            } else {
                self.result = Some(MinigameResult::Draw);
            }
        }

        Some(outcome)
    }

    /// Confirme la lecture du résultat du round et passe au suivant.
    pub fn confirm(&mut self) {
        self.waiting_confirm = false;
    }

    // ── Récompenses ─────────────────────────────────────────────

    pub fn reward(&self) -> StatReward {
        let Some(result) = self.result else {
            return StatReward::none();
        };

        match result {
            MinigameResult::Win => match self.difficulty {
                Difficulty::Easy => StatReward {
                    hp: 0,
                    attack: 1,
                    defense: 0,
                    speed: 0,
                    special_attack: 1,
                    special_defense: 0,
                    xp: 20,
                },
                Difficulty::Medium => StatReward {
                    hp: 1,
                    attack: 1,
                    defense: 1,
                    speed: 0,
                    special_attack: 1,
                    special_defense: 1,
                    xp: 35,
                },
                Difficulty::Hard => StatReward {
                    hp: 1,
                    attack: 2,
                    defense: 1,
                    speed: 1,
                    special_attack: 2,
                    special_defense: 0,
                    xp: 55,
                },
            },
            MinigameResult::Draw => StatReward {
                hp: 0,
                attack: 0,
                defense: 0,
                speed: 0,
                special_attack: 1,
                special_defense: 0,
                xp: 10,
            },
            MinigameResult::Loss => StatReward::none(),
        }
    }

    /// Label du résultat.
    pub fn result_label(&self) -> &'static str {
        match self.result {
            Some(MinigameResult::Win) => "Victoire elementaire !",
            Some(MinigameResult::Draw) => "Match nul.",
            Some(MinigameResult::Loss) => "Defaite...",
            None => "En cours...",
        }
    }

    /// Score textuel.
    pub fn score_display(&self) -> String {
        format!(
            "{} - {} (BO{})",
            self.player_wins,
            self.ai_wins,
            self.difficulty.total_rounds()
        )
    }
}

// ── Tests ───────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_game_not_over() {
        let game = RpsGame::new(Difficulty::Easy, ElementType::Fire);
        assert!(!game.is_over());
        assert_eq!(game.player_wins, 0);
        assert_eq!(game.ai_wins, 0);
    }

    #[test]
    fn triple_outcome_correct() {
        let triple = ElementTriple {
            elements: [ElementType::Fire, ElementType::Plant, ElementType::Water],
        };
        // Fire beats Plant
        assert_eq!(
            triple.outcome(ElementType::Fire, ElementType::Plant),
            RoundOutcome::PlayerWin
        );
        // Plant beats Water
        assert_eq!(
            triple.outcome(ElementType::Plant, ElementType::Water),
            RoundOutcome::PlayerWin
        );
        // Water beats Fire
        assert_eq!(
            triple.outcome(ElementType::Water, ElementType::Fire),
            RoundOutcome::PlayerWin
        );
        // Reverse
        assert_eq!(
            triple.outcome(ElementType::Plant, ElementType::Fire),
            RoundOutcome::AiWin
        );
    }

    #[test]
    fn same_choice_is_draw() {
        let triple = ElementTriple {
            elements: [ElementType::Fire, ElementType::Plant, ElementType::Water],
        };
        assert_eq!(
            triple.outcome(ElementType::Fire, ElementType::Fire),
            RoundOutcome::Draw
        );
    }

    #[test]
    fn play_returns_outcome() {
        let mut game = RpsGame::new(Difficulty::Easy, ElementType::Fire);
        let outcome = game.play(0);
        assert!(outcome.is_some());
        assert_eq!(game.rounds_played, 1);
        assert!(game.waiting_confirm);
    }

    #[test]
    fn game_ends_after_enough_wins() {
        // Play many games to ensure at least some finish via wins_needed
        for _ in 0..20 {
            let mut game = RpsGame::new(Difficulty::Easy, ElementType::Fire);
            while !game.is_over() {
                game.play(0);
                game.confirm();
            }
            assert!(game.is_over());
        }
    }

    #[test]
    fn reward_for_win() {
        let mut game = RpsGame::new(Difficulty::Hard, ElementType::Water);
        game.result = Some(MinigameResult::Win);
        let r = game.reward();
        assert!(!r.is_empty());
        assert!(r.attack > 0 || r.special_attack > 0);
    }
}
