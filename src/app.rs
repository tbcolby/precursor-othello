//! Main application state and logic

use gam::Gid;
use gam::menu::Point;
use othello_core::{GameState, GameResult, Player, Difficulty, find_best_move, pos};

use crate::menu::{Menu, MenuItem, MenuContext};
use crate::storage::{Settings, Statistics};
use crate::ui;
use crate::help::HelpContext;
use crate::AppOp;

/// Game mode (vs CPU or two player)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    VsCpu(Difficulty),
    TwoPlayer,
}

/// Main application state
#[derive(Debug, Clone)]
pub enum AppState {
    /// Main menu
    MainMenu,
    /// New game selection
    NewGameMenu,
    /// Settings screen
    SettingsMenu,
    /// Statistics display
    Statistics,
    /// Active game
    Playing {
        game: GameState,
        mode: GameMode,
        player_color: Player,
        cursor_pos: (u8, u8),
        ai_thinking: bool,
        thinking_dots: u8,
        show_pass_notice: bool,
    },
    /// Game over screen
    GameOver {
        game: GameState,
        mode: GameMode,
        player_color: Player,
    },
    /// What If review mode
    WhatIf {
        base_game: GameState,
        current_game: GameState,
        view_index: usize,
        branched: bool,
        cursor_pos: (u8, u8),
    },
    /// Move history view
    MoveHistory {
        game: GameState,
        scroll_offset: usize,
    },
    /// Help screen
    Help {
        context: HelpContext,
        previous: Box<AppState>,
    },
}

/// Main Othello app
pub struct OthelloApp {
    /// Graphics ID for drawing
    pub gid: Gid,
    /// Screen dimensions
    pub screensize: Point,
    /// Current application state
    pub state: AppState,
    /// Context menu
    pub menu: Menu,
    /// User settings
    pub settings: Settings,
    /// Game statistics
    pub stats: Statistics,
    /// Whether we have a saved game
    pub has_save: bool,
}

impl OthelloApp {
    /// Create a new app
    pub fn new(gid: Gid, screensize: Point) -> Self {
        Self {
            gid,
            screensize,
            state: AppState::MainMenu,
            menu: Menu::new(),
            settings: Settings::default(),
            stats: Statistics::default(),
            has_save: false,
        }
    }

    /// Load settings from PDDB
    pub fn load_settings(&mut self) {
        if let Some(settings) = crate::storage::load_settings() {
            self.settings = settings;
        }
        if let Some(stats) = crate::storage::load_statistics() {
            self.stats = stats;
        }
        self.has_save = crate::storage::has_saved_game();
    }

    /// Save settings to PDDB
    pub fn save_settings(&self) {
        crate::storage::save_settings(&self.settings);
    }

    /// Handle going to background
    pub fn on_background(&mut self) {
        // Pause AI thinking if active
        if let AppState::Playing { ai_thinking, .. } = &mut self.state {
            *ai_thinking = false;
        }
    }

    /// Handle returning to foreground
    pub fn on_foreground(&mut self) {
        // Resume AI if it was their turn
        self.check_ai_turn();
    }

    /// Draw the current state
    pub fn draw(&self, gam: &gam::Gam) {
        ui::draw(self, gam);

        // Draw menu overlay if visible
        if self.menu.visible {
            ui::draw_menu(self, gam);
        }
    }

