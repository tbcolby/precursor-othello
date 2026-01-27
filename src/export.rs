//! TCP export functionality
//!
//! Exports game records over TCP for external analysis.
//! This is an optional feature that can be enabled in settings.

use othello_core::{GameState, Player, pos_to_algebraic};

/// Export a game record as a formatted string
pub fn format_game_record(
    game: &GameState,
    mode: &str,
    player_color: Option<Player>,
    date: &str,
) -> String {
    let mut output = String::new();

    // Header
    output.push_str("[Othello Game Record]\n");
    output.push_str(&format!("Date: {}\n", date));
    output.push_str(&format!("Mode: {}\n", mode));

    if let Some(color) = player_color {
        output.push_str(&format!(
            "Player: {}\n",
            if color == Player::Black { "Black" } else { "White" }
        ));
    }

    // Result
    if let Some(result) = game.result() {
        let (black, white) = result.counts();
        match result.winner() {
            Some(Player::Black) => {
                output.push_str(&format!("Result: Black wins {}-{}\n", black, white))
            }
            Some(Player::White) => {
                output.push_str(&format!("Result: White wins {}-{}\n", black, white))
            }
            None => output.push_str(&format!("Result: Draw {}-{}\n", black, white)),
        }
    }

    output.push_str("\nMoves:\n");

    // Move list
    let history = game.history();
    let mut move_num = 1;
    let mut i = 0;

    while i < history.len() {
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

        output.push_str(&format!("{:2}. {} {}\n", move_num, black_move, white_move));

        move_num += 1;
        i += 2;
    }

    // Final score
    let (black, white) = game.counts();
    output.push_str(&format!(
        "\nFinal: \u{25CF} {} - \u{25CB} {}\n",
        black, white
    ));

    output
}

/// Export as compact move notation (just the moves)
pub fn format_compact(game: &GameState) -> String {
    let mut moves = Vec::new();

    for entry in game.history() {
        if entry.is_pass() {
            moves.push("--".to_string());
        } else {
            let alg = pos_to_algebraic(entry.pos);
            moves.push(core::str::from_utf8(&alg).unwrap_or("??").to_string());
        }
    }

    moves.join(" ")
}

/// Export game over TCP (port 7880)
/// Returns true if successful
#[allow(dead_code)]
pub fn export_via_tcp(game: &GameState, mode: &str, player_color: Option<Player>) -> bool {
    #[cfg(target_os = "none")]
    {
        use std::io::Write;
        use std::net::TcpListener;

        let record = format_game_record(game, mode, player_color, "");

        if let Ok(listener) = TcpListener::bind("0.0.0.0:7880") {
            log::info!("Waiting for connection on port 7880...");

            if let Ok((mut stream, _)) = listener.accept() {
                if stream.write_all(record.as_bytes()).is_ok() {
                    log::info!("Game record exported successfully");
                    return true;
                }
            }
        }
    }
    let _ = (game, mode, player_color);
    false
}
