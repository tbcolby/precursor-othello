//! UI drawing functions

use gam::{Gam, Gid, GlyphStyle};
use gam::menu::{Point, Rectangle, DrawStyle, PixelColor, Circle, Line, TextView, TextBounds};
use othello_core::{Board, Player, pos, pos_to_algebraic};

use crate::app::{OthelloApp, AppState, GameMode};
use crate::menu::MenuItem;

/// Layout constants
const HEADER_HEIGHT: isize = 24;
const FOOTER_HEIGHT: isize = 24;
const BOARD_SIZE: isize = 304;
const CELL_SIZE: isize = 38;
const DISC_RADIUS: isize = 14;
const VALID_MOVE_RADIUS: isize = 4;
const CURSOR_WIDTH: isize = 3;

/// Get board origin point
fn board_origin(screensize: Point, show_coords: bool) -> Point {
    let board_size = if show_coords { 272 } else { BOARD_SIZE };
    let margin_x = (screensize.x - board_size) / 2;
    let content_height = screensize.y - HEADER_HEIGHT - FOOTER_HEIGHT;
    let board_y = HEADER_HEIGHT + (content_height - board_size - 60) / 2;

    if show_coords {
        Point::new(margin_x + 16, board_y + 16) // Space for labels
    } else {
        Point::new(margin_x, board_y)
    }
}

/// Get cell size based on coordinate display
fn cell_size(show_coords: bool) -> isize {
    if show_coords { 34 } else { CELL_SIZE }
}

/// Draw the complete app
pub fn draw(app: &OthelloApp, gam: &Gam) {
    // Clear screen
    clear_screen(gam, app.gid, app.screensize);

    match &app.state {
        AppState::MainMenu => draw_main_menu(app, gam),
        AppState::NewGameMenu => draw_new_game_menu(app, gam),
        AppState::SettingsMenu => draw_settings_menu(app, gam),
        AppState::Statistics => draw_statistics(app, gam),
        AppState::Playing { game, mode, player_color, cursor_pos, ai_thinking, thinking_dots, show_pass_notice } => {
            draw_playing(app, gam, game, *mode, *player_color, *cursor_pos, *ai_thinking, *thinking_dots, *show_pass_notice);
        }
        AppState::GameOver { game, mode, player_color } => {
            draw_game_over(app, gam, game, *mode, *player_color);
        }
        AppState::WhatIf { current_game, view_index, branched, cursor_pos, base_game } => {
            draw_what_if(app, gam, base_game, current_game, *view_index, *branched, *cursor_pos);
        }
        AppState::MoveHistory { game, scroll_offset } => {
            draw_history(app, gam, game, *scroll_offset);
        }
        AppState::Help { context, .. } => {
            crate::help::draw_help(app, gam, *context);
        }
    }
}

/// Clear the screen
fn clear_screen(gam: &Gam, gid: gam::Gid, screensize: Point) {
    gam.draw_rectangle(
        gid,
        Rectangle::new_with_style(
            Point::new(0, 0),
            screensize,
            DrawStyle {
                fill_color: Some(PixelColor::Light),
                stroke_color: None,
                stroke_width: 0,
            },
        ),
    )
    .ok();
}

/// Draw header bar
fn draw_header(app: &OthelloApp, gam: &Gam, title: &str, black_count: u32, white_count: u32) {
    let gid = app.gid;

    // Draw header background line
    gam.draw_line(
        gid,
        Line::new_with_style(
            Point::new(0, HEADER_HEIGHT),
            Point::new(app.screensize.x, HEADER_HEIGHT),
            DrawStyle::new(PixelColor::Dark, PixelColor::Dark, 1),
        ),
    )
    .ok();

    // Title
    let mut tv = TextView::new(
        gid,
        TextBounds::GrowableFromTl(Point::new(8, 4), 150),
    );
    tv.style = GlyphStyle::Bold;
    use core::fmt::Write;
    write!(tv.text, "{}", title).ok();
    gam.post_textview(&mut tv).ok();

    // Score
    let mut tv = TextView::new(
        gid,
        TextBounds::GrowableFromTr(Point::new(app.screensize.x - 8, 4), 150),
    );
    tv.style = GlyphStyle::Regular;
    write!(tv.text, "\u{25CF} {:02}  \u{25CB} {:02}", black_count, white_count).ok();
    gam.post_textview(&mut tv).ok();
}

