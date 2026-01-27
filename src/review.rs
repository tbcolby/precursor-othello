//! What If mode state management
//!
//! This module provides utilities for the What If review mode,
//! allowing players to step through game history and explore
//! alternate lines of play.

use othello_core::GameState;

/// What If mode state
pub struct WhatIfState {
    /// The original completed game
    pub base_game: GameState,
    /// Current game state (may be branched from base)
    pub current_game: GameState,
    /// Current position in base game history
    pub view_index: usize,
    /// Whether we've branched from the original game
    pub branched: bool,
}

impl WhatIfState {
    /// Create a new What If state from a completed game
    pub fn new(game: GameState) -> Self {
        let view_index = game.move_count();
        Self {
            base_game: game.clone(),
            current_game: game,
            view_index,
            branched: false,
        }
    }

    /// Step back one move in history
    pub fn step_back(&mut self) {
        if self.branched {
            // Can't step back once branched
            return;
        }

        if self.view_index > 0 {
            self.view_index -= 1;
            self.current_game = self.base_game.clone_at_move(self.view_index);
        }
    }

    /// Step forward one move in history
    pub fn step_forward(&mut self) {
        if self.branched {
            // Can't step forward once branched
            return;
        }

        if self.view_index < self.base_game.move_count() {
            self.view_index += 1;
            self.current_game = self.base_game.clone_at_move(self.view_index);
        }
    }

    /// Jump to the start of the game
    pub fn jump_to_start(&mut self) {
        if self.branched {
            return;
        }
        self.view_index = 0;
        self.current_game = self.base_game.clone_at_move(0);
    }

    /// Jump to the end of the game
    pub fn jump_to_end(&mut self) {
        if self.branched {
            return;
        }
        self.view_index = self.base_game.move_count();
        self.current_game = self.base_game.clone();
    }

    /// Make an alternate move, branching from the current position
    pub fn make_alternate_move(&mut self, pos: u8) -> bool {
        if self.current_game.is_legal(pos) {
            self.current_game.make_move(pos);
            self.branched = true;
            true
        } else {
            false
        }
    }

    /// Reset to a specific move in the base game
    pub fn reset_to_move(&mut self, index: usize) {
        self.view_index = index.min(self.base_game.move_count());
        self.current_game = self.base_game.clone_at_move(self.view_index);
        self.branched = false;
    }

    /// Reset to the start of the base game
    pub fn reset_to_start(&mut self) {
        self.reset_to_move(0);
    }

    /// Get the current move number being viewed
    pub fn current_move_number(&self) -> usize {
        if self.branched {
            self.view_index + (self.current_game.move_count() - self.base_game.clone_at_move(self.view_index).move_count())
        } else {
            self.view_index
        }
    }

    /// Get the total number of moves in the base game
    pub fn total_moves(&self) -> usize {
        self.base_game.move_count()
    }
}
