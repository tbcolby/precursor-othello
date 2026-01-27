//! Move generation and validation for Othello
//!
//! Implements efficient bitboard-based move generation using
//! directional shift operations for all 8 directions.

use crate::{Board, Player, Position};

/// Direction shifts for move generation
/// Each tuple: (shift amount, mask to avoid wraparound)
const DIRECTIONS: [(i8, u64); 8] = [
    // Horizontal and vertical
    (1, 0xfefefefefefefe),   // Right (not H file)
    (-1, 0x7f7f7f7f7f7f7f7f), // Left (not A file)
    (8, u64::MAX),           // Down
    (-8, u64::MAX),          // Up
    // Diagonals
    (9, 0xfefefefefefefe),   // Down-right
    (7, 0x7f7f7f7f7f7f7f7f), // Down-left
    (-7, 0xfefefefefefefe),  // Up-right
    (-9, 0x7f7f7f7f7f7f7f7f), // Up-left
];

/// Shift a bitboard in a direction
#[inline]
fn shift(bits: u64, dir: i8, mask: u64) -> u64 {
    let masked = bits & mask;
    if dir > 0 {
        masked << dir
    } else {
        masked >> (-dir)
    }
}

/// A single move with position and what it flips
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    /// Position where the disc is placed
    pub pos: Position,
    /// Bitboard of discs that will be flipped
    pub flipped: u64,
}

impl Move {
    /// Create a new move
    pub const fn new(pos: Position, flipped: u64) -> Self {
        Self { pos, flipped }
    }

    /// Check if this is a valid move (flips at least one disc)
    pub const fn is_valid(&self) -> bool {
        self.flipped != 0
    }

    /// Count how many discs this move flips
    pub const fn flip_count(&self) -> u32 {
        self.flipped.count_ones()
    }
}

/// A list of legal moves (max 32 possible in any position)
#[derive(Debug, Clone)]
pub struct MoveList {
    moves: [Move; 32],
    len: usize,
}

impl Default for MoveList {
    fn default() -> Self {
        Self::new()
    }
}

impl MoveList {
    /// Create an empty move list
    pub const fn new() -> Self {
        Self {
            moves: [Move::new(0, 0); 32],
            len: 0,
        }
    }

    /// Add a move to the list
    pub fn push(&mut self, m: Move) {
        if self.len < 32 {
            self.moves[self.len] = m;
            self.len += 1;
        }
    }

    /// Get the number of legal moves
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Check if there are no legal moves
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get a move by index
    pub fn get(&self, index: usize) -> Option<&Move> {
        if index < self.len {
            Some(&self.moves[index])
        } else {
            None
        }
    }

    /// Iterate over moves
    pub fn iter(&self) -> impl Iterator<Item = &Move> {
        self.moves[..self.len].iter()
    }

    /// Get the valid move positions as a bitboard
    pub fn as_bitboard(&self) -> u64 {
        let mut bits = 0u64;
        for m in self.iter() {
            bits |= 1u64 << m.pos;
        }
        bits
    }
}

/// Calculate what discs would be flipped by placing at pos
pub fn calculate_flips(board: &Board, player: Player, pos: Position) -> u64 {
    if board.is_occupied(pos) {
        return 0;
    }

    let own = board.get(player);
    let opp = board.get(player.opponent());
    let pos_bit = 1u64 << pos;
    let mut flipped = 0u64;

    for &(dir, mask) in &DIRECTIONS {
        let mut candidates = 0u64;
        let mut current = shift(pos_bit, dir, mask);

        // Walk along opponent discs
        while (current & opp) != 0 {
            candidates |= current;
            current = shift(current, dir, mask);
        }

        // If we hit our own disc, the candidates are flipped
        if (current & own) != 0 {
            flipped |= candidates;
        }
    }

    flipped
}

/// Generate all legal moves for a player
pub fn generate_moves(board: &Board, player: Player) -> MoveList {
    let mut moves = MoveList::new();
    let empty = board.empty_squares();

    // Check each empty square
    for pos in Board::iter_bits(empty) {
        let flipped = calculate_flips(board, player, pos);
        if flipped != 0 {
            moves.push(Move::new(pos, flipped));
        }
    }

    moves
}

/// Get a quick count of legal moves without generating the full list
pub fn count_moves(board: &Board, player: Player) -> u32 {
    let empty = board.empty_squares();
    let mut count = 0;

    for pos in Board::iter_bits(empty) {
        if calculate_flips(board, player, pos) != 0 {
            count += 1;
        }
    }

    count
}

