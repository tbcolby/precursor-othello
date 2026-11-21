//! Move generation and validation for Othello
//!
//! Implements efficient bitboard-based move generation using
//! directional shift operations for all 8 directions.

use crate::{Board, Player, Position};

/// Direction shifts for move generation.
/// Each tuple: (shift amount, mask applied to the *source* bits before shifting).
/// Bit index = row * 8 + col, so file A is col 0 (bits 0, 8, ...) and file H is
/// col 7. A shift by +/-1, +/-7 or +/-9 wraps across the edge unless the source
/// squares that would wrap are masked out first:
/// * moves toward higher columns (+1, +9, -7) must not start on file H
/// * moves toward lower columns (-1, +7, -9) must not start on file A
const DIRECTIONS: [(i8, u64); 8] = [
    (1, NOT_H),
    (-1, NOT_A),
    (8, u64::MAX),
    (-8, u64::MAX),
    (9, NOT_H),
    (7, NOT_A),
    (-7, NOT_H),
    (-9, NOT_A),
];

const NOT_A: u64 = 0xfefe_fefe_fefe_fefe;
const NOT_H: u64 = 0x7f7f_7f7f_7f7f_7f7f;

/// Shift a bitboard in a direction (mask applied before the shift).
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

/// Largest mobility seen in any reachable Othello position is 33; 34 leaves headroom.
pub const MAX_MOVES_PER_POSITION: usize = 34;

/// A list of legal moves