/// Draw footer bar
fn draw_footer(app: &OthelloApp, gam: &Gam) {
    let gid = app.gid;
    let footer_y = app.screensize.y - FOOTER_HEIGHT;

    // Draw footer line
    gam.draw_line(
        gid,
        Line::new_with_style(
            Point::new(0, footer_y),
            Point::new(app.screensize.x, footer_y),
            DrawStyle::new(PixelColor::Dark, PixelColor::Dark, 1),
        ),
    )
    .ok();

    // F1 Menu hint
    let mut tv = TextView::new(
        gid,
        TextBounds::GrowableFromTl(Point::new(8, footer_y + 4), 100),
    );
    tv.style = GlyphStyle::Small;
    use core::fmt::Write;
    write!(tv.text, "F1 Menu").ok();
    gam.post_textview(&mut tv).ok();

    // F4 Exit hint
    let mut tv = TextView::new(
        gid,
        TextBounds::GrowableFromTr(Point::new(app.screensize.x - 8, footer_y + 4), 100),
    );
    tv.style = GlyphStyle::Small;
    write!(tv.text, "F4 Exit").ok();
    gam.post_textview(&mut tv).ok();
}

/// Draw the Othello board
fn draw_board(app: &OthelloApp, gam: &Gam, board: &Board, cursor: Option<(u8, u8)>, show_valid: bool, current_player: Player, last_move: Option<u8>) {
    let gid = app.gid;
    let show_coords = app.settings.show_coordinates;
    let origin = board_origin(app.screensize, show_coords);
    let cell = cell_size(show_coords);
    let board_px = cell * 8;

    // Draw coordinate labels if enabled
    if show_coords {
        // Column labels (A-H)
        for col in 0..8 {
            let x = origin.x + col * cell + cell / 2 - 4;
            let mut tv = TextView::new(
                gid,
                TextBounds::GrowableFromTl(Point::new(x, origin.y - 14), 20),
            );
            tv.style = GlyphStyle::Small;
            use core::fmt::Write;
            write!(tv.text, "{}", (b'A' + col as u8) as char).ok();
            gam.post_textview(&mut tv).ok();
        }

        // Row labels (1-8)
        for row in 0..8 {
            let y = origin.y + row * cell + cell / 2 - 6;
            let mut tv = TextView::new(
                gid,
                TextBounds::GrowableFromTl(Point::new(origin.x - 14, y), 20),
            );
            tv.style = GlyphStyle::Small;
            use core::fmt::Write;
            write!(tv.text, "{}", row + 1).ok();
            gam.post_textview(&mut tv).ok();
        }
    }

    // Draw board border
    gam.draw_rectangle(
        gid,
        Rectangle::new_with_style(
            origin,
            Point::new(origin.x + board_px, origin.y + board_px),
            DrawStyle::new(PixelColor::Dark, PixelColor::Light, 2),
        ),
    )
    .ok();

    // Draw grid lines
    for i in 1..8 {
        // Vertical lines
        gam.draw_line(
            gid,
            Line::new_with_style(
                Point::new(origin.x + i * cell, origin.y),
                Point::new(origin.x + i * cell, origin.y + board_px),
                DrawStyle::new(PixelColor::Dark, PixelColor::Dark, 1),
            ),
        )
        .ok();
        // Horizontal lines
        gam.draw_line(
            gid,
            Line::new_with_style(
                Point::new(origin.x, origin.y + i * cell),
                Point::new(origin.x + board_px, origin.y + i * cell),
                DrawStyle::new(PixelColor::Dark, PixelColor::Dark, 1),
            ),
        )
        .ok();
    }

    // Get valid moves bitboard
    let valid_moves = if show_valid {
        othello_core::legal_moves_bitboard(board, current_player)
    } else {
        0
    };

    // Draw discs and valid move indicators
    let disc_r: isize = if show_coords { 12 } else { DISC_RADIUS };
    let valid_r: isize = if show_coords { 3 } else { VALID_MOVE_RADIUS };

    for row in 0..8 {
        for col in 0..8 {
            let position = pos(row, col);
            let cx = origin.x + col as isize * cell + cell / 2;
            let cy = origin.y + row as isize * cell + cell / 2;
            let center = Point::new(cx, cy);

            // Draw disc if present
            if let Some(player) = board.get_disc(position) {
                let (fill, stroke) = match player {
                    Player::Black => (PixelColor::Dark, PixelColor::Dark),
                    Player::White => (PixelColor::Light, PixelColor::Dark),
                };
                gam.draw_circle(
                    gid,
                    Circle::new_with_style(
                        center,
                        disc_r,
                        DrawStyle::new(stroke, fill, 2),
                    ),
                )
                .ok();
            } else if (valid_moves & (1u64 << position)) != 0 {
                // Draw valid move indicator
                gam.draw_circle(
                    gid,
                    Circle::new_with_style(
                        center,
                        valid_r,
                        DrawStyle::new(PixelColor::Dark, PixelColor::Dark, 1),
                    ),
                )
                .ok();
            }

            // Draw last move marker
            if let Some(last) = last_move {
                if last == position {
                    let corner_size = 4isize;
                    // Top-left corner
                    gam.draw_rectangle(
                        gid,
                        Rectangle::new_with_style(
                            Point::new(origin.x + col as isize * cell + 2, origin.y + row as isize * cell + 2),
                            Point::new(origin.x + col as isize * cell + 2 + corner_size, origin.y + row as isize * cell + 2 + corner_size),
                            DrawStyle::new(PixelColor::Dark, PixelColor::Dark, 1),
                        ),
                    )
                    .ok();
                    // Top-right corner
                    gam.draw_rectangle(
                        gid,
                        Rectangle::new_with_style(
                            Point::new(origin.x + (col as isize + 1) * cell - 2 - corner_size, origin.y + row as isize * cell + 2),
                            Point::new(origin.x + (col as isize + 1) * cell - 2, origin.y + row as isize * cell + 2 + corner_size),
                            DrawStyle::new(PixelColor::Dark, PixelColor::Dark, 1),
                        ),
                    )
                    .ok();
                    // Bottom-left corner
                    gam.draw_rectangle(
                        gid,
                        Rectangle::new_with_style(
                            Point::new(origin.x + col as isize * cell + 2, origin.y + (row as isize + 1) * cell - 2 - corner_size),
                            Point::new(origin.x + col as isize * cell + 2 + corner_size, origin.y + (row as isize + 1) * cell - 2),
                            DrawStyle::new(PixelColor::Dark, PixelColor::Dark, 1),
                        ),
                    )
                    .ok();
                    // Bottom-right corner
                    gam.draw_rectangle(
                        gid,
                        Rectangle::new_with_style(
                            Point::new(origin.x + (col as isize + 1) * cell - 2 - corner_size, origin.y + (row as isize + 1) * cell - 2 - corner_size),
                            Point::new(origin.x + (col as isize + 1) * cell - 2, origin.y + (row as isize + 1) * cell - 2),
                            DrawStyle::new(PixelColor::Dark, PixelColor::Dark, 1),
                        ),
                    )
                    .ok();
                }
            }
        }
    }

    // Draw cursor
    if let Some((row, col)) = cursor {
        let x = origin.x + col as isize * cell;
        let y = origin.y + row as isize * cell;
        gam.draw_rectangle(
            gid,
            Rectangle::new_with_style(
                Point::new(x + 1, y + 1),
                Point::new(x + cell - 1, y + cell - 1),
                DrawStyle {
                    fill_color: None,
                    stroke_color: Some(PixelColor::Dark),
                    stroke_width: CURSOR_WIDTH,
                },
            ),
        )
        .ok();
    }
}

