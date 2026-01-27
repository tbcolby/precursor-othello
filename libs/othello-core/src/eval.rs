//! Position evaluation for Othello AI
//!
//! Implements a multi-factor evaluation function considering:
//! - Corner control and stability
//! - Mobility (number of legal moves)
//! - Frontier discs
//! - Disc count (weighted by game phase)

use crate::{Board, Player, Position};
use crate::moves::count_moves;

/// Evaluation score (positive = good for player, negative = bad)
pub type Score = i32;

/// Maximum possible score (winning position)
pub const SCORE_WIN: Score = 100_000;
/// Minimum possible score (losing position)
pub const SCORE_LOSS: Score = -100_000;

/// Corner positions (A1, H1, A8, H8)
const CORNERS: [Position; 4] = [0, 7, 56, 63];

/// X-squares (diagonal to corners, dangerous when corner empty)
const X_SQUARES: [(Position, Position); 4] = [
    (9, 0),   // B2 -> A1
    (14, 7),  // G2 -> H1
    (49, 56), // B7 -> A8
    (54, 63), // G7 -> H8
];

/// C-squares (adjacent to corners, somewhat dangerous)
const C_SQUARES: [(Position, Position); 8] = [
    (1, 0),   // B1 -> A1
    (8, 0),   // A2 -> A1
    (6, 7),   // G1 -> H1
    (15, 7),  // H2 -> H1
    (48, 56), // A7 -> A8
    (57, 56), // B8 -> A8
    (55, 63), // H7 -> H8
    (62, 63), // G8 -> H8
];

/// Edge masks for stability calculation
const EDGES: [u64; 4] = [
    0xFF,                 // Top edge (row 0)
    0xFF00000000000000,   // Bottom edge (row 7)
    0x0101010101010101,   // Left edge (col 0)
    0x8080808080808080,   // Right edge (col 7)
];

/// Neighbor masks for frontier calculation (precomputed would be faster)
fn neighbor_mask(pos: Position) -> u64 {
    let row = pos / 8;
    let col = pos % 8;
    let mut mask = 0u64;

    for dr in -1i8..=1 {
        for dc in -1i8..=1 {
            if dr == 0 && dc == 0 {
                continue;
            }
            let nr = row as i8 + dr;
            let nc = col as i8 + dc;
            if nr >= 0 && nr < 8 && nc >= 0 && nc < 8 {
                mask |= 1u64 << (nr as u8 * 8 + nc as u8);
            }
        }
    }

    mask
}

/// Count frontier discs (discs adjacent to empty squares)
fn count_frontier(board: &Board, player: Player) -> u32 {
    let own = board.get(player);
    let empty = board.empty_squares();
    let mut frontier = 0;

    for pos in Board::iter_bits(own) {
        if (neighbor_mask(pos) & empty) != 0 {
            frontier += 1;
        }
    }

    frontier
}

/// Evaluate corner and X/C-square control
fn evaluate_corners(board: &Board, player: Player) -> Score {
    let own = board.get(player);
    let opp = board.get(player.opponent());
    let mut score = 0;

    // Corner control: very valuable
    for corner in CORNERS {
        let mask = 1u64 << corner;
        if (own & mask) != 0 {
            score += 100;
        } else if (opp & mask) != 0 {
            score -= 100;
        }
    }

    // X-squares: dangerous when adjacent corner is empty
    for (x_sq, corner) in X_SQUARES {
        let corner_mask = 1u64 << corner;
        let x_mask = 1u64 << x_sq;

        // Only penalize if corner is empty
        if (own | opp) & corner_mask == 0 {
            if (own & x_mask) != 0 {
                score -= 25;
            } else if (opp & x_mask) != 0 {
                score += 25;
            }
        }
    }

    // C-squares: somewhat dangerous when adjacent corner is empty
    for (c_sq, corner) in C_SQUARES {
        let corner_mask = 1u64 << corner;
        let c_mask = 1u64 << c_sq;

        if (own | opp) & corner_mask == 0 {
            if (own & c_mask) != 0 {
                score -= 10;
            } else if (opp & c_mask) != 0 {
                score += 10;
            }
        }
    }

    score
}

/// Evaluate mobility (number of legal moves)
fn evaluate_mobility(board: &Board, player: Player) -> Score {
    let own_moves = count_moves(board, player) as Score;
    let opp_moves = count_moves(board, player.opponent()) as Score;

    (own_moves - opp_moves) * 3
}

/// Evaluate frontier discs (fewer is better)
fn evaluate_frontier(board: &Board, player: Player) -> Score {
    let own_frontier = count_frontier(board, player) as Score;
    let opp_frontier = count_frontier(board, player.opponent()) as Score;

    // Fewer frontier discs is better
    opp_frontier - own_frontier
}

/// Evaluate disc count (weighted by game phase)
fn evaluate_disc_count(board: &Board, player: Player) -> Score {
    let own = board.count(player) as Score;
    let opp = board.count(player.opponent()) as Score;
    let empty = board.empty_count();

    // Weight disc count more heavily in endgame
    let weight = if empty > 44 {
        0 // Early game: ignore disc count
    } else if empty > 20 {
        1 // Mid game: slight consideration
    } else if empty > 10 {
        2 // Late game: more important
    } else {
        5 // Endgame: primary factor
    };

    (own - opp) * weight
}

