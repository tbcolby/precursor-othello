# Othello for Precursor

Tournament-quality Othello (Reversi) game with AI opponent for the [Precursor](https://www.crowdsupply.com/sutajio-kosagi/precursor) hardware platform.

## Features

- **Multiple AI Difficulty Levels**
  - Easy: Basic play, great for learning
  - Medium: Positional awareness
  - Hard: Deep search with endgame solving
  - Expert: Tournament-level play with opening book

- **Two-Player Mode**: Pass the device between players

- **Full Game Analysis**
  - Move history review
  - "What If" mode to explore alternate lines
  - Step through any game position

- **Persistent Storage**
  - Save and resume games
  - Settings preserved across sessions
  - Win/loss statistics by difficulty

- **Visual Feedback**
  - Valid move indicators
  - Last move highlighting
  - Cursor-based navigation
  - Optional coordinate display (A-H, 1-8)

## Controls

### Universal
| Key | Action |
|-----|--------|
| F1 | Open menu |
| F4 | Exit / Back |

### During Game
| Key | Action |
|-----|--------|
| Arrow Keys | Move cursor |
| Enter | Place disc |
| F2 | Show hint |
| U | Undo (if enabled) |

### Game Over
| Key | Action |
|-----|--------|
| Enter | New game (same mode) |
| W | Enter What If mode |
| N | Select new game mode |

### What If Mode
| Key | Action |
|-----|--------|
| Left/Right | Step through history |
| Arrow Keys | Move cursor (when branching) |
| Enter | Play alternate move |

## Rules

Othello is played on an 8x8 board. Players take turns placing discs of their color. When you place a disc, any opponent discs that are "outflanked" (in a straight line between your new disc and another of your discs) are flipped to your color.

You must make a move that flips at least one disc. If you cannot, you pass. The game ends when neither player can move. The player with the most discs wins.

## Building

This app is designed to be built as part of the Xous operating system:

```bash
# From xous-core directory
cargo xtask app-image othello

# For Renode emulator
cargo xtask renode-image othello
```

## Technical Details

- **Display**: 336x536 pixels, 1-bit (black/white)
- **Board**: 304x304 pixels with 38px cells (272x272 with coordinates)
- **AI**: Minimax with alpha-beta pruning, iterative deepening
- **Storage**: PDDB encrypted key-value store

## Architecture

```
precursor-othello/
├── src/
│   ├── main.rs      # Entry point and event loop
│   ├── app.rs       # Application state machine
│   ├── ui.rs        # Drawing functions
│   ├── menu.rs      # F1 menu system
│   ├── help.rs      # Help screen content
│   ├── storage.rs   # PDDB persistence
│   ├── review.rs    # What If mode logic
│   ├── feedback.rs  # Vibration/haptics
│   └── export.rs    # TCP game export
└── libs/othello-core/
    ├── src/
    │   ├── lib.rs      # Public API
    │   ├── board.rs    # Bitboard representation
    │   ├── moves.rs    # Move generation
    │   ├── game.rs     # Game state with history
    │   ├── ai.rs       # AI search algorithms
    │   ├── eval.rs     # Position evaluation
    │   └── opening.rs  # Opening book
    └── tests/          # Unit tests
```

## License

MIT

## Author

Tyler Colby
