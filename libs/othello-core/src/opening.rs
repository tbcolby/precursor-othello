//! Opening book for Othello
//!
//! Contains known good opening moves for Expert difficulty.
//! Uses board hash to quickly lookup positions.

use crate::{Board, Position};

/// Opening book with hash-based lookup
pub struct OpeningBook;

impl OpeningBook {
    /// Lookup a position in the opening book
    ///
    /// Returns the best move if the position is in the book.
    pub fn lookup(board: &Board) -> Option<Position> {
        // Normalize board by checking all 8 symmetries
        let hash = Self::normalized_hash(board);

        // Binary search in sorted book
        Self::BOOK.iter()
            .find(|(h, _)| *h == hash)
            .map(|(_, pos)| *pos)
    }

    /// Get normalized hash considering symmetries
    fn normalized_hash(board: &Board) -> u64 {
        let mut min_hash = board.hash();

        // Check all 8 symmetries (4 rotations x 2 mirrors)
        let mut b = *board;

        for _ in 0..4 {
            let h = b.hash();
            if h < min_hash {
                min_hash = h;
            }

            let mirrored = Self::mirror_board(&b);
            let mh = mirrored.hash();
            if mh < min_hash {
                min_hash = mh;
            }

            b = Self::rotate_board(&b);
        }

        min_hash
    }

    /// Rotate board 90 degrees clockwise
    fn rotate_board(board: &Board) -> Board {
        let mut black = 0u64;
        let mut white = 0u64;

        for row in 0..8 {
            for col in 0..8 {
                let old_pos = row * 8 + col;
                let new_row = col;
                let new_col = 7 - row;
                let new_pos = new_row * 8 + new_col;

                if (board.black & (1u64 << old_pos)) != 0 {
                    black |= 1u64 << new_pos;
                }
                if (board.white & (1u64 << old_pos)) != 0 {
                    white |= 1u64 << new_pos;
                }
            }
        }

        Board { black, white }
    }

    /// Mirror board horizontally
    fn mirror_board(board: &Board) -> Board {
        let mut black = 0u64;
        let mut white = 0u64;

        for row in 0..8 {
            for col in 0..8 {
                let old_pos = row * 8 + col;
                let new_pos = row * 8 + (7 - col);

                if (board.black & (1u64 << old_pos)) != 0 {
                    black |= 1u64 << new_pos;
                }
                if (board.white & (1u64 << old_pos)) != 0 {
                    white |= 1u64 << new_pos;
                }
            }
        }

        Board { black, white }
    }

    /// Opening book entries (hash, best_move)
    /// These are common tournament openings
    const BOOK: &'static [(u64, Position)] = &[
        // Starting position responses
        // D3 (perpendicular opening)
        (0x0810000000000000_u64 ^ 0x1008000000000000, 19), // C5

        // More openings can be added here
        // Format: (normalized_board_hash, best_move_position)
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotate_board() {
        let mut board = Board::empty();
        board.place(crate::Player::Black, 0); // A1

        let rotated = OpeningBook::rotate_board(&board);
        // A1 rotated 90 CW -> H1
        assert!((rotated.black & (1u64 << 7)) != 0);
    }

    #[test]
    fn test_mirror_board() {
        let mut board = Board::empty();
        board.place(crate::Player::Black, 0); // A1

        let mirrored = OpeningBook::mirror_board(&board);
        // A1 mirrored -> H1
        assert!((mirrored.black & (1u64 << 7)) != 0);
    }

    #[test]
    fn test_normalized_hash() {
        let mut board1 = Board::empty();
        board1.place(crate::Player::Black, 0); // A1

        let mut board2 = Board::empty();
        board2.place(crate::Player::Black, 7); // H1 (mirror of A1)

        // Both should normalize to the same hash
        let h1 = OpeningBook::normalized_hash(&board1);
        let h2 = OpeningBook::normalized_hash(&board2);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_lookup_starting_position() {
        let board = Board::new();
        // Starting position may or may not be in book
        let _result = OpeningBook::lookup(&board);
        // Just verify it doesn't crash
    }
}
