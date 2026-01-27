//! Bitboard representation for Othello
//!
//! Uses two 64-bit integers to represent black and white discs.
//! Each bit corresponds to a board position (0 = A1, 63 = H8).

use crate::Position;

/// Player color
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Player {
    Black,
    White,
}

impl Player {
    /// Get the opponent
    #[inline]
    pub const fn opponent(self) -> Player {
        match self {
            Player::Black => Player::White,
            Player::White => Player::Black,
        }
    }
}

/// Othello board using bitboard representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Board {
    /// Bitboard for black discs
    pub black: u64,
    /// Bitboard for white discs
    pub white: u64,
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Board {
    /// Create a new board with the standard starting position
    pub const fn new() -> Self {
        // Standard Othello starting position:
        // D4=White, E4=Black, D5=Black, E5=White
        // D4 = row 3, col 3 = position 27
        // E4 = row 3, col 4 = position 28
        // D5 = row 4, col 3 = position 35
        // E5 = row 4, col 4 = position 36
        let black = (1u64 << 28) | (1u64 << 35); // E4, D5
        let white = (1u64 << 27) | (1u64 << 36); // D4, E5
        Self { black, white }
    }

    /// Create an empty board
    pub const fn empty() -> Self {
        Self { black: 0, white: 0 }
    }

    /// Get the bitboard for a player
    #[inline]
    pub const fn get(&self, player: Player) -> u64 {
        match player {
            Player::Black => self.black,
            Player::White => self.white,
        }
    }

    /// Get mutable reference to a player's bitboard
    #[inline]
    pub fn get_mut(&mut self, player: Player) -> &mut u64 {
        match player {
            Player::Black => &mut self.black,
            Player::White => &mut self.white,
        }
    }

    /// Check if a position has a disc of the given player
    #[inline]
    pub const fn has_disc(&self, player: Player, pos: Position) -> bool {
        (self.get(player) & (1u64 << pos)) != 0
    }

    /// Check if a position is empty
    #[inline]
    pub const fn is_empty(&self, pos: Position) -> bool {
        ((self.black | self.white) & (1u64 << pos)) == 0
    }

    /// Check if a position is occupied
    #[inline]
    pub const fn is_occupied(&self, pos: Position) -> bool {
        ((self.black | self.white) & (1u64 << pos)) != 0
    }

    /// Get the player at a position, if any
    pub const fn get_disc(&self, pos: Position) -> Option<Player> {
        let mask = 1u64 << pos;
        if (self.black & mask) != 0 {
            Some(Player::Black)
        } else if (self.white & mask) != 0 {
            Some(Player::White)
        } else {
            None
        }
    }

    /// Place a disc for a player
    #[inline]
    pub fn place(&mut self, player: Player, pos: Position) {
        *self.get_mut(player) |= 1u64 << pos;
    }

    /// Remove a disc (used for undo)
    #[inline]
    pub fn remove(&mut self, player: Player, pos: Position) {
        *self.get_mut(player) &= !(1u64 << pos);
    }

    /// Flip discs from one player to another
    #[inline]
    pub fn flip(&mut self, from: Player, flipped: u64) {
        let to = from.opponent();
        *self.get_mut(from) &= !flipped;
        *self.get_mut(to) |= flipped;
    }

    /// Count discs for a player
    #[inline]
    pub const fn count(&self, player: Player) -> u32 {
        self.get(player).count_ones()
    }

    /// Count empty squares
    #[inline]
    pub const fn empty_count(&self) -> u32 {
        (!(self.black | self.white)).count_ones()
    }

    /// Get all empty positions as a bitboard
    #[inline]
    pub const fn empty_squares(&self) -> u64 {
        !(self.black | self.white)
    }

    /// Get all occupied positions as a bitboard
    #[inline]
    pub const fn occupied(&self) -> u64 {
        self.black | self.white
    }

    /// Get a hash of the board position (for transposition tables)
    pub const fn hash(&self) -> u64 {
        // Simple hash combining both bitboards
        self.black.wrapping_mul(0x9e3779b97f4a7c15) ^ self.white
    }

    /// Check if the board is full
    #[inline]
    pub const fn is_full(&self) -> bool {
        (self.black | self.white) == u64::MAX
    }
}

/// Bit manipulation utilities
impl Board {
    /// Get the lowest set bit position
    #[inline]
    pub const fn lowest_bit_pos(bits: u64) -> Option<Position> {
        if bits == 0 {
            None
        } else {
            Some(bits.trailing_zeros() as Position)
        }
    }

    /// Iterate over set bit positions
    pub fn iter_bits(mut bits: u64) -> impl Iterator<Item = Position> {
        core::iter::from_fn(move || {
            if bits == 0 {
                None
            } else {
                let pos = bits.trailing_zeros() as Position;
                bits &= bits - 1; // Clear lowest bit
                Some(pos)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pos;

    #[test]
    fn test_starting_position() {
        let board = Board::new();
        assert_eq!(board.count(Player::Black), 2);
        assert_eq!(board.count(Player::White), 2);
        assert_eq!(board.empty_count(), 60);

        // Check specific positions
        assert!(board.has_disc(Player::Black, pos(3, 4))); // E4
        assert!(board.has_disc(Player::Black, pos(4, 3))); // D5
        assert!(board.has_disc(Player::White, pos(3, 3))); // D4
        assert!(board.has_disc(Player::White, pos(4, 4))); // E5
    }

    #[test]
    fn test_place_and_remove() {
        let mut board = Board::empty();
        assert!(board.is_empty(0));

        board.place(Player::Black, 0);
        assert!(board.has_disc(Player::Black, 0));
        assert!(!board.is_empty(0));

        board.remove(Player::Black, 0);
        assert!(board.is_empty(0));
    }

    #[test]
    fn test_flip() {
        let mut board = Board::empty();
        board.place(Player::Black, 0);
        board.place(Player::Black, 1);
        board.place(Player::Black, 2);

        let to_flip = (1u64 << 1) | (1u64 << 2);
        board.flip(Player::Black, to_flip);

        assert!(board.has_disc(Player::Black, 0));
        assert!(board.has_disc(Player::White, 1));
        assert!(board.has_disc(Player::White, 2));
    }

    #[test]
    fn test_iter_bits() {
        let bits = 0b1010_0101u64;
        let positions: Vec<_> = Board::iter_bits(bits).collect();
        assert_eq!(positions, vec![0, 2, 5, 7]);
    }

    #[test]
    fn test_get_disc() {
        let board = Board::new();
        assert_eq!(board.get_disc(pos(3, 3)), Some(Player::White)); // D4
        assert_eq!(board.get_disc(pos(3, 4)), Some(Player::Black)); // E4
        assert_eq!(board.get_disc(pos(0, 0)), None); // A1 empty
    }
}
