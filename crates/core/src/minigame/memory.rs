//! Memory (paires) — mini-jeu de mémorisation.
//!
//! Grille 4×4 de cartes face cachée. Le joueur retourne deux cartes à la fois.
//! Si elles forment une paire (même icône élémentaire), elles restent visibles.
//! Sinon, elles sont re-cachées après un court délai.
//!
//! Score = nombre de tentatives pour trouver toutes les paires.
//! Récompense en DEF / DEF.S selon la difficulté et le score.

use rand::seq::SliceRandom;

use super::{MinigameResult, StatReward};

// ── Types ───────────────────────────────────────────────────────

/// Difficulté du Memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    /// Grille 4×3 (6 paires) — facile.
    Easy,
    /// Grille 4×4 (8 paires) — moyen.
    Medium,
    /// Grille 4×5 (10 paires) — difficile.
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

    /// Nombre de colonnes de la grille.
    pub fn cols(self) -> usize {
        match self {
            Difficulty::Easy => 4,
            Difficulty::Medium => 4,
            Difficulty::Hard => 5,
        }
    }

    /// Nombre de lignes de la grille.
    pub fn rows(self) -> usize {
        match self {
            Difficulty::Easy => 3,
            Difficulty::Medium => 4,
            Difficulty::Hard => 4,
        }
    }

    /// Nombre total de cartes.
    pub fn card_count(self) -> usize {
        self.cols() * self.rows()
    }

    /// Nombre de paires.
    pub fn pair_count(self) -> usize {
        self.card_count() / 2
    }

    /// Nombre d'essais « parfait » (minimum théorique avec de la chance).
    pub fn perfect_attempts(self) -> usize {
        self.pair_count()
    }
}

/// État d'une carte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardState {
    /// Face cachée.
    Hidden,
    /// Retournée temporairement (sélection en cours).
    Revealed,
    /// Paire trouvée — reste visible.
    Matched,
}

/// Icônes pour les cartes (types élémentaires du jeu).
const ICONS: &[&str] = &["🔥", "💧", "🌿", "⚡", "🪨", "🌪️", "🌑", "✨", "⭐", "🐉"];

/// État complet d'une partie de Memory.
#[derive(Debug, Clone)]
pub struct MemoryGame {
    /// Valeur de chaque carte (index dans ICONS).
    pub cards: Vec<usize>,
    /// État de chaque carte.
    pub states: Vec<CardState>,
    /// Nombre de colonnes.
    pub cols: usize,
    /// Nombre de lignes.
    pub rows: usize,
    /// Curseur du joueur (index dans cards).
    pub cursor: usize,
    /// Première carte retournée (index) — None si aucune.
    pub first_pick: Option<usize>,
    /// Seconde carte retournée (index) — None si aucune.
    pub second_pick: Option<usize>,
    /// Nombre de tentatives (chaque paire retournée = 1 tentative).
    pub attempts: usize,
    /// Nombre de paires trouvées.
    pub pairs_found: usize,
    /// Nombre total de paires.
    pub total_pairs: usize,
    /// Difficulté.
    pub difficulty: Difficulty,
    /// Résultat (None si en cours).
    pub result: Option<MinigameResult>,
    /// Le joueur doit confirmer pour cacher les cartes non-matchées.
    pub needs_dismiss: bool,
}

impl MemoryGame {
    /// Crée une nouvelle partie.
    pub fn new(difficulty: Difficulty) -> Self {
        let cols = difficulty.cols();
        let rows = difficulty.rows();
        let total = cols * rows;
        let pairs = total / 2;

        // Construire les paires
        let mut cards: Vec<usize> = (0..pairs).flat_map(|i| [i, i]).collect();

        // Mélanger
        let mut rng = rand::thread_rng();
        cards.shuffle(&mut rng);

        let states = vec![CardState::Hidden; total];

        Self {
            cards,
            states,
            cols,
            rows,
            cursor: 0,
            first_pick: None,
            second_pick: None,
            attempts: 0,
            pairs_found: 0,
            total_pairs: pairs,
            difficulty,
            result: None,
            needs_dismiss: false,
        }
    }

