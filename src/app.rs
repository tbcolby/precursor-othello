//! Screens, input and persistence for Othello. Rules, AI, match flow and save
//! formats are in `othello-core`; this file maps keys to those operations,
//! stores data, and draws.

use gam::GlyphStyle;
use othello_core::{
    pos_to_algebraic, Difficulty, Match, Mode, Outcome, Player, PlayError, Settings, Stats, WhatIf,
};
use precursor_appkit::keys::Key;
use precursor_appkit::layout::ListState;
use precursor_appkit::rng::Trng;
use precursor_appkit::screen::Screen;
use precursor_appkit::store::Store;

use crate::ai::Worker;
use crate::board_view::{self, Geometry};

const KEY_SETTINGS: &str = "settings";
const KEY_STATS: &str = "stats";
const KEY_SAVE: &str = "save";

#[derive(PartialEq, Eq)]
pub enum Action {
    Stay,
    Leave,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum View {
    Main,
    NewGame,
    Settings,
    Stats,
    Help,
    Playing,
    GameMenu,
    GameOver,
    WhatIf,
    History,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Busy {
    Cpu,
    Hint,
}

/// A stored blob that failed to parse is kept as is, and never overwritten.
#[derive(Default)]
struct ReadOnly {
    settings: bool,
    stats: bool,
    save: bool,
}

pub struct App {
    screen: Screen,
    store: Store,
    trng: Trng,
    ai: Worker,
    view: View,
    /// Where Help / History return to.
    return_to: View,
    list: ListState,
    visible_rows: usize,
    settings: Settings,
    stats: Stats,
    ro: ReadOnly,
    has_save: bool,
    m: Option<Match>,
    cursor: (u8, u8),
    busy: Option<Busy>,
    what_if: Option<WhatIf>,
    save_dirty: bool,
    notice: String,
}

const MAIN_ITEMS: [&str; 6] = ["New game", "Resume game", "Statistics", "Settings", "Help", "Quit"];
const LEVELS: [(&str, Mode); 5] = [
    ("1  Easy", Mode::VsCpu(Difficulty::Easy)),
    ("2  Medium", Mode::VsCpu(Difficulty::Medium)),
    ("3  Hard", Mode::VsCpu(Difficulty::Hard)),
    ("4  Expert", Mode::VsCpu(Difficulty::Expert)),
    ("5  Two players", Mode::TwoPlayer),
];
const GAME_MENU: [&str; 7] =
    ["Back to game", "Hint", "Undo", "Move history", "Resign", "Save & exit", "Help"];

const HELP_TEXT: &str = "Outflank your opponent's discs to flip them. \
Most discs when neither side can move wins. A side with no legal move passes automatically.\n\
In the game: arrows move the cursor, Enter places a disc, F1 menu, F2 hint, U undo, \
Backspace/F4 saves and leaves the game. Dots mark legal moves.\n\
Against the CPU your colour is chosen at random; Black moves first. \
After a game, W reviews it and lets you try other moves (F2/F3 step).\n\
The CPU searches on a background thread with a time limit, so higher levels may play \
weaker than their label when the device is slow.";

fn name(p: Player) -> &'static str {
    match p {
        Player::Black => "Black",
        Player::White => "White",
    }
}

fn mode_name(m: Mode) -> &'static str {
    match m {
        Mode::VsCpu(Difficulty::Easy) => "Easy",
        Mode::VsCpu(Difficulty::Medium) => "Medium",
        Mode::VsCpu(Difficulty::Hard) => "Hard",
        Mode::VsCpu(Difficulty::Expert) => "Expert",
        Mode::TwoPlayer => "Two players",
    }
}

fn sq(p: u8) -> String {
    let [c, r] = pos_to_algebraic(p);
    format!("{}{}", c as char, r as char)
}

impl App {
    pub fn new(screen: Screen, store: Store, trng: Trng, ai: Worker) -> Self {
        let mut app = Self {
            screen,
            store,
            trng,
            ai,
            view: View::Main,
            return_to: View::Main,
            list: ListState::default(),
            visible_rows: 6,
            settings: Settings::default(),
            stats: Stats::default(),
            ro: ReadOnly::default(),
            has_save: false,
            m: None,
            cursor: (3, 3),
            busy: None,
            what_if: None,
            save_dirty: false,
            notice: String::new(),
        };
        app.reload();
        app
    }