/// Count stable discs (cannot be flipped)
/// Simplified version: only counts corner-anchored stable discs
fn count_stable_discs(board: &Board, player: Player) -> u32 {
    let own = board.get(player);
    let mut stable = 0u64;

    // Discs in corners are always stable
    for corner in CORNERS {
        if (own & (1u64 << corner)) != 0 {
            stable |= 1u64 << corner;
        }
    }

    // Expand from corners along filled edges
    // This is a simplified version - full stability is complex
    let occupied = board.occupied();

    // For each edge, if it's completely filled and includes a corner we own,
    // all our discs on that edge are stable
    for (edge_mask, corners) in [
        (EDGES[0], [0u8, 7]),      // Top edge
        (EDGES[1], [56u8, 63]),    // Bottom edge
        (EDGES[2], [0u8, 56]),     // Left edge
        (EDGES[3], [7u8, 63]),     // Right edge
    ] {
        if (occupied & edge_mask) == edge_mask {
            // Edge is full, check if we have a corner
            for corner in corners {
                if (own & (1u64 << corner)) != 0 {
                    stable |= own & edge_mask;
                    break;
                }
            }
        }
    }

    stable.count_ones()
}

/// Evaluate edge stability
fn evaluate_stability(board: &Board, player: Player) -> Score {
    let own_stable = count_stable_discs(board, player) as Score;
    let opp_stable = count_stable_discs(board, player.opponent()) as Score;

    (own_stable - opp_stable) * 10
}

/// Full position evaluation
///
/// Returns a score from the perspective of the given player.
/// Positive = good for player, negative = bad.
pub fn evaluate(board: &Board, player: Player) -> Score {
    // Check for terminal position
    let own_moves = count_moves(board, player);
    let opp_moves = count_moves(board, player.opponent());

    if own_moves == 0 && opp_moves == 0 {
        // Game over - evaluate by disc count
        let own = board.count(player) as Score;
        let opp = board.count(player.opponent()) as Score;

        return if own > opp {
            SCORE_WIN - (opp * 100) // Win by more is better
        } else if opp > own {
            SCORE_LOSS + (own * 100) // Lose by less is better
        } else {
            0 // Draw
        };
    }

    // Combine evaluation factors
    let mut score = 0;

    score += evaluate_corners(board, player);
    score += evaluate_mobility(board, player);
    score += evaluate_frontier(board, player);
    score += evaluate_disc_count(board, player);
    score += evaluate_stability(board, player);

    score
}

/// Quick evaluation for move ordering
/// Faster but less accurate than full evaluation
#[allow(dead_code)]
pub fn quick_evaluate(board: &Board, player: Player) -> Score {
    let mut score = 0;

    // Just corners and mobility
    let own = board.get(player);
    let opp = board.get(player.opponent());

    for corner in CORNERS {
        let mask = 1u64 << corner;
        if (own & mask) != 0 {
            score += 100;
        } else if (opp & mask) != 0 {
            score -= 100;
        }
    }

    let own_moves = count_moves(board, player) as Score;
    let opp_moves = count_moves(board, player.opponent()) as Score;
    score += (own_moves - opp_moves) * 3;

    score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_starting_position_neutral() {
        let board = Board::new();
        let black_eval = evaluate(&board, Player::Black);
        let white_eval = evaluate(&board, Player::White);

        // Starting position should be roughly equal
        assert!(black_eval.abs() < 20);
        assert_eq!(black_eval, -white_eval);
    }

    #[test]
    fn test_corner_value() {
        let mut board = Board::empty();

        // Give black a corner
        board.place(Player::Black, 0); // A1

        let eval = evaluate(&board, Player::Black);
        assert!(eval > 50); // Corner is valuable
    }

    #[test]
    fn test_x_square_penalty() {
        // Test that X-squares are penalized relative to non-X-squares
        let mut board1 = Board::new();
        let mut board2 = Board::new();

        // Add black on X-square (B2) in board1
        board1.place(Player::Black, 9); // B2 - X-square

        // Add black on safe square in board2
        board2.place(Player::Black, 18); // C3 - not X-square

        let eval1 = evaluate_corners(&board1, Player::Black);
        let eval2 = evaluate_corners(&board2, Player::Black);

        // X-square should have lower eval than non-X-square
        assert!(eval1 < eval2, "X-square should be penalized");
    }

    #[test]
    fn test_game_over_evaluation() {
        let mut board = Board::empty();

        // Black wins 40-24
        for i in 0..40 {
            board.place(Player::Black, i);
        }
        for i in 40..64 {
            board.place(Player::White, i);
        }

        let eval = evaluate(&board, Player::Black);
        assert!(eval > SCORE_WIN - 5000); // Should be a winning score
    }

    #[test]
    fn test_mobility_value() {
        let board = Board::new();

        // Both players have equal mobility at start
        let mobility_eval = evaluate_mobility(&board, Player::Black);
        assert_eq!(mobility_eval, 0);
    }

    #[test]
    fn test_stable_disc_counting() {
        let mut board = Board::empty();

        // Fill top edge with black, anchored at A1
        for col in 0..8 {
            board.place(Player::Black, col);
        }

        let stable = count_stable_discs(&board, Player::Black);
        assert_eq!(stable, 8); // All 8 discs on top edge are stable
    }

    #[test]
    fn test_frontier() {
        let board = Board::new();

        // In starting position, all 4 discs are frontier (next to empty)
        let black_frontier = count_frontier(&board, Player::Black);
        let white_frontier = count_frontier(&board, Player::White);

        assert_eq!(black_frontier, 2);
        assert_eq!(white_frontier, 2);
    }
}