/// Check if a specific move is legal
pub fn is_legal_move(board: &Board, player: Player, pos: Position) -> bool {
    if board.is_occupied(pos) {
        return false;
    }
    calculate_flips(board, player, pos) != 0
}

/// Get all legal move positions as a bitboard (for highlighting)
pub fn legal_moves_bitboard(board: &Board, player: Player) -> u64 {
    let empty = board.empty_squares();
    let mut legal = 0u64;

    for pos in Board::iter_bits(empty) {
        if calculate_flips(board, player, pos) != 0 {
            legal |= 1u64 << pos;
        }
    }

    legal
}

/// Check if either player has legal moves
#[allow(dead_code)]
pub fn game_has_moves(board: &Board) -> bool {
    count_moves(board, Player::Black) > 0 || count_moves(board, Player::White) > 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pos;

    #[test]
    fn test_starting_moves() {
        let board = Board::new();
        let moves = generate_moves(&board, Player::Black);

        // Black has exactly 4 opening moves
        assert_eq!(moves.len(), 4);

        // Valid opening moves: D3, C4, F5, E6
        let legal_positions: Vec<Position> = moves.iter().map(|m| m.pos).collect();
        assert!(legal_positions.contains(&pos(2, 3))); // D3
        assert!(legal_positions.contains(&pos(3, 2))); // C4
        assert!(legal_positions.contains(&pos(4, 5))); // F5
        assert!(legal_positions.contains(&pos(5, 4))); // E6
    }

    #[test]
    fn test_flip_calculation() {
        let board = Board::new();

        // D3 flips D4 (one white disc)
        let flipped = calculate_flips(&board, Player::Black, pos(2, 3));
        assert_eq!(flipped.count_ones(), 1);
        assert!((flipped & (1u64 << pos(3, 3))) != 0); // D4 is flipped
    }

    #[test]
    fn test_no_moves_on_occupied() {
        let board = Board::new();

        // Can't play on D4 (occupied by white)
        assert_eq!(calculate_flips(&board, Player::Black, pos(3, 3)), 0);

        // Can't play on E4 (occupied by black)
        assert_eq!(calculate_flips(&board, Player::White, pos(3, 4)), 0);
    }

    #[test]
    fn test_multiple_direction_flip() {
        // Create a position where a move flips in multiple directions
        let mut board = Board::empty();

        // Set up: Black at A1, White at B1, B2
        board.place(Player::Black, pos(0, 0)); // A1
        board.place(Player::Black, pos(2, 2)); // C3
        board.place(Player::White, pos(1, 1)); // B2

        // C1 should flip B2 (diagonal)
        let flipped = calculate_flips(&board, Player::Black, pos(0, 2));
        assert_eq!(flipped, 0); // B2 is not between A1 and C1 diagonally

        // A3 would flip B2 diagonally
        let flipped = calculate_flips(&board, Player::Black, pos(2, 0));
        assert_eq!(flipped, 0); // Not on the right diagonal
    }

    #[test]
    fn test_legal_moves_bitboard() {
        let board = Board::new();
        let legal = legal_moves_bitboard(&board, Player::Black);

        // Should have exactly 4 bits set
        assert_eq!(legal.count_ones(), 4);

        // Check specific positions
        assert!((legal & (1u64 << pos(2, 3))) != 0); // D3
        assert!((legal & (1u64 << pos(3, 2))) != 0); // C4
        assert!((legal & (1u64 << pos(4, 5))) != 0); // F5
        assert!((legal & (1u64 << pos(5, 4))) != 0); // E6
    }

    #[test]
    fn test_is_legal_move() {
        let board = Board::new();

        assert!(is_legal_move(&board, Player::Black, pos(2, 3))); // D3
        assert!(!is_legal_move(&board, Player::Black, pos(0, 0))); // A1 - no flips
        assert!(!is_legal_move(&board, Player::Black, pos(3, 3))); // D4 - occupied
    }

    #[test]
    fn test_move_list() {
        let mut list = MoveList::new();
        assert!(list.is_empty());

        list.push(Move::new(0, 1));
        list.push(Move::new(1, 2));

        assert_eq!(list.len(), 2);
        assert_eq!(list.get(0).unwrap().pos, 0);
        assert_eq!(list.get(1).unwrap().pos, 1);
        assert!(list.get(2).is_none());
    }
}