    pub fn screen(&self) -> &Screen { &self.screen }

    /// (Re)load settings, statistics and the save marker. Runs at start and
    /// whenever the set of open PDDB bases changes. A game in progress is kept.
    pub fn reload(&mut self) {
        self.ro = ReadOnly::default();
        match self.store.read_bytes(KEY_SETTINGS) {
            Ok(Some(b)) => match Settings::decode(&b) {
                Some(s) => self.settings = s,
                None => {
                    self.ro.settings = true;
                    self.settings = Settings::default();
                }
            },
            Ok(None) => self.settings = Settings::default(),
            Err(e) => {
                log::error!("settings read failed: {:?}", e);
                self.ro.settings = true;
            }
        }
        match self.store.read_bytes(KEY_STATS) {
            Ok(Some(b)) => match Stats::decode(&b) {
                Some(s) => self.stats = s,
                None => {
                    self.ro.stats = true;
                    self.stats = Stats::default();
                }
            },
            Ok(None) => self.stats = Stats::default(),
            Err(e) => {
                log::error!("stats read failed: {:?}", e);
                self.ro.stats = true;
            }
        }
        match self.store.read_bytes(KEY_SAVE) {
            Ok(Some(b)) => {
                self.has_save = true;
                if Match::decode(&b).is_err() {
                    self.ro.save = true;
                }
            }
            Ok(None) => self.has_save = false,
            Err(e) => {
                log::error!("save read failed: {:?}", e);
                self.has_save = false;
                self.ro.save = true;
            }
        }
        if self.ro.settings || self.ro.stats || self.ro.save {
            self.notice = "Some saved data is unreadable and was left untouched".into();
        }
        self.draw();
    }

    // ---- persistence -------------------------------------------------------

    fn save_settings(&mut self) {
        if self.ro.settings {
            self.notice = "Settings are read-only (unreadable data)".into();
            return;
        }
        if let Err(e) = self.store.write_bytes(KEY_SETTINGS, &self.settings.encode()) {
            log::error!("settings save failed: {:?}", e);
            self.notice = "Couldn't save settings".into();
        }
    }

    fn save_stats(&mut self) {
        if self.ro.stats {
            return;
        }
        if let Err(e) = self.store.write_bytes(KEY_STATS, &self.stats.encode()) {
            log::error!("stats save failed: {:?}", e);
            self.notice = "Couldn't save statistics".into();
        }
    }

    /// Save the game in progress, only if something changed.
    fn save_game(&mut self) {
        if !self.save_dirty {
            return;
        }
        let Some(m) = self.m.as_ref().filter(|m| !m.is_over()) else { return };
        if self.ro.save {
            self.notice = "Existing save is unreadable; not overwritten".into();
            return;
        }
        match self.store.write_bytes(KEY_SAVE, &m.encode()) {
            Ok(()) => {
                self.save_dirty = false;
                self.has_save = true;
            }
            Err(e) => {
                log::error!("game save failed: {:?}", e);
                self.notice = "Couldn't save the game".into();
            }
        }
    }

    fn delete_save(&mut self) {
        if self.has_save && !self.ro.save {
            if let Err(e) = self.store.delete(KEY_SAVE) {
                log::error!("save delete failed: {:?}", e);
            }
            self.has_save = false;
        }
        self.save_dirty = false;
    }

    // ---- game flow ---------------------------------------------------------

    fn start_game(&mut self, mode: Mode) {
        let human = if self.trng.below(2) == 0 { Player::Black } else { Player::White };
        self.m = Some(Match::new(mode, human));
        self.cursor = (3, 3);
        self.notice.clear();
        self.view = View::Playing;
        self.save_dirty = true;
        self.maybe_start_cpu();
    }

    fn resume_game(&mut self) {
        let loaded = match self.store.read_bytes(KEY_SAVE) {
            Ok(Some(b)) => Match::decode(&b).map_err(|e| format!("Saved game unreadable ({e:?})")),
            Ok(None) => Err("No saved game".to_string()),
            Err(_) => Err("Couldn't read the saved game".to_string()),
        };
        match loaded {
            Ok(m) if !m.is_over() => {
                self.m = Some(m);
                self.cursor = (3, 3);
                self.notice.clear();
                self.view = View::Playing;
                self.save_dirty = false;
                self.maybe_start_cpu();
            }
            Ok(_) => {
                self.delete_save();
                self.notice = "Saved game was already finished".into();
            }
            Err(msg) => {
                self.ro.save = true;
                self.notice = msg;
            }
        }
    }