    /// Handle a key press
    pub fn handle_key(
        &mut self,
        key: char,
        gam: &gam::Gam,
        ticktimer: &ticktimer_server::Ticktimer,
        self_cid: xous::CID,
    ) -> bool {
        // Handle menu if visible
        if self.menu.visible {
            return self.handle_menu_key(key, gam, ticktimer, self_cid);
        }

        // Handle F-keys first
        match key {
            '\u{F001}' | '\u{0091}' => {
                // F1 - Open menu
                self.open_context_menu();
                return true;
            }
            '\u{F004}' | '\u{0094}' => {
                // F4 - Exit/Back
                return self.handle_f4(gam, ticktimer);
            }
            _ => {}
        }

        // State-specific key handling
        match &mut self.state {
            AppState::MainMenu => self.handle_main_menu_key(key),
            AppState::NewGameMenu => self.handle_new_game_menu_key(key, self_cid),
            AppState::SettingsMenu => self.handle_settings_menu_key(key),
            AppState::Statistics => self.handle_statistics_key(key),
            AppState::Playing { .. } => self.handle_playing_key(key, self_cid),
            AppState::GameOver { .. } => self.handle_game_over_key(key, self_cid),
            AppState::WhatIf { .. } => self.handle_what_if_key(key),
            AppState::MoveHistory { .. } => self.handle_history_key(key),
            AppState::Help { .. } => self.handle_help_key(key),
        }
    }

    /// Handle F4 (Exit/Back)
    fn handle_f4(
        &mut self,
        _gam: &gam::Gam,
        _ticktimer: &ticktimer_server::Ticktimer,
    ) -> bool {
        match &self.state {
            AppState::MainMenu => {
                // TODO: Exit app
                false
            }
            AppState::NewGameMenu | AppState::SettingsMenu | AppState::Statistics => {
                self.state = AppState::MainMenu;
                true
            }
            AppState::Playing { game, mode, player_color, .. } => {
                // Save game and go to main menu
                crate::storage::save_game(game, *mode, *player_color);
                self.has_save = true;
                self.state = AppState::MainMenu;
                true
            }
            AppState::GameOver { .. } => {
                self.state = AppState::MainMenu;
                true
            }
            AppState::WhatIf { .. } => {
                // Exit What If mode
                self.state = AppState::MainMenu;
                true
            }
            AppState::MoveHistory { .. } => {
                // Return to previous state (game over or playing)
                self.state = AppState::MainMenu;
                true
            }
            AppState::Help { previous, .. } => {
                self.state = *previous.clone();
                true
            }
        }
    }

    /// Open the context menu for current state
    fn open_context_menu(&mut self) {
        let context = match &self.state {
            AppState::MainMenu => MenuContext::MainMenu { has_save: self.has_save },
            AppState::Playing { .. } => MenuContext::Playing,
            AppState::GameOver { .. } => MenuContext::GameOver,
            AppState::WhatIf { .. } => MenuContext::WhatIf,
            _ => return, // No menu for other states
        };
        self.menu.open(context);
    }

    /// Handle key in menu
    fn handle_menu_key(
        &mut self,
        key: char,
        _gam: &gam::Gam,
        _ticktimer: &ticktimer_server::Ticktimer,
        self_cid: xous::CID,
    ) -> bool {
        match key {
            '\u{F004}' | '\u{0094}' | '\u{001B}' => {
                // F4 or Esc - Close menu
                self.menu.close();
                true
            }
            '↑' | '\u{2191}' => {
                self.menu.up();
                true
            }
            '↓' | '\u{2193}' => {
                self.menu.down();
                true
            }
            '\r' | '\n' => {
                // Select
                if let Some(item) = self.menu.select() {
                    self.handle_menu_action(item, self_cid);
                }
                true
            }
            _ => false,
        }
    }

