//! "What If" review: step through a finished game and try alternate moves.

use crate::{GameState, Position};

/// What If state. Stepping always works (it abandons any branch); playing a
/// move marks the position as branched.
#[derive(Debug, Clone)]
pub struct WhatIf {
    base: GameState,
    current: GameState,
    view_index: usize,
    branched: bool,
}

impl WhatIf {
    pub fn new(game: GameState) -> Self {
        let view_index = game.move_count();
        Self { base: game.clone(), current: game, view_index, branched: false }
    }

    pub fn game(&self) -> &GameState { &self.current }

    pub fn branched(&self) -> bool { self.branched }

    /// History index being viewed (0 = start).
    pub fn index(&self) -> usize { self.view_index }

    pub fn total(&self) -> usize { self.base.move_count() }

    fn goto(&mut self, index: usize) {
        self.view_index = index.min(self.base.move_count());
        self.current = self.base.clone_at_move(self.view_index);
        self.branched = false;
    }

    pub fn step_back(&mut self) {
        let i = self.view_index.saturating_sub(1);
        self.goto(i);
    }

    pub fn step_forward(&mut self) {
        let i = self.view_index + 1;
        self.goto(i);
    }

    /// Play an alternate move at the viewed position. Passes that follow are
    /// applied automatically. Returns false if the move is illegal.
    pub fn play(&mut self, pos: Position) -> bool {
        if self.current.is_game_over() || self.current.make_move(pos).is_none() {
            return false;
        }
        self.branched = true;
        while !self.current.is_game_over() && !self.current.has_moves() {
            self.current.pass();
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pos;

    fn sample() -> GameState {
        let mut g = GameState::new();
        g.make_move(pos(2, 3));
        g.make_move(pos(2, 2));
        g.make_move(pos(2, 1));
        g
    }

    #[test]
    fn stepping_replays_history() {
        let mut w = WhatIf::new(sample());
        assert_eq!(w.index(), 3);
        w.step_back();
        w.step_back();
        assert_eq!(w.index(), 1);
        assert_eq!(w.game().move_count(), 1);
        w.step_forward();
        assert_eq!(w.game().move_count(), 2);
        for _ in 0..10 {
            w.step_forward();
        }
        assert_eq!(w.index(), 3);
    }

    #[test]
    fn branching_and_stepping_out() {
        let mut w = WhatIf::new(sample());
        w.step_back();
        w.step_back();
        w.step_back();
        assert_eq!(w.index(), 0);
        // Black's alternate opening: C4 (row 3, col 2).
        assert!(w.play(pos(3, 2)));
        assert!(w.branched());
        assert!(!w.play(pos(0, 0)), "illegal alternate move rejected");
        w.step_forward();
        assert!(!w.branched(), "stepping abandons the branch");
        assert_eq!(w.index(), 1);
    }
}