    fn maybe_start_cpu(&mut self) {
        let Some(m) = self.m.as_ref() else { return };
        if m.cpu_to_move() {
            if let Mode::VsCpu(d) = m.mode() {
                self.busy = Some(Busy::Cpu);
                self.ai.start(*m.game().board(), m.game().current_player(), d, false);
            }
        }
    }

    fn start_hint(&mut self) {
        let Some(m) = self.m.as_ref() else { return };
        if self.busy.is_none() && m.human_to_move() {
            self.busy = Some(Busy::Hint);
            self.ai.start(*m.game().board(), m.game().current_player(), Difficulty::Hard, true);
        }
    }

    /// Worker result: `pos` is a square, or 255 for "no move".
    pub fn on_ai_done(&mut self, pos: usize, generation: usize, hint: bool) {
        if !self.ai.is_current(generation) {
            return;
        }
        let busy = self.busy.take();
        if self.view != View::Playing {
            return;
        }
        match (busy, hint) {
            (Some(Busy::Hint), true) => {
                if pos < 64 {
                    self.cursor = ((pos / 8) as u8, (pos % 8) as u8);
                    self.notice = format!("Hint: {}", sq(pos as u8));
                }
            }
            (Some(Busy::Cpu), false) => {
                let Some(m) = self.m.as_mut() else { return };
                // The worker only fails to answer if it couldn't start; fall back to any legal move.
                let choice = if pos < 64 {
                    pos as u8
                } else {
                    m.game().legal_moves_bitboard().trailing_zeros() as u8
                };
                match m.play_cpu(choice) {
                    Ok(passed) => self.after_move(passed),
                    Err(e) => {
                        log::error!("CPU move rejected: {:?}", e);
                        self.notice = "CPU move failed".into();
                    }
                }
            }
            _ => {}
        }
    }

    fn after_move(&mut self, passed: Option<Player>) {
        self.save_dirty = true;
        self.notice = passed.map(|p| format!("{} has no move and passes", name(p))).unwrap_or_default();
        if self.m.as_ref().is_some_and(|m| m.is_over()) {
            self.finish_game();
        } else {
            self.maybe_start_cpu();
        }
    }

    /// One path for every way a game ends (last move, double pass, resign).
    fn finish_game(&mut self) {
        self.ai.cancel();
        self.busy = None;
        let Some(m) = self.m.as_ref() else { return };
        self.stats.record(m.mode(), m.outcome());
        self.save_stats();
        self.delete_save();
        self.view = View::GameOver;
    }

    fn leave_game_to_main(&mut self) {
        self.ai.cancel();
        self.busy = None;
        self.save_game();
        self.view = View::Main;
        self.list = ListState::default();
    }

    // ---- lifecycle ---------------------------------------------------------

    pub fn on_foreground(&mut self) {
        self.screen.refresh_size();
        if self.view == View::Playing {
            self.maybe_start_cpu();
        }
        self.draw();
    }

    /// Stop the search, save the game if it changed.
    pub fn on_background(&mut self) {
        self.ai.cancel();
        self.busy = None;
        self.save_game();
    }

    // ---- input -------------------------------------------------------------

