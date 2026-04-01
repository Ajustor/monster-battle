//! Morpion (Tic-Tac-Toe) — mini-jeu pour entraîner un monstre.
//!
//! Le joueur est toujours **X** et commence. L'IA est **O** et utilise un
//! algorithme minimax avec élagage alpha-bêta (IA imbattable en difficulté
//! Hard, aléatoire en Easy, mix en Medium).
//!
//! La récompense en stats dépend de la difficulté et du résultat :
//! - **Victoire** : bonus de stats de base + XP.
//! - **Nul** : petit bonus de XP.
//! - **Défaite** : rien.

use rand::Rng;

use super::{MinigameResult, StatReward};

// ── Types ───────────────────────────────────────────────────────

/// Contenu d'une case du morpion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Empty,
    X,
    O,
}

impl Cell {
    /// Symbole d'affichage.
    pub fn symbol(self) -> &'static str {
        match self {
            Cell::Empty => " ",
            Cell::X => "X",
            Cell::O => "O",
        }
    }
}

/// Difficulté de l'IA.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    /// L'IA joue aléatoirement.
    Easy,
    /// L'IA joue minimax 50 % du temps, aléatoire sinon.
    Medium,
    /// L'IA joue toujours le coup optimal (minimax).
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

    /// Toutes les difficultés dans l'ordre.
    pub fn all() -> &'static [Difficulty] {
        &[Difficulty::Easy, Difficulty::Medium, Difficulty::Hard]
    }
}

/// État complet d'une partie de morpion.
#[derive(Debug, Clone)]
pub struct TicTacToe {
    /// Grille 3×3, indices 0–8 (row-major).
    pub board: [Cell; 9],
    /// Tour courant (X ou O). X commence toujours.
    pub current_turn: Cell,
    /// Curseur du joueur sur la grille (0–8).
    pub cursor: usize,
    /// Résultat de la partie (None si en cours).
    pub result: Option<MinigameResult>,
    /// Difficulté de l'IA.
    pub difficulty: Difficulty,
    /// Le combo gagnant (indices de la ligne victorieuse) pour l'affichage.
    pub winning_line: Option<[usize; 3]>,
}

impl TicTacToe {
    /// Crée une nouvelle partie.
    pub fn new(difficulty: Difficulty) -> Self {
        Self {
            board: [Cell::Empty; 9],
            current_turn: Cell::X,
            cursor: 4, // centre
            result: None,
            difficulty,
            winning_line: None,
        }
    }

    /// La partie est-elle terminée ?
    pub fn is_over(&self) -> bool {
        self.result.is_some()
    }

    // ── Déplacements du curseur ─────────────────────────────────