    /// Handle a menu action
    fn handle_menu_action(&mut self, item: MenuItem, self_cid: xous::CID) {
        self.menu.close();

        match item {
            MenuItem::Help => {
                let context = match &self.state {
                    AppState::Playing { .. } => HelpContext::Playing,
                    AppState::WhatIf { .. } => HelpContext::WhatIf,
                    _ => HelpContext::MainMenu,
                };
                let previous = Box::new(self.state.clone());
                self.state = AppState::Help { context, previous };
            }
            MenuItem::NewGame => {
                self.state = AppState::NewGameMenu;
            }
            MenuItem::Resume => {
                if let Some((game, mode, player_color)) = crate::storage::load_game() {
                    self.state = AppState::Playing {
                        game,
                        mode,
                        player_color,
                        cursor_pos: (3, 3),
                        ai_thinking: false,
                        thinking_dots: 0,
                        show_pass_notice: false,
                    };
                    self.check_ai_turn();
                }
            }
            MenuItem::Statistics => {
                self.state = AppState::Statistics;
            }
            MenuItem::Settings => {
                self.state = AppState::SettingsMenu;
            }
            MenuItem::MoveHistory => {
                if let AppState::Playing { game, .. } | AppState::GameOver { game, .. } = &self.state {
                    self.state = AppState::MoveHistory {
                        game: game.clone(),
                        scroll_offset: 0,
                    };
                }
            }
            MenuItem::Hint => {
                if let AppState::Playing { game, cursor_pos, .. } = &mut self.state {
                    if let Some(pos) = othello_core::get_hint(game.board(), game.current_player()) {
                        let (row, col) = othello_core::pos_to_rc(pos);
                        *cursor_pos = (row, col);
                    }
                }
            }
            MenuItem::Undo => {
                if let AppState::Playing { game, .. } = &mut self.state {
                    game.undo();
                    // In vs CPU mode, undo opponent's move too
                    game.undo();
                }
            }
            MenuItem::Resign => {
                // Extract values before mutating
                let data = if let AppState::Playing { game, mode, player_color, .. } = &self.state {
                    Some((game.clone(), *mode, *player_color))
                } else {
                    None
                };
                if let Some((game_clone, mode_copy, player_copy)) = data {
                    // Record loss and go to game over
                    self.update_stats_loss(mode_copy);
                    self.state = AppState::GameOver {
                        game: game_clone,
                        mode: mode_copy,
                        player_color: player_copy,
                    };
                }
            }
            MenuItem::SaveAndExit => {
                if let AppState::Playing { game, mode, player_color, .. } = &self.state {
                    crate::storage::save_game(game, *mode, *player_color);
                    self.has_save = true;
                    self.state = AppState::MainMenu;
                }
            }
            MenuItem::WhatIf => {
                if let AppState::GameOver { game, .. } = &self.state {
                    self.state = AppState::WhatIf {
                        base_game: game.clone(),
                        current_game: game.clone(),
                        view_index: game.move_count(),
                        branched: false,
                        cursor_pos: (3, 3),
                    };
                }
            }
            MenuItem::ExitWhatIf => {
                self.state = AppState::MainMenu;
            }
            MenuItem::MainMenu => {
                self.state = AppState::MainMenu;
            }
        }
    }

    /// Handle key in main menu
    fn handle_main_menu_key(&mut self, key: char) -> bool {
        match key {
            '↑' | '\u{2191}' | '↓' | '\u{2193}' | '\r' | '\n' => {
                // Main menu uses F1 menu system
                self.open_context_menu();
                true
            }
            'n' | 'N' => {
                self.state = AppState::NewGameMenu;
                true
            }
            's' | 'S' => {
                self.state = AppState::SettingsMenu;
                true
            }
            _ => false,
        }
    }

    /// Handle key in new game menu
    fn handle_new_game_menu_key(&mut self, key: char, self_cid: xous::CID) -> bool {
        match key {
            '1' => {
                self.start_game(GameMode::VsCpu(Difficulty::Easy), self_cid);
                true
            }
            '2' => {
                self.start_game(GameMode::VsCpu(Difficulty::Medium), self_cid);
                true
            }
            '3' => {
                self.start_game(GameMode::VsCpu(Difficulty::Hard), self_cid);
                true
            }
            '4' => {
                self.start_game(GameMode::VsCpu(Difficulty::Expert), self_cid);
                true
            }
            '5' | 't' | 'T' => {
                self.start_game(GameMode::TwoPlayer, self_cid);
                true
            }
            _ => false,
        }
    }