    fn main_items(&self) -> Vec<&'static str> {
        MAIN_ITEMS.iter().copied().filter(|s| *s != "Resume game" || self.has_save).collect()
    }

    pub fn on_key(&mut self, key: Key) -> Action {
        if !matches!(self.view, View::Playing) {
            self.notice.clear();
        }
        match self.view {
            View::Main => return self.key_main(key),
            View::NewGame => self.key_new_game(key),
            View::Settings => self.key_settings(key),
            View::Stats | View::Help => {
                if key.is_back() || key.is_select() {
                    self.view = if self.view == View::Help { self.return_to } else { View::Main };
                }
            }
            View::Playing => self.key_playing(key),
            View::GameMenu => self.key_game_menu(key),
            View::GameOver => self.key_game_over(key),
            View::WhatIf => self.key_what_if(key),
            View::History => self.key_history(key),
        }
        Action::Stay
    }

    fn move_list(&mut self, key: Key, len: usize) {
        match key {
            Key::Up => self.list.step(-1, len, self.visible_rows),
            Key::Down => self.list.step(1, len, self.visible_rows),
            _ => {}
        }
    }

    fn key_main(&mut self, key: Key) -> Action {
        let items = self.main_items();
        self.move_list(key, items.len());
        if key.is_back() {
            return Action::Leave;
        }
        if key.is_select() {
            match items[self.list.selected.min(items.len() - 1)] {
                "New game" => {
                    self.view = View::NewGame;
                    self.list = ListState::default();
                }
                "Resume game" => self.resume_game(),
                "Statistics" => self.view = View::Stats,
                "Settings" => {
                    self.view = View::Settings;
                    self.list = ListState::default();
                }
                "Help" => {
                    self.return_to = View::Main;
                    self.view = View::Help;
                }
                _ => return Action::Leave,
            }
        }
        Action::Stay
    }

    fn key_new_game(&mut self, key: Key) {
        self.move_list(key, LEVELS.len());
        if key.is_back() {
            self.view = View::Main;
            self.list = ListState::default();
        } else if key.is_select() {
            self.start_game(LEVELS[self.list.selected].1);
        } else if let Key::Char(c @ '1'..='5') = key {
            self.start_game(LEVELS[c as usize - '1' as usize].1);
        }
    }

    fn key_settings(&mut self, key: Key) {
        self.move_list(key, 3);
        if key.is_back() {
            self.view = View::Main;
            self.list = ListState::default();
        } else if key.is_select() || matches!(key, Key::Left | Key::Right) {
            match self.list.selected {
                0 => self.settings.show_coordinates = !self.settings.show_coordinates,
                1 => self.settings.show_valid_moves = !self.settings.show_valid_moves,
                _ => self.settings.allow_undo = !self.settings.allow_undo,
            }
            self.save_settings();
        }
    }

    fn key_playing(&mut self, key: Key) {
        // Cursor movement is always allowed, even while the CPU thinks.
        match key {
            Key::Up => self.cursor.0 = self.cursor.0.saturating_sub(1),
            Key::Down => self.cursor.0 = (self.cursor.0 + 1).min(7),
            Key::Left => self.cursor.1 = self.cursor.1.saturating_sub(1),
            Key::Right => self.cursor.1 = (self.cursor.1 + 1).min(7),
            k if k.is_back() => self.leave_game_to_main(),
            _ if self.busy == Some(Busy::Cpu) => {}
            Key::F(1) => {
                self.view = View::GameMenu;
                self.list = ListState::default();
            }
            Key::F(2) => {
                self.notice.clear();
                self.start_hint();
            }
            Key::Char('u') | Key::Char('U') => self.undo(),
            Key::Enter => self.place(),
            _ => {}
        }
    }

    fn place(&mut self) {
        self.notice.clear();
        if self.busy.is_some() {
            return;
        }
        let Some(m) = self.m.as_mut() else { return };
        let pos = self.cursor.0 * 8 + self.cursor.1;
        match m.play(pos) {
            Ok(passed) => self.after_move(passed),
            Err(PlayError::Illegal) => self.notice = "Not a legal move".into(),
            Err(PlayError::NotYourTurn) => {}
        }
    }

    fn undo(&mut self) {
        if !self.settings.allow_undo {
            self.notice = "Undo is turned off in Settings".into();
            return;
        }
        let Some(m) = self.m.as_mut() else { return };
        self.ai.cancel();
        self.busy = None;
        if m.undo() {
            self.save_dirty = true;
            self.notice = "Undone".into();
            self.maybe_start_cpu(); // human is White and the CPU's opening was undone
        } else {
            self.notice = "Nothing to undo".into();
        }
    }

    fn key_game_menu(&mut self, key: Key) {
        self.move_list(key, GAME_MENU.len());
        if key.is_back() {
            self.view = View::Playing;
        } else if key.is_select() {
            self.view = View::Playing;
            match self.list.selected {
                1 => self.start_hint(),
                2 => self.undo(),
                3 => {
                    self.return_to = View::Playing;
                    self.view = View::History;
                    self.list = ListState::default();
                }
                4 => {
                    if let Some(m) = self.m.as_mut() {
                        m.resign();
                    }
                    self.finish_game();
                }
                5 => self.leave_game_to_main(),
                6 => {
                    self.return_to = View::Playing;
                    self.view = View::Help;
                }
                _ => {}
            }
        }
    }

    fn key_game_over(&mut self, key: Key) {
        match key {
            k if k.is_back() => {
                self.view = View::Main;
                self.list = ListState::default();
            }
            Key::Enter => {
                if let Some(mode) = self.m.as_ref().map(|m| m.mode()) {
                    self.start_game(mode);
                }
            }
            Key::Char('n') | Key::Char('N') => {
                self.view = View::NewGame;
                self.list = ListState::default();
            }
            Key::Char('w') | Key::Char('W') => {
                if let Some(m) = self.m.as_ref() {
                    self.what_if = Some(WhatIf::new(m.game().clone()));
                    self.cursor = (3, 3);
                    self.view = View::WhatIf;
                }
            }
            Key::Char('h') | Key::Char('H') => {
                self.return_to = View::GameOver;
                self.view = View::History;
                self.list = ListState::default();
            }
            _ => {}
        }
    }

    fn key_what_if(&mut self, key: Key) {
        let Some(w) = self.what_if.as_mut() else {
            self.view = View::GameOver;
            return;
        };
        match key {
            Key::Up => self.cursor.0 = self.cursor.0.saturating_sub(1),
            Key::Down => self.cursor.0 = (self.cursor.0 + 1).min(7),
            Key::Left => self.cursor.1 = self.cursor.1.saturating_sub(1),
            Key::Right => self.cursor.1 = (self.cursor.1 + 1).min(7),
            Key::F(2) | Key::Char(',') => w.step_back(),
            Key::F(3) | Key::Char('.') => w.step_forward(),
            Key::Enter => {
                if !w.play(self.cursor.0 * 8 + self.cursor.1) {
                    self.notice = "Not a legal move".into();
                }
            }
            k if k.is_back() => {
                self.what_if = None;
                self.view = View::GameOver;
            }
            _ => {}
        }
    }

    fn history_lines(&self) -> Vec<String> {
        let Some(m) = self.m.as_ref() else { return Vec::new() };
        let mut lines = Vec::new();
        let mut n = 0;
        for e in m.game().history() {
            n += 1;
            let mv = if e.is_pass() { "pass".to_string() } else { sq(e.pos) };
            lines.push(format!("{:>2}. {} {}", n, name(e.player), mv));
        }
        if lines.is_empty() {
            lines.push("No moves yet".into());
        }
        lines
    }

    fn key_history(&mut self, key: Key) {
        let len = self.history_lines().len();
        self.move_list(key, len);
        if key.is_back() {
            self.view = self.return_to;
        }
    }

    // ---- drawing -----------------------------------------------------------

    pub fn draw(&mut self) {
        self.screen.clear();
        match self.view {
            View::Main => self.draw_list_screen("Othello", "↑↓ select · ⏎ open · ⌫ quit", self.main_items()),
            View::NewGame => self.draw_list_screen(
                "New game",
                "↑↓ select · ⏎ start · ⌫ back",
                LEVELS.iter().map(|l| l.0).collect(),
            ),
            View::Settings => {
                let on = |b: bool| if b { "on" } else { "off" };
                let items = vec![
                    format!("Coordinates: {}", on(self.settings.show_coordinates)),
                    format!("Show legal moves: {}", on(self.settings.show_valid_moves)),
                    format!("Allow undo: {}", on(self.settings.allow_undo)),
                ];
                self.draw_list_screen("Settings", "↑↓ select · ⏎ toggle · ⌫ back", items);
            }
            View::Stats => self.draw_stats(),
            View::Help => {
                let top = self.screen.header("Help");
                let bottom = self.screen.footer("⌫ back");
                self.screen.para(top, bottom, GlyphStyle::Small, HELP_TEXT);
            }
            View::Playing => self.draw_game(false),
            View::GameMenu => self.draw_list_screen("Game menu", "↑↓ select · ⏎ choose · ⌫ back", GAME_MENU.to_vec()),
            View::GameOver => self.draw_game(false),
            View::WhatIf => self.draw_game(true),
            View::History => {
                let lines = self.history_lines();
                self.draw_list_screen("Move history", "↑↓ scroll · ⌫ back", lines);
            }
        }
        if !self.notice.is_empty() && !matches!(self.view, View::Playing | View::GameOver | View::WhatIf) {
            let h = self.screen.line_height(GlyphStyle::Small);
            let y = self.screen.size().y - 2 * h - 8;
            self.screen.text(y, GlyphStyle::Small, &self.notice);
        }
        self.screen.flush();
    }

    fn draw_list_screen<S: AsRef<str>>(&mut self, title: &str, hint: &str, items: Vec<S>) {
        let top = self.screen.header(title);
        let bottom = self.screen.footer(hint);
        let reserve = if self.notice.is_empty() { 0 } else { self.screen.line_height(GlyphStyle::Small) + 4 };
        self.visible_rows = self.screen.list(top, bottom - reserve, &items, &self.list);
    }

    fn draw_stats(&mut self) {
        let top = self.screen.header("Statistics");
        let bottom = self.screen.footer("⌫ back");
        let mut text = String::new();
        for (i, d) in ["Easy", "Medium", "Hard", "Expert"].iter().enumerate() {
            let [w, l, dr] = self.stats.vs_cpu[i];
            text.push_str(&format!("{d}: {w} won, {l} lost, {dr} drawn\n"));
        }
        text.push_str(&format!("Two-player games: {}\n", self.stats.two_player));
        text.push_str("Resigning counts as a loss.");
        self.screen.para(top, bottom, GlyphStyle::Regular, &text);
    }

    fn draw_game(&mut self, what_if: bool) {
        let top = self.screen.header("Othello");
        let hint = match (self.view, what_if) {
            (_, true) => "arrows cursor · ⏎ play · F2/F3 step · ⌫ back",
            (View::GameOver, _) => "⏎ again · N new · W what-if · H moves · ⌫ menu",
            _ => "arrows · ⏎ place · F1 menu · F2 hint · U undo · ⌫ leave",
        };
        let bottom = self.screen.footer(hint);
        let line_h = self.screen.line_height(GlyphStyle::Small);
        let status_h = 3 * line_h;
        let w = self.screen.size().x;
        let geo = Geometry::fit(w, top, bottom - top - status_h, self.settings.show_coordinates);
        let (game, mode_label) = if what_if {
            match self.what_if.as_ref() {
                Some(wi) => (wi.game().clone(), format!("What if: move {}/{}{}", wi.index(), wi.total(), if wi.branched() { " (branch)" } else { "" })),
                None => return,
            }
        } else {
            match self.m.as_ref() {
                Some(m) => (m.game().clone(), mode_name(m.mode()).to_string()),
                None => return,
            }
        };
        let playing = self.view == View::Playing && self.busy != Some(Busy::Cpu);
        let legal = if self.settings.show_valid_moves && (what_if || playing) { game.legal_moves_bitboard() } else { 0 };
        let last = game.history().iter().rev().find(|e| !e.is_pass()).map(|e| e.pos);
        let show_cursor = what_if || self.view == View::Playing;
        board_view::draw(
            &self.screen,
            geo,
            &board_view::View {
                board: game.board(),
                cursor: if show_cursor { Some(self.cursor) } else { None },
                legal,
                last,
            },
        );
        let (b, wh) = game.counts();
        let y = geo.y0 + 8 * geo.cell + 6;
        self.screen.text(y, GlyphStyle::Small, &format!("{mode_label}   Black {b}  White {wh}"));
        let line2 = if what_if {
            format!("{} to move", name(game.current_player()))
        } else if self.view == View::GameOver {
            self.result_text()
        } else if self.busy == Some(Busy::Cpu) {
            "CPU thinking...".to_string()
        } else if self.busy == Some(Busy::Hint) {
            "Finding a hint...".to_string()
        } else if let Some(m) = self.m.as_ref() {
            match m.mode() {
                Mode::TwoPlayer => format!("{} to move", name(game.current_player())),
                Mode::VsCpu(_) => format!("Your move (you are {})", name(m.human())),
            }
        } else {
            String::new()
        };
        self.screen.text(y + line_h, GlyphStyle::Small, &line2);
        if !self.notice.is_empty() {
            self.screen.text(y + 2 * line_h, GlyphStyle::Small, &self.notice);
        }
    }

    fn result_text(&self) -> String {
        let Some(m) = self.m.as_ref() else { return String::new() };
        if m.resigned() {
            return "You resigned".into();
        }
        match m.outcome() {
            Some(Outcome::Win) => "You win".into(),
            Some(Outcome::Loss) => "CPU wins".into(),
            Some(Outcome::Draw) => "Draw".into(),
            None => match m.result().and_then(|r| r.winner()) {
                Some(p) => format!("{} wins", name(p)),
                None => "Draw".into(),
            },
        }
    }
}
