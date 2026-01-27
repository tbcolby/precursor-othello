//! Othello game engine with AI
//!
//! A complete Othello/Reversi implementation with:
//! - Efficient bitboard representation
//! - Move generation and validation
//! - Game state management with full history
//! - AI with multiple difficulty levels
//!
//! This library is `no_std` compatible (disable default features).

#![cfg_attr(not(feature = "std"), no_std)]

mod board;
mod moves;
mod game;
mod eval;
mod ai;
mod opening;

pub use board::{Board, Player};
pub use moves::{Move, MoveList};
pub use game::{GameState, GameResult};
pub use eval::evaluate;
pub use ai::{Difficulty, find_best_move};
pub use opening::OpeningBook;

/// Position on the board (0-63)
pub type Position = u8;

/// Convert (row, col) to position
#[inline]
pub const fn pos(row: u8, col: u8) -> Position {
    row * 8 + col
}

/// Convert position to (row, col)
#[inline]
pub const fn pos_to_rc(pos: Position) -> (u8, u8) {
    (pos / 8, pos % 8)
}

/// Convert position to algebraic notation (e.g., "D3")
pub fn pos_to_algebraic(pos: Position) -> [u8; 2] {
    let (row, col) = pos_to_rc(pos);
    [b'A' + col, b'1' + row]
}

/// Parse algebraic notation to position
pub fn algebraic_to_pos(s: &[u8]) -> Option<Position> {
    if s.len() != 2 {
        return None;
    }
    let col = s[0].to_ascii_uppercase();
    let row = s[1];
    if !(b'A'..=b'H').contains(&col) || !(b'1'..=b'8').contains(&row) {
        return None;
    }
    Some((row - b'1') * 8 + (col - b'A'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_conversion() {
        assert_eq!(pos(0, 0), 0);
        assert_eq!(pos(7, 7), 63);
        assert_eq!(pos(3, 3), 27); // D4
        assert_eq!(pos_to_rc(27), (3, 3));
    }

    #[test]
    fn test_algebraic() {
        assert_eq!(pos_to_algebraic(27), [b'D', b'4']);
        assert_eq!(algebraic_to_pos(b"D4"), Some(27));
        assert_eq!(algebraic_to_pos(b"A1"), Some(0));
        assert_eq!(algebraic_to_pos(b"H8"), Some(63));
    }
}
