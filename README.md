# Othello

A tournament-quality Othello (Reversi) game for the [Precursor](https://www.crowdsupply.com/sutajio-kosagi/precursor) device with AI opponent and full game analysis capabilities.

## Features

### AI Opponent with Four Difficulty Levels

| Level | Search Depth | Features |
|-------|--------------|----------|
| **Easy** | 2-ply | Disc count evaluation only |
| **Medium** | 4-ply | Positional awareness (corners, edges) |
| **Hard** | 6-ply | Full evaluation + endgame solver (12 empties) |
| **Expert** | 8-ply | Opening book + endgame solver (14 empties) |

The AI uses minimax search with alpha-beta pruning and evaluates positions based on corner control, mobility, stability, and frontier disc count.

### Two-Player Mode

Pass the device between players for head-to-head games. The game tracks whose turn it is and enforces all standard Othello rules.

### Game Analysis - "What If" Mode

After any game, review the entire move history and explore alternate lines of play:

- Step through any game position with Left/Right arrows
- Branch from any point by making an alternate move
- Continue playing out "what if" scenarios
- See how different choices would have changed the outcome

### Persistent Storage

All data is stored encrypted in the PDDB:

- **Settings**: Coordinate display, valid move indicators, undo, vibration
- **Statistics**: Win/loss/draw records for each difficulty level
- **Save Game**: Resume interrupted games exactly where you left off

### Visual Feedback

- Valid move indicators (small dots on legal squares)
- Last move highlighting (corner markers)
- Cursor-based navigation with thick border highlight
- Optional coordinate display (A-H, 1-8)

## Screenshots

*Screenshots coming soon - app builds successfully and runs in Renode emulator.*

## Controls

### Universal Keys

| Key | Action |
|-----|--------|
| **F1** | Open context menu |
| **F4** | Exit / Back |

### During Game

| Key | Action |
|-----|--------|
| **Arrow Keys** | Move cursor |
| **Enter** | Place disc |
| **F2** | Show hint (AI's best move) |
| **U** | Undo last move (if enabled) |

### Game Over

| Key | Action |
|-----|--------|
| **Enter** | New game (same mode) |
| **W** | Enter What If mode |
| **N** | Select new game mode |

### What If Mode

| Key | Action |
|-----|--------|
| **Left/Right** | Step through history |
| **Arrow Keys** | Move cursor (when branching) |
| **Enter** | Play alternate move |

## Rules

Othello is played on an 8x8 board. Black moves first. Players take turns placing discs of their color. When you place a disc, any opponent discs that are "outflanked" (in a straight line between your new disc and another of your discs) are flipped to your color.

You must make a move that flips at least one disc. If you cannot, you pass. The game ends when neither player can move (usually when the board is full). The player with the most discs wins.

**Random Color Assignment**: In vs CPU mode, your color (Black or White) is randomly assigned each game using hardware TRNG, ensuring fair variety.

## Installation

Clone into your xous-core apps directory:

```bash
cd xous-core/apps
git clone https://github.com/tbcolby/precursor-othello.git othello
```

Add to `xous-core/Cargo.toml`:

```toml
[workspace]
members = [
    # ... existing apps ...
    "apps/othello",
    "libs/othello-core",
]
```

Add to `apps/manifest.json`:

```json
{
  "othello": {
    "context_name": "Othello",
    "menu_name": {
      "appmenu.othello": {
        "en": "Othello",
        "en-tts": "Othello"
      }
    }
  }
}
```

Build and run:

```bash
# For Renode emulator
cargo xtask renode-image othello

# For real hardware
cargo xtask app-image othello
```

## Architecture

```
precursor-othello/
├── Cargo.toml              # Main app manifest
├── src/
│   ├── main.rs             # Entry point, event loop
│   ├── app.rs              # State machine, game logic
│   ├── ui.rs               # Drawing functions
│   ├── menu.rs             # F1 context menu system
│   ├── help.rs             # Context-sensitive help screens
│   ├── storage.rs          # PDDB persistence
│   ├── review.rs           # What If mode logic
│   ├── feedback.rs         # Vibration, TRNG
│   └── export.rs           # TCP game export
│
└── libs/othello-core/      # Platform-independent game engine
    ├── Cargo.toml
    └── src/
        ├── lib.rs          # Public API
        ├── board.rs        # Bitboard representation (two u64)
        ├── moves.rs        # Move generation, flip calculation
        ├── game.rs         # GameState with full history
        ├── ai.rs           # Minimax + alpha-beta pruning
        ├── eval.rs         # Position evaluation function
        └── opening.rs      # Opening book for Expert mode
```

### Key Design Decisions

1. **Bitboard Representation**: The board uses two 64-bit integers (one for black, one for white), enabling efficient move generation and flip calculation using bit manipulation.

2. **Separate Game Engine**: `othello-core` has no Xous dependencies and can be tested on the host system with `cargo test`.

3. **State Machine Architecture**: The app uses a clean state machine (`AppState` enum) for predictable UI flow between menus, gameplay, and review modes.

4. **F1/F4 Keyboard Standard**: Follows the Precursor app conventions for menu access and exit.

5. **Random Color Assignment**: Uses hardware TRNG for true randomness when assigning player color vs CPU.

## Testing

The game engine includes 40+ unit tests covering board operations, move generation, game state, and AI:

```bash
cd libs/othello-core
cargo test
```

## PDDB Storage

| Dictionary | Key | Contents |
|------------|-----|----------|
| `othello.settings` | `config` | 10-byte settings blob |
| `othello.stats` | `stats` | 26-byte statistics (13 × u16) |
| `othello.save` | `current` | Serialized game state with history |

## Technical Details

- **Display**: 336×536 pixels, 1-bit (black/white only)
- **Board Size**: 304×304 pixels (38px cells) or 272×272 with coordinates (34px cells)
- **Disc Rendering**: Filled circles using GAM draw_circle
- **AI Performance**: Expert mode typically responds in 1-3 seconds on 100MHz CPU

## License

Apache License 2.0 - See [LICENSE](LICENSE) for details.

## Author

**Tyler Colby**
- GitHub: [@tbcolby](https://github.com/tbcolby)

## Related Projects

- [precursor-writer](https://github.com/tbcolby/precursor-writer) - Markdown editor for Precursor
- [precursor-flashcards](https://github.com/tbcolby/precursor-flashcards) - Spaced repetition flashcards
- [precursor-timers](https://github.com/tbcolby/precursor-timers) - Multiple concurrent timers
- [xous-dev-toolkit](https://github.com/tbcolby/xous-dev-toolkit) - Development tools and templates

## Acknowledgments

Built using the [xous-dev-toolkit](https://github.com/tbcolby/xous-dev-toolkit) development methodology for Precursor apps.