    /// Start a new game
    fn start_game(&mut self, mode: GameMode, _self_cid: xous::CID) {
        let game = GameState::new();

        // Random player color for vs CPU
        let player_color = match mode {
            GameMode::VsCpu(_) => {
                // Use hardware TRNG
                if crate::feedback::random_bit() {
                    Player::Black
                } else {
                    Player::White
                }
            }
            GameMode::TwoPlayer => Player::Black, // Not used in two-player
        };

        self.state = AppState::Playing {
            game,
            mode,
            player_color,
            cursor_pos: (3, 3),
            ai_thinking: false,
            thinking_dots: 0,
            show_pass_notice: false,
        };

        // Start AI if it goes first
        self.check_ai_turn();
    }

    /// Check if it's the AI's turn and start thinking
    fn check_ai_turn(&mut self) {
        if let AppState::Playing { game, mode, player_color, ai_thinking, .. } = &mut self.state {
            if let GameMode::VsCpu(_) = mode {
                if game.current_player() != *player_color && !game.is_game_over() {
                    *ai_thinking = true;
                }
            }
        }
    }

    /// Handle key while playing
    fn handle_playing_key(&mut self, key: char, _self_cid: xous::CID) -> bool {
        // Get mutable access to playing state
        let (game, mode, player_color, cursor_pos, ai_thinking, show_pass_notice) = match &mut self.state {
            AppState::Playing {
                game,
                mode,
                player_color,
                cursor_pos,
                ai_thinking,
                show_pass_notice,
                ..
            } => (game, mode, player_color, cursor_pos, ai_thinking, show_pass_notice),
            _ => return false,
        };

        // If AI is thinking, ignore most keys
        if *ai_thinking {
            return false;
        }

        // If showing pass notice, any key dismisses
        if *show_pass_notice {
            *show_pass_notice = false;
            return true;
        }

        match key {
            // Arrow keys for cursor movement
            '↑' | '\u{2191}' => {
                if cursor_pos.0 > 0 {
                    cursor_pos.0 -= 1;
                }
                true
            }
            '↓' | '\u{2193}' => {
                if cursor_pos.0 < 7 {
                    cursor_pos.0 += 1;
                }
                true
            }
            '←' | '\u{2190}' => {
                if cursor_pos.1 > 0 {
                    cursor_pos.1 -= 1;
                }
                true
            }
            '→' | '\u{2192}' => {
                if cursor_pos.1 < 7 {
                    cursor_pos.1 += 1;
                }
                true
            }
            // Enter to place disc
            '\r' | '\n' => {
                let position = pos(cursor_pos.0, cursor_pos.1);
                if game.is_legal(position) {
                    game.make_move(position);
                    crate::feedback::vibrate_move();

                    // Check for game over
                    if game.is_game_over() {
                        self.handle_game_over();
                        return true;
                    }

                    // Check if opponent must pass
                    if !game.has_moves() {
                        game.pass();
                        *show_pass_notice = true;

                        // Check if now we must pass (game over)
                        if !game.has_moves() {
                            game.pass();
                            if game.is_game_over() {
                                self.handle_game_over();
                                return true;
                            }
                        }
                    }

                    // Start AI thinking
                    self.check_ai_turn();
                    true
                } else {
                    crate::feedback::vibrate_invalid();
                    false
                }
            }
            // F2 for hint
            '\u{F002}' | '\u{0092}' => {
                if let Some(pos) = othello_core::get_hint(game.board(), game.current_player()) {
                    let (row, col) = othello_core::pos_to_rc(pos);
                    *cursor_pos = (row, col);
                }
                true
            }
            // U for undo
            'u' | 'U' => {
                if self.settings.allow_undo {
                    game.undo();
                    if matches!(mode, GameMode::VsCpu(_)) {
                        game.undo(); // Undo AI move too
                    }
                }
                true
            }
            _ => false,
        }
    }

