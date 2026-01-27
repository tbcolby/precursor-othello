//! Game state management with full history tracking

use crate::{Board, Move, MoveList, Player, Position};
use crate::moves::{calculate_flips, count_moves, generate_moves};

/// Maximum number of moves in a game (theoretical max is 60)
pub const MAX_MOVES: usize = 64;

/// Result of a completed game
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResult {
    /// Player won with disc count
    Win(Player, u32, u32),
    /// Draw with equal disc counts
    Draw(u32),
}

impl GameResult {
    /// Get the winner, if any
    pub const fn winner(&self) -> Option<Player> {
        match self {
            GameResult::Win(player, _, _) => Some(*player),
            GameResult::Draw(_) => None,
        }
    }

    /// Get the final disc counts (black, white)
    pub const fn counts(&self) -> (u32, u32) {
        match self {
            GameResult::Win(Player::Black, b, w) => (*b, *w),
            GameResult::Win(Player::White, b, w) => (*b, *w),
            GameResult::Draw(c) => (*c, *c),
        }
    }
}

/// A recorded move in history
#[derive(Debug, Clone, Copy)]
pub struct HistoryEntry {
    /// Position where disc was placed (255 = pass)
    pub pos: u8,
    /// Which discs were flipped
    pub flipped: u64,
    /// Player who made the move
    pub player: Player,
}

impl HistoryEntry {
    /// Check if this was a pass
    pub const fn is_pass(&self) -> bool {
        self.pos == 255
    }
}

