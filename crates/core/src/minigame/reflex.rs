//! Réflexe (Quick Time Event) — mini-jeu de rapidité.
//!
//! Des directions apparaissent (↑↓←→), le joueur doit appuyer sur la bonne
//! touche. Le score dépend du nombre de bonnes réponses sur N rounds.
//!
//! Récompense en VIT / ATK selon la difficulté et le score.

use rand::Rng;

use super::{MinigameResult, StatReward};

// ── Types ───────────────────────────────────────────────────────

/// Direction attendue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arrow {
    Up,
    Down,
    Left,
    Right,
}

impl Arrow {
    /// Symbole d'affichage.
    pub fn symbol(self) -> &'static str {
        match self {
            Arrow::Up => "↑",
            Arrow::Down => "↓",
            Arrow::Left => "←",
            Arrow::Right => "→",
        }
    }

    /// Nom de la direction.
    pub fn label(self) -> &'static str {
        match self {
            Arrow::Up => "Haut",
            Arrow::Down => "Bas",
            Arrow::Left => "Gauche",
            Arrow::Right => "Droite",
        }
    }

    /// Génère une direction aléatoire.
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..4) {
            0 => Arrow::Up,
            1 => Arrow::Down,
            2 => Arrow::Left,
            _ => Arrow::Right,
        }
    }
}

/// Difficulté du jeu de réflexe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    /// 8 rounds.
    Easy,
    /// 12 rounds.
    Medium,
    /// 16 rounds.
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

    /// Nombre de rounds.
    pub fn rounds(self) -> usize {
        match self {
            Difficulty::Easy => 8,
            Difficulty::Medium => 12,
            Difficulty::Hard => 16,
        }
    }
}

/// Résultat d'un round individuel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundResult {
    /// Bonne réponse.
    Correct,
    /// Mauvaise réponse.
    Wrong,
}

/// État complet d'une partie de Réflexe.
#[derive(Debug, Clone)]
pub struct ReflexGame {
    /// Séquence de flèches à reproduire.
    pub sequence: Vec<Arrow>,
    /// Résultats de chaque round.
    pub results: Vec<RoundResult>,
    /// Index du round en cours.
    pub current_round: usize,
    /// Nombre total de rounds.
    pub total_rounds: usize,
    /// Difficulté.
    pub difficulty: Difficulty,
    /// Résultat final (None si en cours).
    pub result: Option<MinigameResult>,
    /// Score (bonnes réponses).
    pub score: usize,
}

impl ReflexGame {
    /// Crée une nouvelle partie.
    pub fn new(difficulty: Difficulty) -> Self {
        let total = difficulty.rounds();
        let sequence: Vec<Arrow> = (0..total).map(|_| Arrow::random()).collect();

        Self {
            sequence,
            results: Vec::with_capacity(total),
            current_round: 0,
            total_rounds: total,
            difficulty,
            result: None,
            score: 0,
        }
    }

    /// La partie est-elle terminée ?
    pub fn is_over(&self) -> bool {
        self.result.is_some()
    }

    /// La flèche attendue pour le round en cours.
    pub fn current_arrow(&self) -> Option<Arrow> {
        self.sequence.get(self.current_round).copied()
    }

    /// Le joueur soumet une direction.
    /// Retourne `true` si la réponse est correcte.
    pub fn submit(&mut self, arrow: Arrow) -> bool {
        if self.is_over() {
            return false;
        }

        let expected = self.sequence[self.current_round];
        let correct = arrow == expected;

        if correct {
            self.results.push(RoundResult::Correct);
            self.score += 1;
        } else {
            self.results.push(RoundResult::Wrong);
        }

        self.current_round += 1;

        // Vérifier si tous les rounds sont joués
        if self.current_round >= self.total_rounds {
            self.result = Some(self.evaluate_result());
        }

        correct
    }

    /// Évalue le résultat final.
    fn evaluate_result(&self) -> MinigameResult {
        let ratio = self.score as f32 / self.total_rounds as f32;

        if ratio >= 0.8 {
            MinigameResult::Win
        } else if ratio >= 0.5 {
            MinigameResult::Draw
        } else {
            MinigameResult::Loss
        }
    }

    /// Calcule la récompense.
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
                    speed: 1,
                    special_attack: 0,
                    special_defense: 0,
                    xp: 20,
                },
                Difficulty::Medium => StatReward {
                    hp: 0,
                    attack: 1,
                    defense: 0,
                    speed: 2,
                    special_attack: 1,
                    special_defense: 0,
                    xp: 35,
                },
                Difficulty::Hard => StatReward {
                    hp: 1,
                    attack: 2,
                    defense: 0,
                    speed: 2,
                    special_attack: 1,
                    special_defense: 1,
                    xp: 55,
                },
            },
            MinigameResult::Draw => StatReward {
                hp: 0,
                attack: 0,
                defense: 0,
                speed: 1,
                special_attack: 0,
                special_defense: 0,
                xp: 10,
            },
            MinigameResult::Loss => StatReward::none(),
        }
    }

    /// Label du résultat.
    pub fn result_label(&self) -> &'static str {
        match self.result {
            Some(MinigameResult::Win) => "Excellents reflexes !",
            Some(MinigameResult::Draw) => "Pas mal.",
            Some(MinigameResult::Loss) => "Trop d'erreurs...",
            None => "En cours...",
        }
    }
}

// ── Tests ───────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_game_not_over() {
        let game = ReflexGame::new(Difficulty::Easy);
        assert!(!game.is_over());
        assert_eq!(game.current_round, 0);
        assert_eq!(game.total_rounds, 8);
    }

    #[test]
    fn correct_answer_increments_score() {
        let mut game = ReflexGame::new(Difficulty::Easy);
        let arrow = game.current_arrow().unwrap();
        assert!(game.submit(arrow));
        assert_eq!(game.score, 1);
        assert_eq!(game.current_round, 1);
    }

    #[test]
    fn wrong_answer_no_score() {
        let mut game = ReflexGame::new(Difficulty::Easy);
        let arrow = game.current_arrow().unwrap();
        // Submit wrong direction
        let wrong = match arrow {
            Arrow::Up => Arrow::Down,
            Arrow::Down => Arrow::Up,
            Arrow::Left => Arrow::Right,
            Arrow::Right => Arrow::Left,
        };
        assert!(!game.submit(wrong));
        assert_eq!(game.score, 0);
    }

    #[test]
    fn game_ends_after_all_rounds() {
        let mut game = ReflexGame::new(Difficulty::Easy);
        for _ in 0..game.total_rounds {
            let arrow = game.current_arrow().unwrap();
            game.submit(arrow);
        }
        assert!(game.is_over());
        assert_eq!(game.result, Some(MinigameResult::Win)); // All correct
    }

    #[test]
    fn all_wrong_gives_loss() {
        let mut game = ReflexGame::new(Difficulty::Easy);
        for _ in 0..game.total_rounds {
            let arrow = game.current_arrow().unwrap();
            let wrong = match arrow {
                Arrow::Up => Arrow::Down,
                Arrow::Down => Arrow::Up,
                Arrow::Left => Arrow::Right,
                Arrow::Right => Arrow::Left,
            };
            game.submit(wrong);
        }
        assert!(game.is_over());
        assert_eq!(game.result, Some(MinigameResult::Loss));
    }

    #[test]
    fn reward_for_win() {
        let mut game = ReflexGame::new(Difficulty::Medium);
        game.result = Some(MinigameResult::Win);
        let r = game.reward();
        assert!(!r.is_empty());
        assert!(r.speed > 0);
    }
}
