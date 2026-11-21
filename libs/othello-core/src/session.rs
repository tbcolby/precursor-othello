//! One game as the UI sees it: whose turn, automatic passes, undo that lands
//! on the human's turn, resign, and a versioned, validated save encoding.

use alloc::vec::Vec;

use crate::game::PASS;
use crate::{Board, Difficulty, GameResult, GameState, Player, Position};

/// Opponent type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    VsCpu(Difficulty),
    TwoPlayer,
}

/// How a finished game ended for the human (vs-CPU) or as a plain result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Win,
    Loss,
    Draw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayError {
    /// It is the CPU's turn, or the game is over.
    NotYourTurn,
    Illegal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    Truncated,
    Version(u8),
    BadField,
    /// The recorded moves are not a legal game.
    BadMove,
    /// Replaying the moves gives a different board than the one saved.
    BoardMismatch,
}

const SAVE_VERSION: u8 = 2;

#[derive(Debug, Clone)]
pub struct Match {
    game: GameState,
    mode: Mode,
    human: Player,
    resigned: bool,
}

fn diff_index(d: Difficulty) -> u8 {
    match d {
        Difficulty::Easy => 0,
        Difficulty::Medium => 1,
        Difficulty::Hard => 2,
        Difficulty::Expert => 3,
    }
}

fn diff_from(i: u8) -> Option<Difficulty> {
    Some(match i {
        0 => Difficulty::Easy,
        1 => Difficulty::Medium,
        2 => Difficulty::Hard,
        3 => Difficulty::Expert,
        _ => return None,
    })
}

impl Match {
    /// Start a game. `human` is ignored for two-player games.
    pub fn new(mode: Mode, human: Player) -> Self {
        let mut m = Self { game: GameState::new(), mode, human, resigned: false };
        m.settle();
        m
    }

    pub fn game(&self) -> &GameState { &self.game }
    pub fn mode(&self) -> Mode { self.mode }
    pub fn human(&self) -> Player { self.human }
    pub fn resigned(&self) -> bool { self.resigned }

    pub fn is_over(&self) -> bool { self.resigned || self.game.is_game_over() }

    /// Apply forced passes until the side to move has a move or the game ends.
    /// Returns the player who passed first, if anyone did.
    fn settle(&mut self) -> Option<Player> {
        let mut first = None;
        while !self.game.is_game_over() && !self.game.has_moves() {
            let p = self.game.current_player();
            if !self.game.pass() {
                break;
            }
            first.get_or_insert(p);
        }
        first
    }

    /// True when the CPU should be asked for a move now.
    pub fn cpu_to_move(&self) -> bool {
        matches!(self.mode, Mode::VsCpu(_)) && !self.is_over() && self.game.current_player() != self.human
    }

    /// Whether the human may place a disc now.
    pub fn human_to_move(&self) -> bool { !self.is_over() && !self.cpu_to_move() }

    fn place(&mut self, pos: Position) -> Result<Option<Player>, PlayError> {
        if self.is_over() {
            return Err(PlayError::NotYourTurn);
        }
        self.game.make_move(pos).ok_or(PlayError::Illegal)?;
        Ok(self.settle())
    }

    /// A human move. `Ok(Some(p))`: player `p` then had to pass.
    pub fn play(&mut self, pos: Position) -> Result<Option<Player>, PlayError> {
        if !self.human_to_move() {
            return Err(PlayError::NotYourTurn);
        }
        self.place(pos)
    }

    /// The CPU's chosen move.
    pub fn play_cpu(&mut self, pos: Position) -> Result<Option<Player>, PlayError> {
        if !self.cpu_to_move() {
            return Err(PlayError::NotYourTurn);
        }
        self.place(pos)
    }

    /// Undo back to the human's previous turn (vs CPU) or one move (two
    /// player). Forced passes are undone with the move before them. Returns
    /// false if there was nothing to undo.
    pub fn undo(&mut self) -> bool {
        if self.resigned || self.game.move_count() == 0 {
            return false;
        }
        loop {
            let Some(entry) = self.game.undo() else { break };
            if entry.is_pass() {
                continue;
            }
            match self.mode {
                Mode::TwoPlayer => break,
                Mode::VsCpu(_) => {
                    if entry.player == self.human {
                        break;
                    }
                }
            }
        }
        true
    }