    pub fn move_cursor_up(&mut self) {
        if self.cursor >= 3 {
            self.cursor -= 3;
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.cursor + 3 < 9 {
            self.cursor += 3;
        }
    }

    pub fn move_cursor_left(&mut self) {
        if !self.cursor.is_multiple_of(3) {
            self.cursor -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor % 3 < 2 {
            self.cursor += 1;
        }
    }

    // ── Actions ─────────────────────────────────────────────────

    /// Le joueur tente de placer X à la position `cursor`.
    /// Retourne `true` si le coup a été joué (case vide, partie pas finie).
    pub fn play(&mut self) -> bool {
        if self.is_over() || self.current_turn != Cell::X {
            return false;
        }
        if self.board[self.cursor] != Cell::Empty {
            return false;
        }

        self.board[self.cursor] = Cell::X;
        if self.check_end() {
            return true;
        }

        // Tour de l'IA
        self.current_turn = Cell::O;
        self.ai_play();
        self.check_end();
        true
    }

    /// L'IA joue son coup.
    fn ai_play(&mut self) {
        if self.is_over() {
            return;
        }

        let idx = match self.difficulty {
            Difficulty::Easy => self.random_move(),
            Difficulty::Hard => self.best_move(),
            Difficulty::Medium => {
                let mut rng = rand::thread_rng();
                if rng.gen_bool(0.5) {
                    self.best_move()
                } else {
                    self.random_move()
                }
            }
        };

        if let Some(i) = idx {
            self.board[i] = Cell::O;
            self.current_turn = Cell::X;
        }
    }

    /// Coup aléatoire parmi les cases vides.
    fn random_move(&self) -> Option<usize> {
        let empties: Vec<usize> = self
            .board
            .iter()
            .enumerate()
            .filter(|(_, c)| **c == Cell::Empty)
            .map(|(i, _)| i)
            .collect();
        if empties.is_empty() {
            return None;
        }
        let mut rng = rand::thread_rng();
        Some(empties[rng.gen_range(0..empties.len())])
    }

    /// Meilleur coup via minimax (pour l'IA = O, maximisant le score O).
    fn best_move(&self) -> Option<usize> {
        let mut best_score = i32::MIN;
        let mut best_idx = None;

        for i in 0..9 {
            if self.board[i] != Cell::Empty {
                continue;
            }
            let mut board = self.board;
            board[i] = Cell::O;
            let score = minimax(&board, false, i32::MIN, i32::MAX);
            if score > best_score {
                best_score = score;
                best_idx = Some(i);
            }
        }

        best_idx
    }

    // ── Vérification de fin de partie ───────────────────────────

    fn check_end(&mut self) -> bool {
        if let Some((winner, line)) = check_winner(&self.board) {
            self.winning_line = Some(line);
            self.result = Some(if winner == Cell::X {
                MinigameResult::Win
            } else {
                MinigameResult::Loss
            });
            return true;
        }
        if self.board.iter().all(|c| *c != Cell::Empty) {
            self.result = Some(MinigameResult::Draw);
            return true;
        }
        false
    }

    // ── Récompenses ─────────────────────────────────────────────

    /// Calcule la récompense en fonction du résultat et de la difficulté.
    pub fn reward(&self) -> StatReward {
        let Some(result) = self.result else {
            return StatReward::none();
        };

        match result {
            MinigameResult::Win => match self.difficulty {
                Difficulty::Easy => StatReward {
                    hp: 1,
                    attack: 0,
                    defense: 0,
                    speed: 1,
                    special_attack: 0,
                    special_defense: 0,
                    xp: 15,
                },
                Difficulty::Medium => StatReward {
                    hp: 1,
                    attack: 1,
                    defense: 1,
                    speed: 1,
                    special_attack: 0,
                    special_defense: 0,
                    xp: 30,
                },
                Difficulty::Hard => StatReward {
                    hp: 2,
                    attack: 1,
                    defense: 1,
                    speed: 1,
                    special_attack: 1,
                    special_defense: 1,
                    xp: 50,
                },
            },
            MinigameResult::Draw => StatReward {
                hp: 0,
                attack: 0,
                defense: 0,
                speed: 0,
                special_attack: 0,
                special_defense: 0,
                xp: 10,
            },
            MinigameResult::Loss => StatReward::none(),
        }
    }

    /// Label du résultat pour l'affichage.
    pub fn result_label(&self) -> &'static str {
        match self.result {
            Some(MinigameResult::Win) => "Victoire !",
            Some(MinigameResult::Draw) => "Match nul.",
            Some(MinigameResult::Loss) => "Défaite...",
            None => "En cours...",
        }
    }

    /// Ligne et colonne à partir de l'index (0–8).
    pub fn row_col(idx: usize) -> (usize, usize) {
        (idx / 3, idx % 3)
    }
}

// ── Minimax avec alpha-bêta ─────────────────────────────────────

const WIN_LINES: [[usize; 3]; 8] = [
    [0, 1, 2],
    [3, 4, 5],
    [6, 7, 8], // rows
    [0, 3, 6],
    [1, 4, 7],
    [2, 5, 8], // cols
    [0, 4, 8],
    [2, 4, 6], // diags
];

/// Vérifie s'il y a un gagnant. Retourne le gagnant et la ligne gagnante.
fn check_winner(board: &[Cell; 9]) -> Option<(Cell, [usize; 3])> {
    for line in &WIN_LINES {
        let a = board[line[0]];
        if a != Cell::Empty && a == board[line[1]] && a == board[line[2]] {
            return Some((a, *line));
        }
    }
    None
}

/// Minimax avec élagage alpha-bêta.
/// `is_maximizing` = true quand c'est au tour de O (IA).
fn minimax(board: &[Cell; 9], is_maximizing: bool, mut alpha: i32, mut beta: i32) -> i32 {
    // Terminal ?
    if let Some((winner, _)) = check_winner(board) {
        return if winner == Cell::O { 10 } else { -10 };
    }
    if board.iter().all(|c| *c != Cell::Empty) {
        return 0;
    }

    if is_maximizing {
        let mut best = i32::MIN;
        for i in 0..9 {
            if board[i] != Cell::Empty {
                continue;
            }
            let mut b = *board;
            b[i] = Cell::O;
            let score = minimax(&b, false, alpha, beta);
            best = best.max(score);
            alpha = alpha.max(score);
            if beta <= alpha {
                break;
            }
        }
        best
    } else {
        let mut best = i32::MAX;
        for i in 0..9 {
            if board[i] != Cell::Empty {
                continue;
            }
            let mut b = *board;
            b[i] = Cell::X;
            let score = minimax(&b, true, alpha, beta);
            best = best.min(score);
            beta = beta.min(score);
            if beta <= alpha {
                break;
            }
        }
        best
    }
}

// ── Tests ───────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_game_is_not_over() {
        let game = TicTacToe::new(Difficulty::Easy);
        assert!(!game.is_over());
        assert_eq!(game.current_turn, Cell::X);
    }

    #[test]
    fn player_can_play() {
        let mut game = TicTacToe::new(Difficulty::Easy);
        game.cursor = 0;
        assert!(game.play());
        assert_eq!(game.board[0], Cell::X);
    }

    #[test]
    fn cannot_play_on_occupied_cell() {
        let mut game = TicTacToe::new(Difficulty::Easy);
        game.cursor = 0;
        game.play();
        // Find which cell AI played on, try to play on X's cell again
        game.cursor = 0;
        assert!(!game.play());
    }

    #[test]
    fn hard_ai_never_loses() {
        // Play 50 random games against Hard AI; it should never lose.
        for _ in 0..50 {
            let mut game = TicTacToe::new(Difficulty::Hard);
            let mut rng = rand::thread_rng();
            while !game.is_over() {
                let empties: Vec<usize> = game
                    .board
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| **c == Cell::Empty)
                    .map(|(i, _)| i)
                    .collect();
                if empties.is_empty() {
                    break;
                }
                game.cursor = empties[rng.gen_range(0..empties.len())];
                game.play();
            }
            assert_ne!(game.result, Some(MinigameResult::Win));
        }
    }

    #[test]
    fn reward_for_win_easy() {
        let mut game = TicTacToe::new(Difficulty::Easy);
        game.result = Some(MinigameResult::Win);
        let r = game.reward();
        assert!(r.xp > 0);
        assert!(!r.is_empty());
    }

    #[test]
    fn reward_for_loss_is_empty() {
        let mut game = TicTacToe::new(Difficulty::Hard);
        game.result = Some(MinigameResult::Loss);
        let r = game.reward();
        assert!(r.is_empty());
    }
}
