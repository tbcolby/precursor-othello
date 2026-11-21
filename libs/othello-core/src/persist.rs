//! Settings and statistics: small versioned byte encodings.

use alloc::vec::Vec;

use crate::{Difficulty, Mode, Outcome};

const SETTINGS_VERSION: u8 = 2;
const STATS_VERSION: u8 = 2;

/// User settings (only the ones the UI actually uses).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    pub show_coordinates: bool,
    pub show_valid_moves: bool,
    pub allow_undo: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { show_coordinates: true, show_valid_moves: true, allow_undo: true }
    }
}

impl Settings {
    pub fn encode(&self) -> Vec<u8> {
        alloc::vec![SETTINGS_VERSION, self.show_coordinates as u8, self.show_valid_moves as u8, self.allow_undo as u8]
    }

    /// None if the bytes are not a version we understand (callers keep the
    /// stored data and use defaults read-only).
    pub fn decode(d: &[u8]) -> Option<Self> {
        if d.len() < 4 || d[0] != SETTINGS_VERSION {
            return None;
        }
        Some(Self { show_coordinates: d[1] != 0, show_valid_moves: d[2] != 0, allow_undo: d[3] != 0 })
    }
}

/// Win/loss/draw per difficulty plus a two-player game count.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Stats {
    /// [easy, medium, hard, expert] x [wins, losses, draws]
    pub vs_cpu: [[u16; 3]; 4],
    pub two_player: u16,
}

fn level(d: Difficulty) -> usize {
    match d {
        Difficulty::Easy => 0,
        Difficulty::Medium => 1,
        Difficulty::Hard => 2,
        Difficulty::Expert => 3,
    }
}

impl Stats {
    /// Record a finished game. `outcome` is None for two-player games.
    pub fn record(&mut self, mode: Mode, outcome: Option<Outcome>) {
        match (mode, outcome) {
            (Mode::VsCpu(d), Some(o)) => {
                let slot = match o {
                    Outcome::Win => 0,
                    Outcome::Loss => 1,
                    Outcome::Draw => 2,
                };
                let c = &mut self.vs_cpu[level(d)][slot];
                *c = c.saturating_add(1);
            }
            (Mode::TwoPlayer, _) => self.two_player = self.two_player.saturating_add(1),
            (Mode::VsCpu(_), None) => {}
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut v = alloc::vec![STATS_VERSION];
        for row in &self.vs_cpu {
            for c in row {
                v.extend_from_slice(&c.to_le_bytes());
            }
        }
        v.extend_from_slice(&self.two_player.to_le_bytes());
        v
    }

    pub fn decode(d: &[u8]) -> Option<Self> {
        if d.len() < 1 + 26 || d[0] != STATS_VERSION {
            return None;
        }
        let mut s = Stats::default();
        let mut i = 1;
        for row in s.vs_cpu.iter_mut() {
            for c in row.iter_mut() {
                *c = u16::from_le_bytes([d[i], d[i + 1]]);
                i += 2;
            }
        }
        s.two_player = u16::from_le_bytes([d[i], d[i + 1]]);
        Some(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip_and_reject_unknown_version() {
        let s = Settings { show_coordinates: false, show_valid_moves: true, allow_undo: false };
        assert_eq!(Settings::decode(&s.encode()), Some(s));
        assert_eq!(Settings::decode(&[1, 1, 1, 1]), None);
        assert_eq!(Settings::decode(&[2, 1]), None);
    }

    #[test]
    fn stats_record_each_outcome_once() {
        let mut s = Stats::default();
        s.record(Mode::VsCpu(Difficulty::Hard), Some(Outcome::Win));
        s.record(Mode::VsCpu(Difficulty::Hard), Some(Outcome::Loss));
        s.record(Mode::VsCpu(Difficulty::Easy), Some(Outcome::Draw));
        s.record(Mode::TwoPlayer, None);
        assert_eq!(s.vs_cpu[2], [1, 1, 0]);
        assert_eq!(s.vs_cpu[0], [0, 0, 1]);
        assert_eq!(s.two_player, 1);
        assert_eq!(Stats::decode(&s.encode()), Some(s));
        assert_eq!(Stats::decode(&[2, 0]), None);
    }
}
