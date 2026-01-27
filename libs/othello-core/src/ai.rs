//! AI opponent with multiple difficulty levels
//!
//! Implements minimax with alpha-beta pruning,
//! iterative deepening, and endgame solving.

use crate::{Board, MoveList, Player, Position};
use crate::eval::{evaluate, Score, SCORE_LOSS, SCORE_WIN};
use crate::moves::{count_moves, generate_moves};
use crate::opening::OpeningBook;

/// AI difficulty levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Expert,
}

impl Difficulty {
    /// Get the search depth for this difficulty
    pub const fn depth(&self) -> u8 {
        match self {
            Difficulty::Easy => 2,
            Difficulty::Medium => 4,
            Difficulty::Hard => 6,
            Difficulty::Expert => 8,
        }
    }

    /// Whether to use endgame solving
    pub const fn use_endgame_solver(&self) -> bool {
        match self {
            Difficulty::Easy | Difficulty::Medium => false,
            Difficulty::Hard | Difficulty::Expert => true,
        }
    }

    /// Whether to use opening book
    pub const fn use_opening_book(&self) -> bool {
        matches!(self, Difficulty::Expert)
    }

    /// Endgame solver threshold (empty squares)
    pub const fn endgame_threshold(&self) -> u32 {
        match self {
            Difficulty::Easy => 0,
            Difficulty::Medium => 0,
            Difficulty::Hard => 12,
            Difficulty::Expert => 14,
        }
    }
}

/// Search state for the AI
struct SearchState {
    nodes_searched: u32,
}

impl SearchState {
    fn new() -> Self {
        Self { nodes_searched: 0 }
    }
}

/// Apply a move to a board, returning the new board
fn apply_move(board: &Board, player: Player, pos: Position, flipped: u64) -> Board {
    let mut new_board = *board;
    new_board.place(player, pos);
    new_board.flip(player.opponent(), flipped);
    new_board
}

/// Order moves for better alpha-beta pruning
fn order_moves(board: &Board, player: Player, moves: &MoveList) -> [usize; 32] {
    let mut indices: [usize; 32] = core::array::from_fn(|i| i);
    let mut scores: [Score; 32] = [0; 32];

    for i in 0..moves.len() {
        let m = moves.get(i).unwrap();
        let new_board = apply_move(board, player, m.pos, m.flipped);

        // Score based on position quality
        let mut score = 0i32;

        // Corners are best
        if m.pos == 0 || m.pos == 7 || m.pos == 56 || m.pos == 63 {
            score += 1000;
        }
        // X-squares are worst
        else if m.pos == 9 || m.pos == 14 || m.pos == 49 || m.pos == 54 {
            score -= 500;
        }
        // C-squares are bad
        else if [1, 6, 8, 15, 48, 55, 57, 62].contains(&m.pos) {
            score -= 200;
        }
        // Edge positions are good
        else if m.pos < 8 || m.pos >= 56 || m.pos % 8 == 0 || m.pos % 8 == 7 {
            score += 100;
        }

        // More flips is generally good
        score += m.flip_count() as i32 * 5;

        // Opponent mobility after our move
        let opp_mobility = count_moves(&new_board, player.opponent()) as i32;
        score -= opp_mobility * 3;

        scores[i] = score;
    }

    // Sort indices by score (descending)
    for i in 0..moves.len() {
        for j in i + 1..moves.len() {
            if scores[indices[j]] > scores[indices[i]] {
                indices.swap(i, j);
            }
        }
    }

    indices
}