/// Complete game state with history
#[derive(Debug, Clone)]
pub struct GameState {
    /// Current board position
    board: Board,
    /// Current player to move
    current_player: Player,
    /// Move history
    history: [HistoryEntry; MAX_MOVES],
    /// Number of moves in history
    history_len: usize,
    /// Consecutive passes (2 = game over)
    consecutive_passes: u8,
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameState {
    /// Create a new game with standard starting position
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            current_player: Player::Black,
            history: [HistoryEntry {
                pos: 0,
                flipped: 0,
                player: Player::Black,
            }; MAX_MOVES],
            history_len: 0,
            consecutive_passes: 0,
        }
    }

    /// Create a game from a specific board position
    pub fn from_board(board: Board, current_player: Player) -> Self {
        Self {
            board,
            current_player,
            history: [HistoryEntry {
                pos: 0,
                flipped: 0,
                player: Player::Black,
            }; MAX_MOVES],
            history_len: 0,
            consecutive_passes: 0,
        }
    }

    /// Get the current board
    pub const fn board(&self) -> &Board {
        &self.board
    }

    /// Get the current player to move
    pub const fn current_player(&self) -> Player {
        self.current_player
    }

    /// Get the number of moves made
    pub const fn move_count(&self) -> usize {
        self.history_len
    }

    /// Get move history
    pub fn history(&self) -> &[HistoryEntry] {
        &self.history[..self.history_len]
    }

    /// Get the last move made, if any
    pub fn last_move(&self) -> Option<&HistoryEntry> {
        if self.history_len > 0 {
            Some(&self.history[self.history_len - 1])
        } else {
            None
        }
    }

    /// Generate legal moves for the current player
    pub fn legal_moves(&self) -> MoveList {
        generate_moves(&self.board, self.current_player)
    }

    /// Check if the current player has any legal moves
    pub fn has_moves(&self) -> bool {
        count_moves(&self.board, self.current_player) > 0
    }

    /// Count legal moves for the current player
    pub fn count_legal_moves(&self) -> u32 {
        count_moves(&self.board, self.current_player)
    }

    /// Check if the game is over
    pub fn is_game_over(&self) -> bool {
        // Game ends when both players must pass consecutively
        self.consecutive_passes >= 2 || self.board.is_full()
    }

    /// Get the game result (only valid when game is over)
    pub fn result(&self) -> Option<GameResult> {
        if !self.is_game_over() {
            return None;
        }

        let black = self.board.count(Player::Black);
        let white = self.board.count(Player::White);

        Some(if black > white {
            GameResult::Win(Player::Black, black, white)
        } else if white > black {
            GameResult::Win(Player::White, black, white)
        } else {
            GameResult::Draw(black)
        })
    }

    /// Check if a move is legal
    pub fn is_legal(&self, pos: Position) -> bool {
        crate::moves::is_legal_move(&self.board, self.current_player, pos)
    }

    /// Make a move at the given position
    ///
    /// Returns the move made (with flip info) or None if illegal
    pub fn make_move(&mut self, pos: Position) -> Option<Move> {
        let flipped = calculate_flips(&self.board, self.current_player, pos);
        if flipped == 0 {
            return None;
        }

        // Place disc
        self.board.place(self.current_player, pos);

        // Flip opponent discs
        self.board.flip(self.current_player.opponent(), flipped);

        // Record in history
        if self.history_len < MAX_MOVES {
            self.history[self.history_len] = HistoryEntry {
                pos,
                flipped,
                player: self.current_player,
            };
            self.history_len += 1;
        }

        // Reset consecutive passes
        self.consecutive_passes = 0;

        // Switch player
        self.current_player = self.current_player.opponent();

        Some(Move::new(pos, flipped))
    }

    /// Pass the turn (when no legal moves)
    ///
    /// Returns true if the pass was valid
    pub fn pass(&mut self) -> bool {
        // Can only pass if no legal moves
        if self.has_moves() {
            return false;
        }

        // Record pass in history
        if self.history_len < MAX_MOVES {
            self.history[self.history_len] = HistoryEntry {
                pos: 255, // Special marker for pass
                flipped: 0,
                player: self.current_player,
            };
            self.history_len += 1;
        }

        self.consecutive_passes += 1;
        self.current_player = self.current_player.opponent();

        true
    }

    /// Undo the last move
    ///
    /// Returns the undone move or None if no history
    pub fn undo(&mut self) -> Option<HistoryEntry> {
        if self.history_len == 0 {
            return None;
        }

        self.history_len -= 1;
        let entry = self.history[self.history_len];

        if entry.is_pass() {
            // Undo pass
            self.consecutive_passes = self.consecutive_passes.saturating_sub(1);
        } else {
            // Undo move: remove placed disc and unflip
            self.board.remove(entry.player, entry.pos);
            self.board.flip(entry.player, entry.flipped);
            self.consecutive_passes = 0;
        }

        self.current_player = entry.player;

        Some(entry)
    }

    /// Get disc counts (black, white)
    pub fn counts(&self) -> (u32, u32) {
        (
            self.board.count(Player::Black),
            self.board.count(Player::White),
        )
    }

    /// Get empty square count
    pub fn empty_count(&self) -> u32 {
        self.board.empty_count()
    }

    /// Clone the game state at a specific move in history
    pub fn clone_at_move(&self, move_index: usize) -> Self {
        let mut game = Self::new();

        for entry in &self.history[..move_index.min(self.history_len)] {
            if entry.is_pass() {
                game.pass();
            } else {
                game.make_move(entry.pos);
            }
        }

        game
    }

    /// Get the board position after a specific move in history
    pub fn board_at_move(&self, move_index: usize) -> Board {
        self.clone_at_move(move_index).board
    }

    /// Get mobility (legal move count) for a player
    pub fn mobility(&self, player: Player) -> u32 {
        count_moves(&self.board, player)
    }

    /// Get legal moves bitboard for highlighting
    pub fn legal_moves_bitboard(&self) -> u64 {
        crate::moves::legal_moves_bitboard(&self.board, self.current_player)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pos;

    #[test]
    fn test_new_game() {
        let game = GameState::new();
        assert_eq!(game.current_player(), Player::Black);
        assert_eq!(game.move_count(), 0);
        assert!(!game.is_game_over());
    }

    #[test]
    fn test_make_move() {
        let mut game = GameState::new();

        // Black plays D3
        let result = game.make_move(pos(2, 3));
        assert!(result.is_some());

        let m = result.unwrap();
        assert_eq!(m.pos, pos(2, 3));
        assert_eq!(m.flip_count(), 1);

        // Now white's turn
        assert_eq!(game.current_player(), Player::White);
        assert_eq!(game.move_count(), 1);

        // Black should have 4 discs, white should have 1
        let (black, white) = game.counts();
        assert_eq!(black, 4);
        assert_eq!(white, 1);
    }

    #[test]
    fn test_undo() {
        let mut game = GameState::new();

        // Make a move
        game.make_move(pos(2, 3));
        assert_eq!(game.current_player(), Player::White);

        // Undo
        let undone = game.undo();
        assert!(undone.is_some());
        assert_eq!(undone.unwrap().pos, pos(2, 3));
        assert_eq!(game.current_player(), Player::Black);
        assert_eq!(game.move_count(), 0);

        // Board should be back to starting position
        let (black, white) = game.counts();
        assert_eq!(black, 2);
        assert_eq!(white, 2);
    }

    #[test]
    fn test_illegal_move() {
        let mut game = GameState::new();

        // Try to play on occupied square
        let result = game.make_move(pos(3, 3)); // D4 - white disc
        assert!(result.is_none());

        // Try to play where no flips
        let result = game.make_move(pos(0, 0)); // A1 - no flips
        assert!(result.is_none());

        // Game state should be unchanged
        assert_eq!(game.current_player(), Player::Black);
        assert_eq!(game.move_count(), 0);
    }

    #[test]
    fn test_clone_at_move() {
        let mut game = GameState::new();

        // Play a few moves
        game.make_move(pos(2, 3)); // D3
        game.make_move(pos(2, 2)); // C3
        game.make_move(pos(2, 1)); // B3

        // Clone at move 1
        let clone = game.clone_at_move(1);
        assert_eq!(clone.move_count(), 1);
        assert_eq!(clone.current_player(), Player::White);
    }

    #[test]
    fn test_pass() {
        // Create a position where white must pass
        let mut board = Board::empty();
        // Fill most of the board with black, leaving white with no moves
        board.black = u64::MAX & !0xFF; // All but first row
        board.white = 0x01; // Single white disc at A1

        let mut game = GameState::from_board(board, Player::White);

        // White should have no moves
        assert!(!game.has_moves());

        // Pass should succeed
        assert!(game.pass());
        assert_eq!(game.current_player(), Player::Black);
    }

    #[test]
    fn test_game_over() {
        let mut board = Board::empty();
        // Fill board completely
        board.black = 0xFFFFFFFF00000000;
        board.white = 0x00000000FFFFFFFF;

        let game = GameState::from_board(board, Player::Black);
        assert!(game.is_game_over());

        let result = game.result().unwrap();
        assert!(matches!(result, GameResult::Draw(32)));
    }

    #[test]
    fn test_history() {
        let mut game = GameState::new();

        game.make_move(pos(2, 3)); // D3
        game.make_move(pos(2, 2)); // C3

        let history = game.history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].pos, pos(2, 3));
        assert_eq!(history[0].player, Player::Black);
        assert_eq!(history[1].pos, pos(2, 2));
        assert_eq!(history[1].player, Player::White);
    }

    #[test]
    fn test_last_move() {
        let mut game = GameState::new();
        assert!(game.last_move().is_none());

        game.make_move(pos(2, 3));
        let last = game.last_move().unwrap();
        assert_eq!(last.pos, pos(2, 3));
    }
}