/// Draw main menu
fn draw_main_menu(app: &OthelloApp, gam: &Gam) {
    draw_header(app, gam, "OTHELLO", 0, 0);
    draw_footer(app, gam);

    let gid = app.gid;
    let center_x = app.screensize.x / 2;
    let center_y = app.screensize.y / 2;

    // Title
    let mut tv = TextView::new(
        gid,
        TextBounds::BoundingBox(Rectangle::new_coords(0, center_y - 60, app.screensize.x, center_y)),
    );
    tv.style = GlyphStyle::ExtraLarge;
    use core::fmt::Write;
    write!(tv.text, "OTHELLO").ok();
    gam.post_textview(&mut tv).ok();

    // Instructions
    let mut tv = TextView::new(
        gid,
        TextBounds::BoundingBox(Rectangle::new_coords(0, center_y + 20, app.screensize.x, center_y + 100)),
    );
    tv.style = GlyphStyle::Regular;
    write!(tv.text, "Press F1 for Menu").ok();
    gam.post_textview(&mut tv).ok();
}

/// Draw new game menu
fn draw_new_game_menu(app: &OthelloApp, gam: &Gam) {
    draw_header(app, gam, "NEW GAME", 0, 0);
    draw_footer(app, gam);

    let gid = app.gid;
    let start_y = HEADER_HEIGHT + 40;

    let options = [
        "1. Easy",
        "2. Medium",
        "3. Hard",
        "4. Expert",
        "",
        "5. Two Players",
    ];

    for (i, option) in options.iter().enumerate() {
        let mut tv = TextView::new(
            gid,
            TextBounds::GrowableFromTl(Point::new(40, start_y + i as isize * 30), 256),
        );
        tv.style = GlyphStyle::Regular;
        use core::fmt::Write;
        write!(tv.text, "{}", option).ok();
        gam.post_textview(&mut tv).ok();
    }
}

