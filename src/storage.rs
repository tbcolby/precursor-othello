//! PDDB storage for settings, statistics, and saved games

use othello_core::{GameState, Player};
use crate::app::GameMode;
use othello_core::Difficulty;

/// Dictionary name for Othello data
const DICT_SETTINGS: &str = "othello.settings";
const DICT_STATS: &str = "othello.stats";
const DICT_SAVE: &str = "othello.save";

const KEY_SETTINGS: &str = "config";
const KEY_STATS: &str = "stats";
const KEY_GAME: &str = "current";

/// User settings
#[derive(Debug, Clone)]
pub struct Settings {
    pub show_coordinates: bool,
    pub show_valid_moves: bool,
    pub allow_undo: bool,
    pub danger_zones: bool,
    pub flip_animation: bool,
    pub ai_think_animation: bool,
    pub ai_delay: bool,
    pub vibration: bool,
    pub sound: bool,
    pub last_difficulty: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            show_coordinates: false,
            show_valid_moves: true,
            allow_undo: true,
            danger_zones: false,
            flip_animation: true,
            ai_think_animation: true,
            ai_delay: true,
            vibration: true,
            sound: true,
            last_difficulty: 1, // Medium
        }
    }
}

impl Settings {
    /// Serialize to bytes
    pub fn to_bytes(&self) -> [u8; 10] {
        [
            self.show_coordinates as u8,
            self.show_valid_moves as u8,
            self.allow_undo as u8,
            self.danger_zones as u8,
            self.flip_animation as u8,
            self.ai_think_animation as u8,
            self.ai_delay as u8,
            self.vibration as u8,
            self.sound as u8,
            self.last_difficulty,
        ]
    }

    /// Deserialize from bytes
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 10 {
            return None;
        }
        Some(Self {
            show_coordinates: data[0] != 0,
            show_valid_moves: data[1] != 0,
            allow_undo: data[2] != 0,
            danger_zones: data[3] != 0,
            flip_animation: data[4] != 0,
            ai_think_animation: data[5] != 0,
            ai_delay: data[6] != 0,
            vibration: data[7] != 0,
            sound: data[8] != 0,
            last_difficulty: data[9],
        })
    }
}

/// Game statistics
#[derive(Debug, Clone, Default)]
pub struct Statistics {
    pub easy_wins: u16,
    pub easy_losses: u16,
    pub easy_draws: u16,
    pub medium_wins: u16,
    pub medium_losses: u16,
    pub medium_draws: u16,
    pub hard_wins: u16,
    pub hard_losses: u16,
    pub hard_draws: u16,
    pub expert_wins: u16,
    pub expert_losses: u16,
    pub expert_draws: u16,
    pub two_player_games: u16,
}

impl Statistics {
    /// Serialize to bytes
    pub fn to_bytes(&self) -> [u8; 26] {
        let mut bytes = [0u8; 26];
        let values = [
            self.easy_wins,
            self.easy_losses,
            self.easy_draws,
            self.medium_wins,
            self.medium_losses,
            self.medium_draws,
            self.hard_wins,
            self.hard_losses,
            self.hard_draws,
            self.expert_wins,
            self.expert_losses,
            self.expert_draws,
            self.two_player_games,
        ];
        for (i, val) in values.iter().enumerate() {
            bytes[i * 2] = (*val & 0xFF) as u8;
            bytes[i * 2 + 1] = ((*val >> 8) & 0xFF) as u8;
        }
        bytes
    }

    /// Deserialize from bytes
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 26 {
            return None;
        }
        let read_u16 = |i: usize| -> u16 {
            u16::from_le_bytes([data[i * 2], data[i * 2 + 1]])
        };
        Some(Self {
            easy_wins: read_u16(0),
            easy_losses: read_u16(1),
            easy_draws: read_u16(2),
            medium_wins: read_u16(3),
            medium_losses: read_u16(4),
            medium_draws: read_u16(5),
            hard_wins: read_u16(6),
            hard_losses: read_u16(7),
            hard_draws: read_u16(8),
            expert_wins: read_u16(9),
            expert_losses: read_u16(10),
            expert_draws: read_u16(11),
            two_player_games: read_u16(12),
        })
    }
}

/// Load settings from PDDB
pub fn load_settings() -> Option<Settings> {
    #[cfg(target_os = "none")]
    {
        let pddb = pddb::Pddb::new();
        match pddb.get(DICT_SETTINGS, KEY_SETTINGS, None, false, false, None, None::<fn()>) {
            Ok(mut key) => {
                use std::io::Read;
                let mut data = [0u8; 10];
                if key.read_exact(&mut data).is_ok() {
                    return Settings::from_bytes(&data);
                }
            }
            Err(_) => {}
        }
    }
    None
}

/// Save settings to PDDB
pub fn save_settings(settings: &Settings) {
    #[cfg(target_os = "none")]
    {
        let pddb = pddb::Pddb::new();
        match pddb.get(DICT_SETTINGS, KEY_SETTINGS, None, true, true, Some(16), None::<fn()>) {
            Ok(mut key) => {
                use std::io::Write;
                key.write_all(&settings.to_bytes()).ok();
                pddb.sync().ok();
            }
            Err(_) => {}
        }
    }
    let _ = settings;
}