/// Minimax with alpha-beta pruning
fn alphabeta(
    board: &Board,
    player: Player,
    depth: u8,
    mut alpha: Score,
    mut beta: Score,
    maximizing: bool,
    state: &mut SearchState,
) -> Score {
    state.nodes_searched += 1;

    // Terminal depth or game over
    if depth == 0 {
        return evaluate(board, player);
    }

    let current = if maximizing { player } else { player.opponent() };
    let moves = generate_moves(board, current);

    if moves.is_empty() {
        // No moves - check if opponent can move
        let opponent = current.opponent();
        let opp_moves = generate_moves(board, opponent);

        if opp_moves.is_empty() {
            // Game over
            return evaluate(board, player);
        }

        // Pass - search opponent's moves at same depth
        return alphabeta(board, player, depth, alpha, beta, !maximizing, state);
    }

    let ordered = order_moves(board, current, &moves);

    if maximizing {
        let mut max_eval = SCORE_LOSS;

        for &idx in &ordered[..moves.len()] {
            let m = moves.get(idx).unwrap();
            let new_board = apply_move(board, current, m.pos, m.flipped);
            let eval = alphabeta(&new_board, player, depth - 1, alpha, beta, false, state);
            max_eval = max_eval.max(eval);
            alpha = alpha.max(eval);

            if beta <= alpha {
                break; // Beta cutoff
            }
        }

        max_eval
    } else {
        let mut min_eval = SCORE_WIN;

        for &idx in &ordered[..moves.len()] {
            let m = moves.get(idx).unwrap();
            let new_board = apply_move(board, current, m.pos, m.flipped);
            let eval = alphabeta(&new_board, player, depth - 1, alpha, beta, true, state);
            min_eval = min_eval.min(eval);
            beta = beta.min(eval);

            if beta <= alpha {
                break; // Alpha cutoff
            }
        }

        min_eval
    }
}

/// Endgame solver - perfect play search
fn solve_endgame(
    board: &Board,
    player: Player,
    mut alpha: Score,
    beta: Score,
    maximizing: bool,
    state: &mut SearchState,
) -> Score {
    state.nodes_searched += 1;

    let current = if maximizing { player } else { player.opponent() };
    let moves = generate_moves(board, current);

    if moves.is_empty() {
        let opponent = current.opponent();
        let opp_moves = generate_moves(board, opponent);

        if opp_moves.is_empty() {
            // Game over - exact score
            let own = board.count(player) as Score;
            let opp = board.count(player.opponent()) as Score;
            return if own > opp {
                SCORE_WIN - opp
            } else if opp > own {
                SCORE_LOSS + own
            } else {
                0
            };
        }

        return solve_endgame(board, player, alpha, beta, !maximizing, state);
    }

    let ordered = order_moves(board, current, &moves);

    if maximizing {
        let mut max_eval = SCORE_LOSS;

        for &idx in &ordered[..moves.len()] {
            let m = moves.get(idx).unwrap();
            let new_board = apply_move(board, current, m.pos, m.flipped);
            let eval = solve_endgame(&new_board, player, alpha, beta, false, state);
            max_eval = max_eval.max(eval);
            alpha = alpha.max(eval);

            if beta <= alpha {
                break;
            }
        }

        max_eval
    } else {
        let mut min_eval = SCORE_WIN;

        for &idx in &ordered[..moves.len()] {
            let m = moves.get(idx).unwrap();
            let new_board = apply_move(board, current, m.pos, m.flipped);
            let eval = solve_endgame(&new_board, player, alpha, beta, true, state);
            min_eval = min_eval.min(eval);

            if beta <= alpha {
                break;
            }
        }

        min_eval
    }
}

/// Find the best move for the given difficulty
pub fn find_best_move(
    board: &Board,
    player: Player,
    difficulty: Difficulty,
) -> Option<Position> {
    let moves = generate_moves(board, player);
    if moves.is_empty() {
        return None;
    }

    // Single move - no need to search
    if moves.len() == 1 {
        return Some(moves.get(0).unwrap().pos);
    }

    // Check opening book for Expert
    if difficulty.use_opening_book() {
        if let Some(book_move) = OpeningBook::lookup(board) {
            return Some(book_move);
        }
    }

    let empty = board.empty_count();
    let mut state = SearchState::new();

    // Endgame solving
    if difficulty.use_endgame_solver() && empty <= difficulty.endgame_threshold() {
        return find_best_move_endgame(board, player, &moves, &mut state);
    }

    // Regular search
    let depth = difficulty.depth();
    let ordered = order_moves(board, player, &moves);

    let mut best_pos = moves.get(ordered[0]).unwrap().pos;
    let mut best_score = SCORE_LOSS;

    for &idx in &ordered[..moves.len()] {
        let m = moves.get(idx).unwrap();
        let new_board = apply_move(board, player, m.pos, m.flipped);
        let score = alphabeta(
            &new_board,
            player,
            depth - 1,
            SCORE_LOSS,
            SCORE_WIN,
            false,
            &mut state,
        );

        if score > best_score {
            best_score = score;
            best_pos = m.pos;
        }
    }

    Some(best_pos)
}