/// Draw settings menu
fn draw_settings_menu(app: &OthelloApp, gam: &Gam) {
    draw_header(app, gam, "SETTINGS", 0, 0);
    draw_footer(app, gam);

    let gid = app.gid;
    let start_y = HEADER_HEIGHT + 40;

    let check = |b: bool| if b { "[X]" } else { "[ ]" };

    let options = [
        format!("1. Show Coordinates  {}", check(app.settings.show_coordinates)),
        format!("2. Show Valid Moves  {}", check(app.settings.show_valid_moves)),
        format!("3. Allow Undo        {}", check(app.settings.allow_undo)),
        format!("4. Vibration         {}", check(app.settings.vibration)),
    ];

    for (i, option) in options.iter().enumerate() {
        let mut tv = TextView::new(
            gid,
            TextBounds::GrowableFromTl(Point::new(40, start_y + i as isize * 30), 280),
        );
        tv.style = GlyphStyle::Regular;
        use core::fmt::Write;
        write!(tv.text, "{}", option).ok();
        gam.post_textview(&mut tv).ok();
    }
}

/// Draw statistics
fn draw_statistics(app: &OthelloApp, gam: &Gam) {
    draw_header(app, gam, "STATISTICS", 0, 0);
    draw_footer(app, gam);

    let gid = app.gid;
    let start_y = HEADER_HEIGHT + 30;
    let stats = &app.stats;

    // Draw each stats line
    let mut y = start_y;
    let line_height = 22isize;

    // Easy stats
    draw_stats_line(gam, gid, y, "vs CPU Easy", true);
    y += line_height;
    draw_stats_line(gam, gid, y, &format!("  Won: {}  Lost: {}  Draw: {}", stats.easy_wins, stats.easy_losses, stats.easy_draws), false);
    y += line_height * 2;

    // Medium stats
    draw_stats_line(gam, gid, y, "vs CPU Medium", true);
    y += line_height;
    draw_stats_line(gam, gid, y, &format!("  Won: {}  Lost: {}  Draw: {}", stats.medium_wins, stats.medium_losses, stats.medium_draws), false);
    y += line_height * 2;

    // Hard stats
    draw_stats_line(gam, gid, y, "vs CPU Hard", true);
    y += line_height;
    draw_stats_line(gam, gid, y, &format!("  Won: {}  Lost: {}  Draw: {}", stats.hard_wins, stats.hard_losses, stats.hard_draws), false);
    y += line_height * 2;

    // Expert stats
    draw_stats_line(gam, gid, y, "vs CPU Expert", true);
    y += line_height;
    draw_stats_line(gam, gid, y, &format!("  Won: {}  Lost: {}  Draw: {}", stats.expert_wins, stats.expert_losses, stats.expert_draws), false);
    y += line_height * 2;

    // Two player stats
    draw_stats_line(gam, gid, y, &format!("Two Player Games: {}", stats.two_player_games), true);
}

