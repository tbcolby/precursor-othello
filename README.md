# precursor-othello

Othello (Reversi) for the [Precursor](https://precursor.dev) with an AI opponent in four strengths, two-player mode, undo, hints, save and resume, statistics, and a "What if" review of finished games.

## What it does

- Standard rules on an 8x8 board, bitboard engine. Forced passes are applied automatically; a double pass or a full board ends the game.
- Move generation is checked against the standard perft counts from the opening position (depth 1 to 6: 4, 12, 56, 244, 1396, 8200) and against a slow reference implementation over thousands of random positions.
- **vs CPU**: Easy, Medium, Hard, Expert. Your colour is chosen with the hardware TRNG each game (Black moves first). The CPU thinks on a background thread, so the screen and keys stay responsive; leaving the app or going to the background cancels the search.
- **Two players** on one device.
- Hint (F2, searched at Hard strength), Undo (U; goes back to your previous turn against the CPU, one move in two-player), Resign (counts as a loss), Move history.
- Save and resume one game. Settings (coordinates, legal-move dots, undo allowed) and win/loss/draw statistics per level.
- **What if**: after a game, step through it (F2/F3) and play a different move from any position.

## Keys

| Screen | Key | Action |
|---|---|---|
| Menus | ↑ ↓, Enter | Move, choose |
| | Backspace / F4 | Back (leaves the app from the main menu) |
| Game | arrows | Move cursor |
| | Enter | Place disc |
| | F1 | Game menu (hint, undo, history, resign, save and exit, help) |
| | F2 / U | Hint / Undo |
| | Backspace / F4 | Save and go to the main menu |
| Game over | Enter / N / W / H | Play again / new game / What if / move list |
| | Backspace / F4 | Main menu |
| What if | arrows, Enter | Cursor, play alternate move |
| | F2 / F3 (or `,` `.`) | Step back / forward (abandons a branch) |
| | Backspace / F4 | Back |

The `∴` menu key belongs to the system; the app never receives it.

## Storage

PDDB dictionary `othello`, three keys, each a small binary record starting with a version byte:

- `settings`: 3 flags. `stats`: 4 levels x (wins, losses, draws) and a two-player count.
- `save`: mode, human colour, the move list, and both bitboards. On load the moves are replayed and the result must equal the stored board; otherwise the save is reported unreadable.
- A record that is unreadable or from another version is left untouched and never overwritten. The game is saved only when it changed (leaving the game, going to the background), and is deleted when the game ends.

## How strong is the CPU, honestly

The search is negamax with alpha-beta and iterative deepening, a hand-written evaluation (corners, mobility, frontier, stability, disc count), and an exact solver for the last 8 (Hard) or 10 (Expert) empty squares. Each search has a node budget and a wall-clock limit (Easy 2 s, Medium 4 s, Hard 8 s, Expert 15 s); when the limit hits, the result of the last completed depth is played.

Nothing here has been timed on a Precursor. On a 100 MHz core the deeper levels will likely stop at a shallower depth than their label (6 and 8 plies) and so play weaker than on a PC. The limits are constants in `src/ai.rs` and `libs/othello-core/src/ai.rs` to tune once measured. There is no opening book (the old one never matched a position and was removed) and no transposition table. Play is deterministic for a given position.

## Not implemented

Vibration, sound, flip and "thinking" animations, danger-zone highlighting, and the TCP game export from earlier versions. They were never wired up, so they were removed rather than left as dead settings.

## Limitations

- Renode capture not yet run. `capture.py` is written but not executed; verification so far is the hosted build, clippy, lint and the host tests in `libs/othello-core`.
- Final scoring counts discs only (empty squares are not awarded to the winner).

## Build and test

```bash
cd libs/othello-core && cargo test                      # engine, AI, match flow, persistence
cd xous-dev-toolkit && bin/integrate --xous-root ~/xous-core --grade ../precursor-othello
cd ~/xous-core && cargo xtask run othello                # hosted mode
```

Layout: `libs/othello-core/` (pure logic, no Xous dependencies; `bin/integrate` copies it to `xous-core/libs/`), `src/main.rs` (message routing), `src/app.rs` (screens, PDDB, input), `src/ai.rs` (search thread), `src/board_view.rs` (drawing). Built on `precursor-appkit`.

---

Made by Tyler Colby, [Colby's Data Movers, LLC](https://colbysdatamovers.com).