    /// La partie est-elle terminée ?
    pub fn is_over(&self) -> bool {
        self.result.is_some()
    }

    /// Icône d'une carte donnée.
    pub fn card_icon(&self, index: usize) -> &'static str {
        ICONS[self.cards[index] % ICONS.len()]
    }

    /// La carte à `index` est-elle visible (retournée ou matchée) ?
    pub fn is_visible(&self, index: usize) -> bool {
        matches!(self.states[index], CardState::Revealed | CardState::Matched)
    }

    // ── Déplacements du curseur ─────────────────────────────────

    pub fn move_cursor_up(&mut self) {
        if self.cursor >= self.cols {
            self.cursor -= self.cols;
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.cursor + self.cols < self.cards.len() {
            self.cursor += self.cols;
        }
    }

    pub fn move_cursor_left(&mut self) {
        if !self.cursor % self.cols == 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor % self.cols < self.cols - 1 {
            self.cursor += 1;
        }
    }

    // ── Actions ─────────────────────────────────────────────────

    /// Le joueur retourne la carte sous le curseur.
    /// Retourne `true` si l'action a été effectuée.
    pub fn reveal(&mut self) -> bool {
        if self.is_over() {
            return false;
        }

        // Si on doit d'abord cacher les cartes non-matchées
        if self.needs_dismiss {
            self.dismiss();
            return true;
        }

        let idx = self.cursor;

        // Ne peut pas retourner une carte déjà visible
        if self.states[idx] != CardState::Hidden {
            return false;
        }

        self.states[idx] = CardState::Revealed;

        match self.first_pick {
            None => {
                // Première carte
                self.first_pick = Some(idx);
            }
            Some(first) => {
                // Deuxième carte
                self.second_pick = Some(idx);
                self.attempts += 1;

                if self.cards[first] == self.cards[idx] {
                    // Paire trouvée !
                    self.states[first] = CardState::Matched;
                    self.states[idx] = CardState::Matched;
                    self.pairs_found += 1;
                    self.first_pick = None;
                    self.second_pick = None;

                    // Vérifier si toutes les paires sont trouvées
                    if self.pairs_found == self.total_pairs {
                        self.result = Some(self.evaluate_result());
                    }
                } else {
                    // Pas de paire — marquer pour dismiss
                    self.needs_dismiss = true;
                }
            }
        }

        true
    }

    /// Cache les deux cartes non-matchées (appelé après dismiss).
    pub fn dismiss(&mut self) {
        if let (Some(first), Some(second)) = (self.first_pick, self.second_pick) {
            self.states[first] = CardState::Hidden;
            self.states[second] = CardState::Hidden;
        }
        self.first_pick = None;
        self.second_pick = None;
        self.needs_dismiss = false;
    }

    /// Évalue le résultat en fonction du nombre de tentatives.
    fn evaluate_result(&self) -> MinigameResult {
        let perfect = self.difficulty.perfect_attempts();
        let ratio = self.attempts as f32 / perfect as f32;

        if ratio <= 1.5 {
            MinigameResult::Win // Excellent
        } else if ratio <= 2.5 {
            MinigameResult::Draw // Correct
        } else {
            MinigameResult::Loss // Trop de tentatives
        }
    }

    // ── Récompenses ─────────────────────────────────────────────

    /// Calcule la récompense du mini-jeu.
    pub fn reward(&self) -> StatReward {
        let Some(result) = self.result else {
            return StatReward::none();
        };

        match result {
            MinigameResult::Win => match self.difficulty {
                Difficulty::Easy => StatReward {
                    hp: 0,
                    attack: 0,
                    defense: 1,
                    speed: 0,
                    special_attack: 0,
                    special_defense: 1,
                    xp: 20,
                },
                Difficulty::Medium => StatReward {
                    hp: 1,
                    attack: 0,
                    defense: 2,
                    speed: 0,
                    special_attack: 0,
                    special_defense: 2,
                    xp: 35,
                },
                Difficulty::Hard => StatReward {
                    hp: 2,
                    attack: 0,
                    defense: 2,
                    speed: 1,
                    special_attack: 1,
                    special_defense: 2,
                    xp: 55,
                },
            },
            MinigameResult::Draw => StatReward {
                hp: 0,
                attack: 0,
                defense: 1,
                speed: 0,
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
            Some(MinigameResult::Win) => "Excellent !",
            Some(MinigameResult::Draw) => "Pas mal.",
            Some(MinigameResult::Loss) => "Trop de tentatives...",
            None => "En cours...",
        }
    }

    /// Coordonnées (ligne, colonne) à partir de l'index.
    pub fn row_col(&self, idx: usize) -> (usize, usize) {
        (idx / self.cols, idx % self.cols)
    }
}

// ── Tests ───────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_game_not_over() {
        let game = MemoryGame::new(Difficulty::Easy);
        assert!(!game.is_over());
        assert_eq!(game.pairs_found, 0);
        assert_eq!(game.attempts, 0);
        assert_eq!(game.cards.len(), 12); // 4×3
    }

    #[test]
    fn medium_has_correct_size() {
        let game = MemoryGame::new(Difficulty::Medium);
        assert_eq!(game.cards.len(), 16); // 4×4
    }

    #[test]
    fn hard_has_correct_size() {
        let game = MemoryGame::new(Difficulty::Hard);
        assert_eq!(game.cards.len(), 20); // 4×5
    }

    #[test]
    fn matching_pair_stays_visible() {
        let mut game = MemoryGame::new(Difficulty::Easy);
        // Find two cards with the same value
        let first_val = game.cards[0];
        let second_idx = game
            .cards
            .iter()
            .enumerate()
            .position(|(i, &v)| i != 0 && v == first_val)
            .unwrap();

        game.cursor = 0;
        game.reveal();
        game.cursor = second_idx;
        game.reveal();

        assert_eq!(game.states[0], CardState::Matched);
        assert_eq!(game.states[second_idx], CardState::Matched);
        assert_eq!(game.pairs_found, 1);
    }

    #[test]
    fn non_matching_pair_requires_dismiss() {
        let mut game = MemoryGame::new(Difficulty::Easy);
        // Find two cards with different values
        let first_val = game.cards[0];
        let second_idx = game
            .cards
            .iter()
            .enumerate()
            .position(|(i, &v)| i != 0 && v != first_val)
            .unwrap();

        game.cursor = 0;
        game.reveal();
        game.cursor = second_idx;
        game.reveal();

        assert!(game.needs_dismiss);
        assert_eq!(game.pairs_found, 0);

        game.dismiss();
        assert_eq!(game.states[0], CardState::Hidden);
        assert_eq!(game.states[second_idx], CardState::Hidden);
        assert!(!game.needs_dismiss);
    }

    #[test]
    fn complete_game_gives_result() {
        let mut game = MemoryGame::new(Difficulty::Easy);
        // Cheat: reveal all pairs by using known card positions
        while !game.is_over() {
            // Find the first hidden card
            let first = game.states.iter().position(|s| *s == CardState::Hidden);
            let Some(first) = first else { break };

            let val = game.cards[first];
            let second = game
                .cards
                .iter()
                .enumerate()
                .position(|(i, &v)| i != first && v == val && game.states[i] == CardState::Hidden)
                .unwrap();

            game.cursor = first;
            game.reveal();
            game.cursor = second;
            game.reveal();

            if game.needs_dismiss {
                game.dismiss();
            }
        }

        assert!(game.is_over());
        assert!(game.result.is_some());
    }

    #[test]
    fn reward_for_win() {
        let mut game = MemoryGame::new(Difficulty::Easy);
        game.result = Some(MinigameResult::Win);
        let r = game.reward();
        assert!(!r.is_empty());
        assert!(r.defense > 0);
    }
}