fn draw_stats_line(gam: &Gam, gid: Gid, y: isize, text: &str, bold: bool) {
    let mut tv = TextView::new(
        gid,
        TextBounds::GrowableFromTl(Point::new(20, y), 300),
    );
    tv.style = if bold { GlyphStyle::Bold } else { GlyphStyle::Regular };
    use core::fmt::Write;
    write!(tv.text, "{}", text).ok();
    gam.post_textview(&mut tv).ok();
}

/// Draw playing state
fn draw_playing(
    app: &OthelloApp,
    gam: &Gam,
    game: &othello_core::GameState,
    mode: GameMode,
    player_color: Player,
    cursor_pos: (u8, u8),
    ai_thinking: bool,
    thinking_dots: u8,
    show_pass_notice: bool,
) {
    let (black, white) = game.counts();
    draw_header(app, gam, "OTHELLO", black, white);
    draw_footer(app, gam);

    // Get last move position
    let last_move = game.last_move().map(|e| if e.is_pass() { 255 } else { e.pos });

    draw_board(
        app,
        gam,
        game.board(),
        Some(cursor_pos),
        app.settings.show_valid_moves,
        game.current_player(),
        last_move,
    );

    // Status area
    let status_y = app.screensize.y - FOOTER_HEIGHT - 60;
    let gid = app.gid;

    // Mobility info
    let black_moves = othello_core::count_moves(game.board(), Player::Black);
    let white_moves = othello_core::count_moves(game.board(), Player::White);

    let mut tv = TextView::new(
        gid,
        TextBounds::GrowableFromTl(Point::new(16, status_y), 320),
    );
    tv.style = GlyphStyle::Small;
    use core::fmt::Write;

    let last_str = if let Some(pos) = last_move {
        if pos < 64 {
            let alg = pos_to_algebraic(pos);
            core::str::from_utf8(&alg).unwrap_or("--").to_string()
        } else {
            "Pass".to_string()
        }
    } else {
        "--".to_string()
    };
    write!(tv.text, "\u{25CF} {} moves  \u{25CB} {} moves  Last: {}", black_moves, white_moves, last_str).ok();
    gam.post_textview(&mut tv).ok();

    // Turn indicator
    let mut tv = TextView::new(
        gid,
        TextBounds::GrowableFromTl(Point::new(16, status_y + 20), 320),
    );
    tv.style = GlyphStyle::Regular;

    if ai_thinking {
        let dots = ".".repeat((thinking_dots + 1) as usize);
        write!(tv.text, "CPU thinking{}", dots).ok();
    } else if show_pass_notice {
        write!(tv.text, "No legal moves! Pass to opponent").ok();
    } else {
        let current = game.current_player();
        let disc = if current == Player::Black { "\u{25CF}" } else { "\u{25CB}" };
        match mode {
            GameMode::VsCpu(_) => {
                if current == player_color {
                    write!(tv.text, "Your move ({})", disc).ok();
                } else {
                    write!(tv.text, "CPU's move ({})", disc).ok();
                }
            }
            GameMode::TwoPlayer => {
                let color = if current == Player::Black { "Black" } else { "White" };
                write!(tv.text, "{}'s move ({})", color, disc).ok();
            }
        }
    }
    gam.post_textview(&mut tv).ok();
}