/// Find best move using endgame solver
fn find_best_move_endgame(
    board: &Board,
    player: Player,
    moves: &MoveList,
    state: &mut SearchState,
) -> Option<Position> {
    let ordered = order_moves(board, player, moves);

    let mut best_pos = moves.get(ordered[0]).unwrap().pos;
    let mut best_score = SCORE_LOSS;

    for &idx in &ordered[..moves.len()] {
        let m = moves.get(idx).unwrap();
        let new_board = apply_move(board, player, m.pos, m.flipped);
        let score = solve_endgame(
            &new_board,
            player,
            SCORE_LOSS,
            SCORE_WIN,
            false,
            state,
        );

        if score > best_score {
            best_score = score;
            best_pos = m.pos;
        }
    }

    Some(best_pos)
}

/// Get a random legal move (for testing)
#[cfg(feature = "std")]
#[allow(dead_code)]
pub fn random_move(board: &Board, player: Player) -> Option<Position> {
    let moves = generate_moves(board, player);
    if moves.is_empty() {
        return None;
    }

    // Use a simple counter for pseudo-randomness in tests
    let idx = (board.hash() as usize) % moves.len();
    Some(moves.get(idx).unwrap().pos)
}

/// Get a hint (best move) for the player
#[allow(dead_code)]
pub fn get_hint(board: &Board, player: Player) -> Option<Position> {
    find_best_move(board, player, Difficulty::Hard)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moves::calculate_flips;

    #[test]
    fn test_find_best_move_opening() {
        let board = Board::new();

        // Easy difficulty should find a move
        let best = find_best_move(&board, Player::Black, Difficulty::Easy);
        assert!(best.is_some());

        // Move should be legal
        let pos = best.unwrap();
        let flipped = calculate_flips(&board, Player::Black, pos);
        assert!(flipped != 0);
    }

    #[test]
    fn test_all_difficulties() {
        let board = Board::new();

        for difficulty in [
            Difficulty::Easy,
            Difficulty::Medium,
            Difficulty::Hard,
            Difficulty::Expert,
        ] {
            let best = find_best_move(&board, Player::Black, difficulty);
            assert!(best.is_some(), "Difficulty {:?} failed", difficulty);

            let pos = best.unwrap();
            assert!(calculate_flips(&board, Player::Black, pos) != 0);
        }
    }

    #[test]
    fn test_forced_move() {
        // Create position with only one legal move
        let mut board = Board::empty();
        board.place(Player::Black, 0);  // A1
        board.place(Player::White, 1);  // B1

        let best = find_best_move(&board, Player::Black, Difficulty::Easy);
        // Only legal move is C1 (position 2)
        if let Some(pos) = best {
            assert!(calculate_flips(&board, Player::Black, pos) != 0);
        }
    }

    #[test]
    fn test_corner_preference() {
        // Create position where corner is available
        let mut board = Board::empty();

        // Set up so A1 is a valid move for black
        board.place(Player::White, 1);  // B1
        board.place(Player::Black, 2);  // C1

        let best = find_best_move(&board, Player::Black, Difficulty::Medium);
        // AI should prefer corner A1
        if let Some(pos) = best {
            if calculate_flips(&board, Player::Black, 0) != 0 {
                assert_eq!(pos, 0, "AI should take corner");
            }
        }
    }

    #[test]
    fn test_endgame() {
        // Create near-endgame position
        let mut board = Board::empty();

        // Fill most of the board
        for i in 0..60 {
            if i % 2 == 0 {
                board.place(Player::Black, i);
            } else {
                board.place(Player::White, i);
            }
        }

        // This won't have legal moves, but tests the code path
        let result = find_best_move(&board, Player::Black, Difficulty::Hard);
        // May or may not find a move depending on position
        let _ = result;
    }

    #[test]
    fn test_move_ordering() {
        let board = Board::new();
        let moves = generate_moves(&board, Player::Black);
        let ordered = order_moves(&board, Player::Black, &moves);

        // Should have 4 moves
        assert!(moves.len() == 4);

        // First moves in ordering should be the ones with best quick eval
        assert!(ordered[0] < moves.len());
    }
}