    pub fn resign(&mut self) { self.resigned = true; }

    /// Final result of a finished game (resignation has no disc result).
    pub fn result(&self) -> Option<GameResult> {
        if self.resigned { None } else { self.game.result() }
    }

    /// Outcome for the human vs the CPU; None for two-player or unfinished games.
    pub fn outcome(&self) -> Option<Outcome> {
        if !matches!(self.mode, Mode::VsCpu(_)) || !self.is_over() {
            return None;
        }
        if self.resigned {
            return Some(Outcome::Loss);
        }
        Some(match self.game.result()?.winner() {
            Some(w) if w == self.human => Outcome::Win,
            Some(_) => Outcome::Loss,
            None => Outcome::Draw,
        })
    }

    /// Save encoding: version, mode, human colour, move count, moves
    /// (255 = pass), then black and white bitboards for verification.
    pub fn encode(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(8 + self.game.move_count() + 16);
        v.push(SAVE_VERSION);
        v.push(match self.mode {
            Mode::TwoPlayer => 255,
            Mode::VsCpu(d) => diff_index(d),
        });
        v.push(matches!(self.human, Player::White) as u8);
        v.push(self.game.move_count() as u8);
        for e in self.game.history() {
            v.push(e.pos);
        }
        v.extend_from_slice(&self.game.board().black.to_le_bytes());
        v.extend_from_slice(&self.game.board().white.to_le_bytes());
        v
    }