/// Draw game over state
fn draw_game_over(
    app: &OthelloApp,
    gam: &Gam,
    game: &othello_core::GameState,
    mode: GameMode,
    player_color: Player,
) {
    let (black, white) = game.counts();
    draw_header(app, gam, "GAME OVER", black, white);
    draw_footer(app, gam);

    draw_board(app, gam, game.board(), None, false, Player::Black, None);

    // Result box
    let gid = app.gid;
    let center_x = app.screensize.x / 2;
    let box_y = app.screensize.y / 2 - 30;

    // Draw result box background
    gam.draw_rectangle(
        gid,
        Rectangle::new_with_style(
            Point::new(center_x - 100, box_y),
            Point::new(center_x + 100, box_y + 80),
            DrawStyle::new(PixelColor::Dark, PixelColor::Light, 2),
        ),
    )
    .ok();

    // Result text
    let result_text = if let Some(result) = game.result() {
        match mode {
            GameMode::VsCpu(_) => {
                match result.winner() {
                    Some(winner) if winner == player_color => "YOU WIN!",
                    Some(_) => "CPU WINS!",
                    None => "DRAW!",
                }
            }
            GameMode::TwoPlayer => {
                match result.winner() {
                    Some(Player::Black) => "BLACK WINS!",
                    Some(Player::White) => "WHITE WINS!",
                    None => "DRAW!",
                }
            }
        }
    } else {
        "GAME OVER"
    };

    let mut tv = TextView::new(
        gid,
        TextBounds::BoundingBox(Rectangle::new_coords(center_x - 90, box_y + 10, center_x + 90, box_y + 40)),
    );
    tv.style = GlyphStyle::Bold;
    use core::fmt::Write;
    write!(tv.text, "{}", result_text).ok();
    gam.post_textview(&mut tv).ok();

    // Score
    let mut tv = TextView::new(
        gid,
        TextBounds::BoundingBox(Rectangle::new_coords(center_x - 90, box_y + 45, center_x + 90, box_y + 70)),
    );
    tv.style = GlyphStyle::Regular;
    write!(tv.text, "\u{25CF} {}  -  \u{25CB} {}", black, white).ok();
    gam.post_textview(&mut tv).ok();

    // Instructions
    let mut tv = TextView::new(
        gid,
        TextBounds::GrowableFromTl(Point::new(16, app.screensize.y - FOOTER_HEIGHT - 30), 320),
    );
    tv.style = GlyphStyle::Small;
    write!(tv.text, "Enter: New Game   W: What If   N: Mode").ok();
    gam.post_textview(&mut tv).ok();
}

/// Draw What If mode
fn draw_what_if(
    app: &OthelloApp,
    gam: &Gam,
    base_game: &othello_core::GameState,
    current_game: &othello_core::GameState,
    view_index: usize,
    branched: bool,
    cursor_pos: (u8, u8),
) {
    let title = if branched { "WHAT IF (BRANCHED)" } else { "WHAT IF" };
    let (black, white) = current_game.counts();
    draw_header(app, gam, title, black, white);
    draw_footer(app, gam);

    draw_board(
        app,
        gam,
        current_game.board(),
        if branched { Some(cursor_pos) } else { None },
        branched && app.settings.show_valid_moves,
        current_game.current_player(),
        None,
    );

    // Navigation info
    let gid = app.gid;
    let status_y = app.screensize.y - FOOTER_HEIGHT - 40;

    let mut tv = TextView::new(
        gid,
        TextBounds::GrowableFromTl(Point::new(16, status_y), 320),
    );
    tv.style = GlyphStyle::Small;
    use core::fmt::Write;
    write!(tv.text, "Move {}/{}  Empty: {}", view_index, base_game.move_count(), current_game.empty_count()).ok();
    gam.post_textview(&mut tv).ok();

    let mut tv = TextView::new(
        gid,
        TextBounds::GrowableFromTl(Point::new(16, status_y + 18), 320),
    );
    tv.style = GlyphStyle::Small;
    if branched {
        write!(tv.text, "Playing alternate timeline...").ok();
    } else {
        write!(tv.text, "Left/Right: Navigate  Enter: Branch").ok();
    }
    gam.post_textview(&mut tv).ok();
}