    /// Handle game over transition
    fn handle_game_over(&mut self) {
        // Extract values before mutating
        let data = if let AppState::Playing { game, mode, player_color, .. } = &self.state {
            let result = game.result();
            let winner = result.as_ref().and_then(|r| r.winner());
            Some((game.clone(), *mode, *player_color, winner))
        } else {
            None
        };

        if let Some((game_clone, mode_copy, player_color_copy, winner)) = data {
            // Update statistics
            match mode_copy {
                GameMode::VsCpu(_) => {
                    match winner {
                        Some(w) if w == player_color_copy => {
                            self.update_stats_win(mode_copy);
                        }
                        Some(_) => {
                            self.update_stats_loss(mode_copy);
                        }
                        None => {
                            self.update_stats_draw(mode_copy);
                        }
                    }
                }
                GameMode::TwoPlayer => {
                    self.stats.two_player_games += 1;
                }
            }

            crate::storage::save_statistics(&self.stats);
            crate::feedback::vibrate_game_over();

            // Clear saved game
            crate::storage::delete_saved_game();
            self.has_save = false;

            self.state = AppState::GameOver {
                game: game_clone,
                mode: mode_copy,
                player_color: player_color_copy,
            };
        }
    }

    /// Update stats for a win
    fn update_stats_win(&mut self, mode: GameMode) {
        match mode {
            GameMode::VsCpu(Difficulty::Easy) => self.stats.easy_wins += 1,
            GameMode::VsCpu(Difficulty::Medium) => self.stats.medium_wins += 1,
            GameMode::VsCpu(Difficulty::Hard) => self.stats.hard_wins += 1,
            GameMode::VsCpu(Difficulty::Expert) => self.stats.expert_wins += 1,
            GameMode::TwoPlayer => self.stats.two_player_games += 1,
        }
    }

    /// Update stats for a loss
    fn update_stats_loss(&mut self, mode: GameMode) {
        match mode {
            GameMode::VsCpu(Difficulty::Easy) => self.stats.easy_losses += 1,
            GameMode::VsCpu(Difficulty::Medium) => self.stats.medium_losses += 1,
            GameMode::VsCpu(Difficulty::Hard) => self.stats.hard_losses += 1,
            GameMode::VsCpu(Difficulty::Expert) => self.stats.expert_losses += 1,
            GameMode::TwoPlayer => self.stats.two_player_games += 1,
        }
    }

    /// Update stats for a draw
    fn update_stats_draw(&mut self, mode: GameMode) {
        match mode {
            GameMode::VsCpu(Difficulty::Easy) => self.stats.easy_draws += 1,
            GameMode::VsCpu(Difficulty::Medium) => self.stats.medium_draws += 1,
            GameMode::VsCpu(Difficulty::Hard) => self.stats.hard_draws += 1,
            GameMode::VsCpu(Difficulty::Expert) => self.stats.expert_draws += 1,
            GameMode::TwoPlayer => self.stats.two_player_games += 1,
        }
    }

    /// Handle key in game over state
    fn handle_game_over_key(&mut self, key: char, self_cid: xous::CID) -> bool {
        match key {
            '\r' | '\n' => {
                // New game with same mode
                if let AppState::GameOver { mode, .. } = self.state {
                    self.start_game(mode, self_cid);
                }
                true
            }
            'w' | 'W' => {
                // Enter What If mode
                if let AppState::GameOver { game, .. } = &self.state {
                    self.state = AppState::WhatIf {
                        base_game: game.clone(),
                        current_game: game.clone(),
                        view_index: game.move_count(),
                        branched: false,
                        cursor_pos: (3, 3),
                    };
                }
                true
            }
            'n' | 'N' => {
                self.state = AppState::NewGameMenu;
                true
            }
            _ => false,
        }
    }