    /// Decode and validate a save: replays every move and compares the board.
    pub fn decode(data: &[u8]) -> Result<Self, DecodeError> {
        let (&ver, rest) = data.split_first().ok_or(DecodeError::Truncated)?;
        if ver != SAVE_VERSION {
            return Err(DecodeError::Version(ver));
        }
        if rest.len() < 3 {
            return Err(DecodeError::Truncated);
        }
        let mode = match rest[0] {
            255 => Mode::TwoPlayer,
            d => Mode::VsCpu(diff_from(d).ok_or(DecodeError::BadField)?),
        };
        let human = match rest[1] {
            0 => Player::Black,
            1 => Player::White,
            _ => return Err(DecodeError::BadField),
        };
        let n = rest[2] as usize;
        let body = &rest[3..];
        if body.len() < n + 16 {
            return Err(DecodeError::Truncated);
        }
        let mut game = GameState::new();
        for &p in &body[..n] {
            let ok = if p == PASS { game.pass() } else { p < 64 && game.make_move(p).is_some() };
            if !ok {
                return Err(DecodeError::BadMove);
            }
        }
        let black = u64::from_le_bytes(body[n..n + 8].try_into().unwrap());
        let white = u64::from_le_bytes(body[n + 8..n + 16].try_into().unwrap());
        if *game.board() != (Board { black, white }) {
            return Err(DecodeError::BoardMismatch);
        }
        let mut m = Self { game, mode, human, resigned: false };
        m.settle();
        Ok(m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pos;

    fn vs(human: Player) -> Match {
        Match::new(Mode::VsCpu(Difficulty::Easy), human)
    }

    #[test]
    fn turns_alternate_and_cpu_flag() {
        let mut m = vs(Player::Black);
        assert!(m.human_to_move() && !m.cpu_to_move());
        assert_eq!(m.play(pos(2, 3)), Ok(None));
        assert!(m.cpu_to_move());
        assert_eq!(m.play(pos(2, 2)), Err(PlayError::NotYourTurn));
        let w = vs(Player::White);
        assert!(w.cpu_to_move(), "CPU is Black and opens");
    }

    #[test]
    fn illegal_move_rejected_without_state_change() {
        let mut m = vs(Player::Black);
        assert_eq!(m.play(pos(0, 0)), Err(PlayError::Illegal));
        assert_eq!(m.game().move_count(), 0);
    }

    // Regression for O-5: undo with the human as White after the CPU opened.
    #[test]
    fn undo_lands_on_humans_turn_or_empties_history() {
        let mut m = vs(Player::White);
        m.play_cpu(pos(2, 3)).unwrap(); // CPU (Black) opens
        assert!(m.human_to_move());
        assert!(m.undo()); // pops the CPU's opening: history empty, CPU to move again
        assert_eq!(m.game().move_count(), 0);
        assert!(m.cpu_to_move());
        assert!(!m.undo());
        // Human replies, CPU replies, undo returns to the human's turn.
        m.play_cpu(pos(2, 3)).unwrap();
        m.play(pos(2, 2)).unwrap();
        m.play_cpu(pos(3, 2)).unwrap_or(None);
        let before = m.game().move_count();
        if m.human_to_move() {
            assert!(m.undo());
            assert!(m.game().move_count() < before);
            assert!(m.human_to_move());
        }
    }

    #[test]
    fn two_player_undo_is_one_move() {
        let mut m = Match::new(Mode::TwoPlayer, Player::Black);
        m.play(pos(2, 3)).unwrap();
        m.play(pos(2, 2)).unwrap();
        assert!(m.undo());
        assert_eq!(m.game().move_count(), 1);
        assert_eq!(m.game().current_player(), Player::White);
    }

    // Regression for O-14: after any pass, passes are re-evaluated.
    #[test]
    fn double_pass_ends_the_game() {
        // Black A1, White B1, Black to... Neither side can move once this resolves.
        let mut game = GameState::from_board(Board { black: 1, white: 2 }, Player::Black);
        // Black to move: C1 flips B1. Then board is full of black on row 1: White has none, Black none.
        assert!(game.make_move(2).is_some());
        let m = Match { game, mode: Mode::TwoPlayer, human: Player::Black, resigned: false };
        let mut m2 = m.clone();
        assert!(m2.settle().is_some());
        assert!(m2.is_over());
    }

    #[test]
    fn resign_is_a_loss_for_the_human_only_vs_cpu() {
        let mut m = vs(Player::Black);
        m.resign();
        assert!(m.is_over());
        assert_eq!(m.outcome(), Some(Outcome::Loss));
        assert!(!m.undo());
        let mut t = Match::new(Mode::TwoPlayer, Player::Black);
        t.resign();
        assert_eq!(t.outcome(), None);
    }

    #[test]
    fn save_round_trip_and_validation() {
        let mut m = vs(Player::Black);
        m.play(pos(2, 3)).unwrap();
        m.play_cpu(pos(2, 2)).unwrap();
        let bytes = m.encode();
        let back = Match::decode(&bytes).unwrap();
        assert_eq!(back.game().board(), m.game().board());
        assert_eq!(back.game().move_count(), 2);
        assert_eq!(back.mode(), m.mode());
        assert_eq!(back.human(), Player::Black);

        // Truncated, wrong version, corrupted move, corrupted board.
        assert_eq!(Match::decode(&bytes[..5]).unwrap_err(), DecodeError::Truncated);
        let mut b = bytes.clone();
        b[0] = 1;
        assert_eq!(Match::decode(&b).unwrap_err(), DecodeError::Version(1));
        let mut b = bytes.clone();
        b[4] = 0; // A1 is not legal as a first move
        assert_eq!(Match::decode(&b).unwrap_err(), DecodeError::BadMove);
        let mut b = bytes.clone();
        let n = b.len();
        b[n - 1] ^= 0x80;
        assert_eq!(Match::decode(&b).unwrap_err(), DecodeError::BoardMismatch);
        let mut b = bytes;
        b[1] = 9;
        assert_eq!(Match::decode(&b).unwrap_err(), DecodeError::BadField);
        assert_eq!(Match::decode(&[]).unwrap_err(), DecodeError::Truncated);
    }

    #[test]
    fn save_with_passes_round_trips() {
        let mut seed = 5u64;
        for _ in 0..30 {
            let mut m = Match::new(Mode::TwoPlayer, Player::Black);
            while !m.is_over() {
                let legal = m.game().legal_moves_bitboard();
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                let p = Board::iter_bits(legal).nth((seed % legal.count_ones() as u64) as usize).unwrap();
                m.play(p).unwrap();
            }
            let back = Match::decode(&m.encode()).unwrap();
            assert_eq!(back.game().board(), m.game().board());
            assert!(back.is_over());
        }
    }
}