/// Draw move history
fn draw_history(
    app: &OthelloApp,
    gam: &Gam,
    game: &othello_core::GameState,
    scroll_offset: usize,
) {
    let (black, white) = game.counts();
    draw_header(app, gam, "MOVE HISTORY", black, white);
    draw_footer(app, gam);

    let gid = app.gid;
    let start_y = HEADER_HEIGHT + 30;
    let history = game.history();

    // Column headers
    let mut tv = TextView::new(
        gid,
        TextBounds::GrowableFromTl(Point::new(20, start_y), 300),
    );
    tv.style = GlyphStyle::Bold;
    use core::fmt::Write;
    write!(tv.text, " #   \u{25CF}        \u{25CB}").ok();
    gam.post_textview(&mut tv).ok();

    // Move pairs
    let mut line = 1;
    let mut move_num = 1 + scroll_offset;
    let mut i = scroll_offset * 2;

    while i < history.len() && line < 15 {
        let black_move = if i < history.len() {
            let entry = &history[i];
            if entry.is_pass() {
                "--".to_string()
            } else {
                let alg = pos_to_algebraic(entry.pos);
                core::str::from_utf8(&alg).unwrap_or("??").to_string()
            }
        } else {
            "".to_string()
        };

        let white_move = if i + 1 < history.len() {
            let entry = &history[i + 1];
            if entry.is_pass() {
                "--".to_string()
            } else {
                let alg = pos_to_algebraic(entry.pos);
                core::str::from_utf8(&alg).unwrap_or("??").to_string()
            }
        } else {
            "".to_string()
        };

        let mut tv = TextView::new(
            gid,
            TextBounds::GrowableFromTl(Point::new(20, start_y + line * 22), 300),
        );
        tv.style = GlyphStyle::Monospace;
        write!(tv.text, "{:2}.  {}       {}", move_num, black_move, white_move).ok();
        gam.post_textview(&mut tv).ok();

        move_num += 1;
        i += 2;
        line += 1;
    }

    // Total
    let mut tv = TextView::new(
        gid,
        TextBounds::GrowableFromTl(Point::new(20, app.screensize.y - FOOTER_HEIGHT - 30), 300),
    );
    tv.style = GlyphStyle::Small;
    write!(tv.text, "Total: {} moves", history.len()).ok();
    gam.post_textview(&mut tv).ok();
}

/// Draw menu overlay
pub fn draw_menu(app: &OthelloApp, gam: &Gam) {
    let gid = app.gid;
    let menu = &app.menu;

    let menu_width = 200isize;
    let item_height = 24isize;
    let menu_height = (menu.items.len() as isize + 1) * item_height + 10;
    let x = (app.screensize.x - menu_width) / 2;
    let y = (app.screensize.y - menu_height) / 2;

    // Background
    gam.draw_rectangle(
        gid,
        Rectangle::new_with_style(
            Point::new(x, y),
            Point::new(x + menu_width, y + menu_height),
            DrawStyle::new(PixelColor::Dark, PixelColor::Light, 2),
        ),
    )
    .ok();

    // Menu items
    for (i, item) in menu.items.iter().enumerate() {
        let item_y = y + 8 + i as isize * item_height;
        let is_selected = i == menu.selected;

        if is_selected {
            gam.draw_rectangle(
                gid,
                Rectangle::new_with_style(
                    Point::new(x + 4, item_y),
                    Point::new(x + menu_width - 4, item_y + item_height - 2),
                    DrawStyle::new(PixelColor::Dark, PixelColor::Dark, 1),
                ),
            )
            .ok();
        }

        let mut tv = TextView::new(
            gid,
            TextBounds::GrowableFromTl(Point::new(x + 12, item_y + 4), (menu_width - 24) as u16),
        );
        tv.style = if is_selected { GlyphStyle::Bold } else { GlyphStyle::Regular };
        tv.invert = is_selected;
        use core::fmt::Write;
        write!(tv.text, "{}", item.label()).ok();
        gam.post_textview(&mut tv).ok();
    }

    // Footer hint
    let mut tv = TextView::new(
        gid,
        TextBounds::BoundingBox(Rectangle::new_coords(x, y + menu_height - item_height, x + menu_width, y + menu_height)),
    );
    tv.style = GlyphStyle::Small;
    use core::fmt::Write;
    write!(tv.text, "F4 to close").ok();
    gam.post_textview(&mut tv).ok();
}