    /// Handle key in What If mode
    fn handle_what_if_key(&mut self, key: char) -> bool {
        let (base_game, current_game, view_index, branched, cursor_pos) = match &mut self.state {
            AppState::WhatIf {
                base_game,
                current_game,
                view_index,
                branched,
                cursor_pos,
            } => (base_game, current_game, view_index, branched, cursor_pos),
            _ => return false,
        };

        match key {
            // Step back in history
            '←' | '\u{2190}' => {
                if *view_index > 0 {
                    *view_index -= 1;
                    *current_game = base_game.clone_at_move(*view_index);
                }
                true
            }
            // Step forward in history
            '→' | '\u{2192}' => {
                if *view_index < base_game.move_count() {
                    *view_index += 1;
                    *current_game = base_game.clone_at_move(*view_index);
                }
                true
            }
            // Cursor movement
            '↑' | '\u{2191}' => {
                if cursor_pos.0 > 0 {
                    cursor_pos.0 -= 1;
                }
                true
            }
            '↓' | '\u{2193}' => {
                if cursor_pos.0 < 7 {
                    cursor_pos.0 += 1;
                }
                true
            }
            // Play alternate move (branch)
            '\r' | '\n' => {
                let position = pos(cursor_pos.0, cursor_pos.1);
                if current_game.is_legal(position) {
                    current_game.make_move(position);
                    *branched = true;
                }
                true
            }
            _ => false,
        }
    }

    /// Handle key in history view
    fn handle_history_key(&mut self, key: char) -> bool {
        let scroll_offset = match &mut self.state {
            AppState::MoveHistory { scroll_offset, .. } => scroll_offset,
            _ => return false,
        };

        match key {
            '↑' | '\u{2191}' => {
                if *scroll_offset > 0 {
                    *scroll_offset -= 1;
                }
                true
            }
            '↓' | '\u{2193}' => {
                *scroll_offset += 1;
                true
            }
            _ => false,
        }
    }

    /// Handle key in settings
    fn handle_settings_menu_key(&mut self, key: char) -> bool {
        match key {
            '1' => {
                self.settings.show_coordinates = !self.settings.show_coordinates;
                self.save_settings();
                true
            }
            '2' => {
                self.settings.show_valid_moves = !self.settings.show_valid_moves;
                self.save_settings();
                true
            }
            '3' => {
                self.settings.allow_undo = !self.settings.allow_undo;
                self.save_settings();
                true
            }
            '4' => {
                self.settings.vibration = !self.settings.vibration;
                self.save_settings();
                true
            }
            _ => false,
        }
    }

    /// Handle key in statistics view
    fn handle_statistics_key(&mut self, _key: char) -> bool {
        false
    }

    /// Handle key in help screen
    fn handle_help_key(&mut self, _key: char) -> bool {
        // Any key dismisses help
        if let AppState::Help { previous, .. } = &self.state {
            self.state = *previous.clone();
            return true;
        }
        false
    }

    /// AI thinking tick
    pub fn ai_tick(
        &mut self,
        _gam: &gam::Gam,
        ticktimer: &ticktimer_server::Ticktimer,
    ) {
        if let AppState::Playing {
            game,
            mode: GameMode::VsCpu(difficulty),
            ai_thinking,
            thinking_dots,
            show_pass_notice,
            ..
        } = &mut self.state
        {
            if *ai_thinking {
                // Animate thinking dots
                *thinking_dots = (*thinking_dots + 1) % 4;

                // Add delay if enabled
                if self.settings.ai_delay {
                    ticktimer.sleep_ms(100).ok();
                }

                // Actually compute AI move
                if let Some(pos) = find_best_move(game.board(), game.current_player(), *difficulty) {
                    game.make_move(pos);
                    *ai_thinking = false;

                    // Check for game over
                    if game.is_game_over() {
                        self.handle_game_over();
                        return;
                    }

                    // Check if player must pass
                    if !game.has_moves() {
                        game.pass();
                        *show_pass_notice = true;

                        // Check if AI must also pass (game over)
                        if !game.has_moves() {
                            game.pass();
                            if game.is_game_over() {
                                self.handle_game_over();
                            }
                        } else {
                            // AI's turn again
                            *ai_thinking = true;
                        }
                    }

                    crate::feedback::vibrate_move();
                } else {
                    // AI must pass
                    game.pass();
                    *ai_thinking = false;

                    if game.is_game_over() {
                        self.handle_game_over();
                    }
                }
            }
        }
    }
}