#[derive(Debug, Clone)]
pub struct MoveList {
    moves: [Move; MAX_MOVES_PER_POSITION],
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
            moves: [Move::new(0, 0); MAX_MOVES_PER_POSITION],
            len: 0,
        }
    }

    /// Add a move to the list
    pub fn push(&mut self, m: Move) {
        if self.len < MAX_MOVES_PER_POSITION {
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
    if pos >= 64 || board.is_occupied(pos) {
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
    for pos in Board::iter_bits(legal_moves_bitboard(board, player)) {
        moves.push(Move::new(pos, calculate_flips(board, player, pos)));
    }
    moves
}

/// Count legal moves without generating the full list.
pub fn count_moves(board: &Board, player: Player) -> u32 {
    legal_moves_bitboard(board, player).count_ones()
}

/// Check if a specific move is legal
pub fn is_legal_move(board: &Board, player: Player, pos: Position) -> bool {
    if pos >= 64 || board.is_occupied(pos) {
        return false;
    }
    calculate_flips(board, player, pos) != 0
}

/// Get all legal move positions as a bitboard (Dumb7Fill in each direction).
pub fn legal_moves_bitboard(board: &Board, player: Player) -> u64 {
    let own = board.get(player);
    let opp = board.get(player.opponent());
    let empty = board.empty_squares();
    let mut legal = 0u64;
    for &(dir, mask) in &DIRECTIONS {
        // Runs of opponent discs adjacent to our discs in this direction...
        let mut run = shift(own, dir, mask) & opp;
        for _ in 0..5 {
            run |= shift(run, dir, mask) & opp;
        }
        // ...that end on an empty square.
        legal |= shift(run, dir, mask) & empty;
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

    fn board_from(black: &[Position], white: &[Position]) -> Board {
        let mut b = Board::empty();
        for &p in black {
            b.place(Player::Black, p);
        }
        for &p in white {
            b.place(Player::White, p);
        }
        b
    }

    // Regression for audit O-1: direction masks were swapped/truncated, so edge
    // captures failed. White B1, Black C1: Black A1 flips B1 (rightward capture
    // starting on file A).
    #[test]
    fn edge_capture_rightward_from_file_a() {
        let b = board_from(&[2], &[1]);
        assert_eq!(calculate_flips(&b, Player::Black, 0), 1 << 1);
    }

    #[test]
    fn edge_capture_leftward_from_file_h() {
        // Black F1, White G1, Black plays H1.
        let b = board_from(&[5], &[6]);
        assert_eq!(calculate_flips(&b, Player::Black, 7), 1 << 6);
    }

    #[test]
    fn row_eight_horizontal() {
        // Row 8 (bits 56..63) was zeroed by the 14-digit mask.
        let b = board_from(&[58], &[57]);
        assert_eq!(calculate_flips(&b, Player::Black, 56), 1 << 57);
        let b = board_from(&[61], &[62]);
        assert_eq!(calculate_flips(&b, Player::Black, 63), 1 << 62);
    }

    #[test]
    fn no_wraparound_across_edges() {
        // Black H3 (23), White A4 (24): next to each other in bit order only.
        // Black playing... nothing flips across the board edge.
        let b = board_from(&[23], &[24]);
        assert_eq!(calculate_flips(&b, Player::Black, 25), 0);
        // White at H2 (15), Black A3 (16): a bit-adjacent wrap pair.
        let b = board_from(&[16], &[15]);
        assert_eq!(calculate_flips(&b, Player::Black, 14), 0);
        assert_eq!(legal_moves_bitboard(&b, Player::Black), 0);
    }

    #[test]
    fn diagonals_in_all_four_directions() {
        // Centre target D4 (27). Black at distance 2 on each diagonal, White between.
        for (white, black) in [(18u8, 9u8), (20, 13), (34, 41), (36, 45)] {
            let b = board_from(&[black], &[white]);
            assert_eq!(calculate_flips(&b, Player::Black, 27), 1 << white, "diag via {white}");
        }
    }

    #[test]
    fn multi_direction_flip() {
        // Black plays D4 (27); flips C4 (26, west, anchor B4=25) and D5 (35, south, anchor D6=43)
        // and E5 (36, SE diagonal, anchor F6=45).
        let b = board_from(&[25, 43, 45], &[26, 35, 36]);
        assert_eq!(calculate_flips(&b, Player::Black, 27), (1 << 26) | (1 << 35) | (1 << 36));
    }

    /// Slow, obviously-correct reference: walk rays with (row, col) arithmetic.
    fn naive_flips(board: &Board, player: Player, pos: Position) -> u64 {
        if board.is_occupied(pos) {
            return 0;
        }
        let (r0, c0) = ((pos / 8) as i32, (pos % 8) as i32);
        let mut flipped = 0u64;
        for dr in -1i32..=1 {
            for dc in -1i32..=1 {
                if dr == 0 && dc == 0 {
                    continue;
                }
                let (mut r, mut c) = (r0 + dr, c0 + dc);
                let mut run = 0u64;
                while (0..8).contains(&r) && (0..8).contains(&c) {
                    let p = (r * 8 + c) as u8;
                    if board.has_disc(player.opponent(), p) {
                        run |= 1 << p;
                    } else {
                        if board.has_disc(player, p) {
                            flipped |= run;
                        }
                        break;
                    }
                    r += dr;
                    c += dc;
                }
            }
        }
        flipped
    }

    /// Deterministic xorshift for reproducible random playouts.
    fn xorshift(state: &mut u64) -> u64 {
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;
        *state
    }

    #[test]
    fn differential_against_naive_over_random_playouts() {
        let mut seed = 0x1234_5678_9abc_def1u64;
        let mut positions = 0;
        for _ in 0..150 {
            let mut b = Board::new();
            let mut player = Player::Black;
            for _ply in 0..80 {
                let mut legal = 0u64;
                for p in 0..64u8 {
                    let f = calculate_flips(&b, player, p);
                    assert_eq!(f, naive_flips(&b, player, p), "flips at {p}");
                    if f != 0 {
                        legal |= 1 << p;
                    }
                }
                assert_eq!(legal_moves_bitboard(&b, player), legal);
                assert_eq!(count_moves(&b, player), legal.count_ones());
                positions += 1;
                if legal == 0 {
                    if legal_moves_bitboard(&b, player.opponent()) == 0 {
                        break;
                    }
                    player = player.opponent();
                    continue;
                }
                let n = legal.count_ones() as u64;
                let pick = (xorshift(&mut seed) % n) as usize;
                let p = Board::iter_bits(legal).nth(pick).unwrap();
                let f = calculate_flips(&b, player, p);
                b.place(player, p);
                b.flip(player.opponent(), f);
                player = player.opponent();
            }
        }
        assert!(positions > 5000);
    }

    fn perft(board: &Board, player: Player, depth: u32) -> u64 {
        if depth == 0 {
            return 1;
        }
        let moves = generate_moves(board, player);
        if moves.is_empty() {
            if legal_moves_bitboard(board, player.opponent()) == 0 {
                return 1; // game over: a leaf
            }
            return perft(board, player.opponent(), depth - 1);
        }
        let mut n = 0;
        for m in moves.iter() {
            let mut nb = *board;
            nb.place(player, m.pos);
            nb.flip(player.opponent(), m.flipped);
            n += perft(&nb, player.opponent(), depth - 1);
        }
        n
    }

    // Standard Othello perft values from the initial position (passes count as a ply).
    #[test]
    fn perft_standard_counts() {
        let b = Board::new();
        let expected = [1u64, 4, 12, 56, 244, 1396, 8200];
        for (d, &e) in expected.iter().enumerate() {
            assert_eq!(perft(&b, Player::Black, d as u32), e, "perft({d})");
        }
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