/// Load statistics from PDDB
pub fn load_statistics() -> Option<Statistics> {
    #[cfg(target_os = "none")]
    {
        let pddb = pddb::Pddb::new();
        match pddb.get(DICT_STATS, KEY_STATS, None, false, false, None, None::<fn()>) {
            Ok(mut key) => {
                use std::io::Read;
                let mut data = [0u8; 26];
                if key.read_exact(&mut data).is_ok() {
                    return Statistics::from_bytes(&data);
                }
            }
            Err(_) => {}
        }
    }
    None
}

/// Save statistics to PDDB
pub fn save_statistics(stats: &Statistics) {
    #[cfg(target_os = "none")]
    {
        let pddb = pddb::Pddb::new();
        match pddb.get(DICT_STATS, KEY_STATS, None, true, true, Some(32), None::<fn()>) {
            Ok(mut key) => {
                use std::io::Write;
                key.write_all(&stats.to_bytes()).ok();
                pddb.sync().ok();
            }
            Err(_) => {}
        }
    }
    let _ = stats;
}

/// Check if there's a saved game
pub fn has_saved_game() -> bool {
    #[cfg(target_os = "none")]
    {
        let pddb = pddb::Pddb::new();
        pddb.get(DICT_SAVE, KEY_GAME, None, false, false, None, None::<fn()>).is_ok()
    }
    #[cfg(not(target_os = "none"))]
    {
        false
    }
}

/// Save a game to PDDB
pub fn save_game(game: &GameState, mode: GameMode, player_color: Player) {
    #[cfg(target_os = "none")]
    {
        let pddb = pddb::Pddb::new();
        let board = game.board();

        // Serialize: black(8) + white(8) + current(1) + player_color(1) + mode(1) + move_count(2) + history
        let history = game.history();
        let size = 20 + history.len() * 9;

        match pddb.get(DICT_SAVE, KEY_GAME, None, true, true, Some(size), None::<fn()>) {
            Ok(mut key) => {
                use std::io::Write;
                key.write_all(&board.black.to_le_bytes()).ok();
                key.write_all(&board.white.to_le_bytes()).ok();
                key.write_all(&[match game.current_player() {
                    Player::Black => 0,
                    Player::White => 1,
                }])
                .ok();
                key.write_all(&[match player_color {
                    Player::Black => 0,
                    Player::White => 1,
                }])
                .ok();
                key.write_all(&[match mode {
                    GameMode::VsCpu(Difficulty::Easy) => 0,
                    GameMode::VsCpu(Difficulty::Medium) => 1,
                    GameMode::VsCpu(Difficulty::Hard) => 2,
                    GameMode::VsCpu(Difficulty::Expert) => 3,
                    GameMode::TwoPlayer => 4,
                }])
                .ok();
                key.write_all(&(history.len() as u16).to_le_bytes()).ok();

                for entry in history {
                    key.write_all(&[entry.pos]).ok();
                    key.write_all(&entry.flipped.to_le_bytes()).ok();
                }

                pddb.sync().ok();
            }
            Err(_) => {}
        }
    }
    let _ = (game, mode, player_color);
}

/// Load a saved game from PDDB
pub fn load_game() -> Option<(GameState, GameMode, Player)> {
    #[cfg(target_os = "none")]
    {
        let pddb = pddb::Pddb::new();
        match pddb.get(DICT_SAVE, KEY_GAME, None, false, false, None, None::<fn()>) {
            Ok(mut key) => {
                use std::io::Read;

                let mut header = [0u8; 19];
                if key.read_exact(&mut header).is_err() {
                    return None;
                }

                let black = u64::from_le_bytes(header[0..8].try_into().ok()?);
                let white = u64::from_le_bytes(header[8..16].try_into().ok()?);
                let current = if header[16] == 0 { Player::Black } else { Player::White };
                let player_color = if header[17] == 0 { Player::Black } else { Player::White };
                let mode = match header[18] {
                    0 => GameMode::VsCpu(Difficulty::Easy),
                    1 => GameMode::VsCpu(Difficulty::Medium),
                    2 => GameMode::VsCpu(Difficulty::Hard),
                    3 => GameMode::VsCpu(Difficulty::Expert),
                    _ => GameMode::TwoPlayer,
                };

                let mut count_bytes = [0u8; 2];
                if key.read_exact(&mut count_bytes).is_err() {
                    return None;
                }
                let move_count = u16::from_le_bytes(count_bytes) as usize;

                // Reconstruct game state by replaying moves
                let mut game = GameState::new();

                for _ in 0..move_count {
                    let mut entry = [0u8; 9];
                    if key.read_exact(&mut entry).is_err() {
                        break;
                    }
                    let pos = entry[0];
                    if pos == 255 {
                        game.pass();
                    } else {
                        game.make_move(pos);
                    }
                }

                return Some((game, mode, player_color));
            }
            Err(_) => {}
        }
    }
    None
}

/// Delete saved game
pub fn delete_saved_game() {
    #[cfg(target_os = "none")]
    {
        let pddb = pddb::Pddb::new();
        pddb.delete_key(DICT_SAVE, KEY_GAME, None).ok();
        pddb.sync().ok();
    }
}
