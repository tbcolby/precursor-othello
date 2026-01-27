# Othello for Precursor: Implementation Plan

## Overview

A tournament-quality Othello game for the Precursor hardware platform. Features a strong AI opponent with multiple difficulty levels, full game analysis with "What If" branching mode, and comprehensive game state management.

| | |
|---|---|
| **Target** | App #5 in the Precursor lineup |
| **Repo** | `precursor-othello` |
| **Library** | `libs/othello-core` (host-testable engine) |
| **Author** | Tyler Colby |
| **Status** | Planning Complete |

---

## Hardware Constraints

| Component | Spec | Impact |
|-----------|------|--------|
| Display | 336×536, 1-bit | Perfect for black/white discs |
| CPU | 100MHz RISC-V | AI depth limited, need efficient bitboards |
| RAM | ~4-8 MiB available | Board state trivial, bound transposition tables |
| Input | Physical keyboard | Arrow keys + Enter, F1-F4 standards |
| Storage | PDDB encrypted | Settings, stats, save game |

---

## Display Layout

### Main Game View (Default - No Coordinates)

```
┌───────────────────────────────────┐
│ OTHELLO         ● 02      ○ 02   │  <- Header (24px)
├───────────────────────────────────┤
│                                   │
│    ┌──┬──┬──┬──┬──┬──┬──┬──┐     │
│    │  │  │  │  │  │  │  │  │     │
│    ├──┼──┼──┼──┼──┼──┼──┼──┤     │
│    │  │  │  │  │  │  │  │  │     │
│    ├──┼──┼──┼──┼──┼──┼──┼──┤     │
│    │  │  │  │  │  │  │  │  │     │
│    ├──┼──┼──┼──┼──┼──┼──┼──┤     │    Board: 304×304
│    │  │  │  │● │○ │  │  │  │     │    (38px cells, centered)
│    ├──┼──┼──┼──┼──┼──┼──┼──┤     │
│    │  │  │  │○ │● │  │  │  │     │
│    ├──┼──┼──┼──┼──┼──┼──┼──┤     │
│    │  │  │  │  │  │  │  │  │     │
│    ├──┼──┼──┼──┼──┼──┼──┼──┤     │
│    │  │  │  │  │  │  │  │  │     │
│    ├──┼──┼──┼──┼──┼──┼──┼──┤     │
│    │  │  │  │  │  │  │  │  │     │
│    └──┴──┴──┴──┴──┴──┴──┴──┘     │
│                                   │
│ ● 4 moves  ○ 4 moves  Last: --   │  <- Mobility + last move
│ Your move (●)                    │  <- Status
│                                   │
├───────────────────────────────────┤
│ F1 Menu                  F4 Exit │  <- Footer (24px)
└───────────────────────────────────┘
```

### With Coordinates (Toggle via Settings)

```
┌───────────────────────────────────┐
│ OTHELLO         ● 02      ○ 02   │
├───────────────────────────────────┤
│       A  B  C  D  E  F  G  H      │
│    ┌──┬──┬──┬──┬──┬──┬──┬──┐     │
│  1 │  │  │  │  │  │  │  │  │     │
│    ├──┼──┼──┼──┼──┼──┼──┼──┤     │    Board: 272×272
│  2 │  │  │  │  │  │  │  │  │     │    (34px cells)
│    ├──┼──┼──┼──┼──┼──┼──┼──┤     │
│  3 │  │  │  │  │  │  │  │  │     │
│    ├──┼──┼──┼──┼──┼──┼──┼──┤     │
│  4 │  │  │  │● │○ │  │  │  │     │
│    ├──┼──┼──┼──┼──┼──┼──┼──┤     │
│  5 │  │  │  │○ │● │  │  │  │     │
│    ├──┼──┼──┼──┼──┼──┼──┼──┤     │
│  6 │  │  │  │  │  │  │  │  │     │
│    ├──┼──┼──┼──┼──┼──┼──┼──┤     │
│  7 │  │  │  │  │  │  │  │  │     │
│    ├──┼──┼──┼──┼──┼──┼──┼──┤     │
│  8 │  │  │  │  │  │  │  │  │     │
│    └──┴──┴──┴──┴──┴──┴──┴──┘     │
│                                   │
│ ● 4 moves  ○ 4 moves  Last: --   │
│ Your move (●)   Cursor: D4       │  <- Shows cursor notation
│                                   │
├───────────────────────────────────┤
│ F1 Menu                  F4 Exit │
└───────────────────────────────────┘
```

### Pixel Measurements

| Element | Default | With Coords |
|---------|---------|-------------|
| Screen | 336 × 536 | 336 × 536 |
| Header | 24px | 24px |
| Footer | 24px | 24px |
| Content area | 488px vertical | 488px vertical |
| Cell size | 38px | 34px |
| Board size | 304 × 304 | 272 × 272 |
| Horizontal margin | 16px each side | 16px + 16px labels |
| Board Y position | 24 + 62 = 86px | 24 + 46 = 70px |
| Status area height | ~60px | ~60px |
| Disc diameter | 32px (84%) | 28px (82%) |
| Valid move dot | 8px | 6px |
| Cursor border | 3px thick | 3px thick |
| Last move marker | 5px corner dots | 4px corner dots |

### Layout Calculations

```
Screen width:     336px
Board width:      304px (38 × 8)
Margin each side: (336 - 304) / 2 = 16px  ✓ Clean margins

Content height:   536 - 24 - 24 = 488px
Board height:     304px
Status area:      60px (two lines of text + padding)
Vertical space:   488 - 304 - 60 = 124px
Board Y offset:   24 + (124 / 2) = 24 + 62 = 86px from top
```

---

## Visual Elements

### Disc Rendering

```
Normal disc (filled circle):
  ┌─────────┐
  │  ┌───┐  │
  │ │█████│ │   ● = PixelColor::Dark fill
  │  └───┘  │   ○ = PixelColor::Light fill + Dark border (2px)
  └─────────┘

Disc size: 32px diameter in 38px cell
```

### Cursor Highlight

```
Thick border around selected cell:
  ╔═══════╗
  ║       ║   3px dark border
  ║   ●   ║   (or inverted if on occupied cell)
  ║       ║
  ╚═══════╝
```

### Valid Move Indicator

```
Small centered dot in empty valid cells:
  ┌───────┐
  │       │
  │   ·   │   8px dark filled circle
  │       │
  └───────┘
```

### Last Move Highlight

```
Corner markers on the last-played cell:
  ┌─■───■─┐
  │       │   Small 5px squares in corners
  │   ●   │
  │       │
  └─■───■─┘
```

### Danger Zone Indicators (Toggle)

When enabled, X-squares and C-squares near empty corners show subtle warning:

```
X-square (diagonal to corner):     C-square (adjacent to corner):
  ┌───────┐                          ┌───────┐
  │ ╲   ╱ │   Diagonal lines         │   ─   │   Single line
  │   ╳   │   in corners             │   │   │   toward corner
  │ ╱   ╲ │                          │       │
  └───────┘                          └───────┘
```

### Flip Animation (Toggle)

Sequence over ~120ms total:
1. Disc shrinks horizontally (40ms)
2. Disc at minimum width, color swaps (instant)
3. Disc expands to full size (40ms)

All flipped discs animate simultaneously.

### AI Thinking Animation (Toggle)

When enabled, show animated indicator:
```
CPU thinking .     (cycle every 300ms)
CPU thinking ..
CPU thinking ...
CPU thinking .
```

---

## State Machine

```
                         ┌─────────────┐
                         │  MainMenu   │
                         └──────┬──────┘
                                │
          ┌─────────────────────┼─────────────────────┐
          ▼                     ▼                     ▼
   ┌─────────────┐      ┌─────────────┐      ┌─────────────┐
   │ NewGameMenu │      │   Resume    │      │    Stats    │
   │             │      │ (if exists) │      │             │
   └──────┬──────┘      └──────┬──────┘      └─────────────┘
          │                    │
          ▼                    ▼
   ┌─────────────────────────────────────────────────────┐
   │                      Playing                        │
   │  ┌───────────┐      ┌───────────┐      ┌─────────┐  │
   │  │  Player   │ ───▶ │   Pass    │ ───▶ │   AI    │  │
   │  │   Turn    │      │  Check    │      │  Turn   │  │
   │  └─────┬─────┘      └───────────┘      └────┬────┘  │
   │        │                                    │       │
   │        └────────────────────────────────────┘       │
   │                         │                           │
   │              (game over check)                      │
   └─────────────────────────┼───────────────────────────┘
                             │
                             ▼
                      ┌─────────────┐
                      │  GameOver   │
                      └──────┬──────┘
                             │
                             ▼
                      ┌─────────────┐
                      │   What If   │  (optional, from GameOver or menu)
                      │   Review    │
                      └─────────────┘
```

### App States

```rust
pub enum AppState {
    MainMenu {
        cursor: usize,
    },
    NewGameMenu {
        cursor: usize,
    },
    SettingsMenu {
        cursor: usize,
    },
    Statistics,
    Playing {
        game: GameState,
        mode: GameMode,
        player_color: Player,        // Random assignment for vs CPU
        cursor_pos: (u8, u8),        // Board cursor (0-7, 0-7)
        ai_thinking: bool,
        thinking_dots: u8,           // Animation state (0-3)
        show_pass_notice: bool,
    },
    GameOver {
        game: GameState,
        mode: GameMode,
        player_color: Player,
    },
    WhatIf {
        base_game: GameState,
        current_game: GameState,
        view_index: usize,
        branched: bool,
        cursor_pos: (u8, u8),
    },
    MoveHistory {
        game: GameState,
        scroll_offset: usize,
    },
    Help {
        context: HelpContext,
    },
}

pub enum GameMode {
    VsCpu(Difficulty),
    TwoPlayer,
}

pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Expert,
}

pub enum HelpContext {
    MainMenu,
    Playing,
    WhatIf,
}
```

---

## Player Color Assignment

### vs CPU Mode

At game start:
1. **Random coin flip** using hardware TRNG
2. Player assigned Black (moves first) or White (moves second)
3. CPU gets the opposite color
4. UI adapts: "Your move (●)" or "Your move (○)"

```rust
fn assign_player_color() -> Player {
    // Use Xous TRNG for true randomness
    let random_bit = trng::get_u32() & 1;
    if random_bit == 0 {
        Player::Black
    } else {
        Player::White
    }
}
```

### Display Adaptation

When player is White (CPU moves first):
```
│ CPU's move (●)                   │  <- CPU plays black, moves first
```

Then after CPU moves:
```
│ Your move (○)                    │  <- Player plays white
```

### Two-Player Mode

- Black always moves first (standard Othello rules)
- Display shows: "Black's move (●)" / "White's move (○)"

---

## Menu System

### F1 Menu: Main Menu State

```
┌───────────────────────┐
│   > Help              │
│     New Game          │
│     Resume Game       │  <- Only if save exists
│     Statistics        │
│     Settings          │
├───────────────────────┤
│     F4 to close       │
└───────────────────────┘
```

### F1 Menu: Playing State

```
┌───────────────────────┐
│   > Help              │
│     Move History      │
│     Hint              │
│     Undo              │  <- If enabled
│     Resign            │
│     Save & Exit       │
│     New Game          │
├───────────────────────┤
│     F4 to close       │
└───────────────────────┘
```

### F1 Menu: Game Over State

```
┌───────────────────────┐
│   > Help              │
│     What If           │
│     Move History      │
│     Export Game       │  <- If enabled
│     New Game          │
│     Main Menu         │
├───────────────────────┤
│     F4 to close       │
└───────────────────────┘
```

### F1 Menu: What If State

```
┌───────────────────────┐
│   > Help              │
│     Reset to Move     │
│     Reset to Start    │
│     Exit What If      │
├───────────────────────┤
│     F4 to close       │
└───────────────────────┘
```

### New Game Submenu

```
┌───────────────────────┐
│      New Game         │
├───────────────────────┤
│   > vs CPU Easy       │
│     vs CPU Medium     │
│     vs CPU Hard       │
│     vs CPU Expert     │
│     ─────────────     │
│     Two Players       │
├───────────────────────┤
│     F4 to cancel      │
└───────────────────────┘
```

### Settings Submenu

```
┌─────────────────────────┐
│       Settings          │
├─────────────────────────┤
│   Show Coords      [ ]  │  <- Toggle
│   Show Valid       [×]  │  <- Toggle
│   Allow Undo       [×]  │  <- Toggle
│   Danger Zones     [ ]  │  <- Toggle
│   Export Games     [ ]  │  <- Toggle
│   ───────────────────   │
│   Flip Animation   [×]  │  <- Toggle
│   AI Think Anim    [×]  │  <- Toggle
│   AI Delay         [×]  │  <- Toggle
│   ───────────────────   │
│   Vibration        [×]  │  <- Toggle
│   Sound            [×]  │  <- Toggle
├─────────────────────────┤
│      F4 to close        │
└─────────────────────────┘
```

---

## Help Screens

### Main Menu Help

```
┌───────────────────────────────────┐
│          OTHELLO v1.0             │
├───────────────────────────────────┤
│                                   │
│ Classic Reversi strategy game.    │
│ Outflank your opponent's discs    │
│ to flip them to your color.       │
│                                   │
│ Controls:                         │
│                                   │
│ F1        Menu                    │
│ F4        Exit                    │
│ ↑↓        Navigate                │
│ Enter     Select                  │
│                                   │
│ The player with the most discs    │
│ when the board is full wins!      │
│                                   │
│ In vs CPU mode, your color is     │
│ randomly assigned each game.      │
│                                   │
├───────────────────────────────────┤
│        Press any key              │
└───────────────────────────────────┘
```

### Playing Help

```
┌───────────────────────────────────┐
│       OTHELLO - Playing           │
├───────────────────────────────────┤
│                                   │
│ Controls:                         │
│                                   │
│ F1        Menu                    │
│ F4        Save & Exit             │
│ F2        Show Hint               │
│ F3        Pass (when no moves)    │
│                                   │
│ ↑↓←→      Move cursor             │
│ Enter     Place disc              │
│ H         Toggle hint display     │
│ U         Undo last move          │
│                                   │
│ Legend:                           │
│ [█]  Your cursor                  │
│  ·   Valid move                   │
│  ■   Last move played             │
│                                   │
├───────────────────────────────────┤
│        Press any key              │
└───────────────────────────────────┘
```

### What If Help

```
┌───────────────────────────────────┐
│       OTHELLO - What If           │
├───────────────────────────────────┤
│                                   │
│ Review and explore alternate      │
│ moves from any point in the game. │
│                                   │
│ Controls:                         │
│                                   │
│ F1        Menu                    │
│ F4        Exit What If            │
│                                   │
│ ←→        Step back/forward       │
│ Home      Jump to start           │
│ End       Jump to end             │
│                                   │
│ ↑↓        Move cursor             │
│ Enter     Play alternate move     │
│           (branches the game)     │
│                                   │
│ Once you branch, continue playing │
│ to explore "what if" scenarios.   │
│                                   │
├───────────────────────────────────┤
│        Press any key              │
└───────────────────────────────────┘
```

---

## Game Over Screen

```
┌───────────────────────────────────┐
│ OTHELLO         ● 38      ○ 26   │
├───────────────────────────────────┤
│                                   │
│    ┌──┬──┬──┬──┬──┬──┬──┬──┐     │
│    │ (final board state shown)   │
│    ...                           │
│    └──┴──┴──┴──┴──┴──┴──┴──┘     │
│                                   │
│     ┌─────────────────────┐      │
│     │                     │      │
│     │     YOU WIN!        │      │
│     │                     │      │
│     │   ● 38  -  ○ 26     │      │
│     │                     │      │
│     └─────────────────────┘      │
│                                   │
│ Enter: New Game   W: What If     │
│                                   │
├───────────────────────────────────┤
│ F1 Menu                  F4 Exit │
└───────────────────────────────────┘
```

Result text variations:
- "YOU WIN!" (player wins vs CPU)
- "CPU WINS!" (CPU wins)
- "DRAW!" (equal disc count)
- "BLACK WINS!" / "WHITE WINS!" (two-player mode)

---

## What If Mode

### Display

```
┌───────────────────────────────────┐
│ WHAT IF           Move 24/58     │
├───────────────────────────────────┤
│                                   │
│    ┌──┬──┬──┬──┬──┬──┬──┬──┐     │
│    │ (board at move 24)          │
│    ...                           │
│    └──┴──┴──┴──┴──┴──┴──┴──┘     │
│                                   │
│ ● 18  ○ 14       Empty: 32       │
│                                   │
│ ←/→ Step    Enter: Play alternate│
│                                   │
├───────────────────────────────────┤
│ F1 Menu                  F4 Exit │
└───────────────────────────────────┘
```

### Branched State Display

```
┌───────────────────────────────────┐
│ WHAT IF (BRANCHED)  Move 24+3    │
├───────────────────────────────────┤
│      ...                          │
│                                   │
│ Playing alternate timeline...    │
│ ↑↓←→ + Enter to continue         │
│                                   │
├───────────────────────────────────┤
│ F1 Menu                  F4 Exit │
└───────────────────────────────────┘
```

### Behavior

1. **Enter What If** from Game Over or F1 menu during play
2. **Navigate history** with ←/→ keys, Home/End to jump
3. **View board state** at any move
4. **Branch** by making an alternate move (Enter on a different square)
5. **Continue playing** the alternate timeline
6. **Reset** via F1 menu to return to original game
7. **Exit** via F4 returns to previous state

---

## Move History Screen

```
┌───────────────────────────────────┐
│         MOVE HISTORY              │
├───────────────────────────────────┤
│                                   │
│  #   ●        ○                   │
│  1.  D3       C3                  │
│  2.  C4       E3                  │
│  3.  C2       B3                  │
│  4.  D2       C5                  │
│  5.  F4       E2                  │
│  6.  F5       F3                  │
│  7.  E6       D6                  │
│  8.  C6       F6                  │
│  9.  G5       G4                  │
│ 10.  G3       ...                 │
│                                   │
│ ↑↓ Scroll      Total: 58 moves   │
│                                   │
├───────────────────────────────────┤
│ F1 Menu                  F4 Back │
└───────────────────────────────────┘
```

---

## Statistics Screen

```
┌───────────────────────────────────┐
│           STATISTICS              │
├───────────────────────────────────┤
│                                   │
│ vs CPU Easy                       │
│   Won: 12    Lost: 3    Draw: 1   │
│                                   │
│ vs CPU Medium                     │
│   Won: 8     Lost: 5    Draw: 0   │
│                                   │
│ vs CPU Hard                       │
│   Won: 4     Lost: 9    Draw: 2   │
│                                   │
│ vs CPU Expert                     │
│   Won: 1     Lost: 14   Draw: 0   │
│                                   │
│ Two Player Games: 7               │
│                                   │
│                                   │
├───────────────────────────────────┤
│ F1 Menu                  F4 Back │
└───────────────────────────────────┘
```

---

## Pass Notification

When a player has no legal moves:

```
┌───────────────────────────────────┐
│                                   │
│     ┌─────────────────────┐      │
│     │                     │      │
│     │   No legal moves!   │      │
│     │                     │      │
│     │   Passing to ○      │      │
│     │                     │      │
│     │   Press any key     │      │
│     │                     │      │
│     └─────────────────────┘      │
│                                   │
└───────────────────────────────────┘
```

Auto-dismiss after 2 seconds OR on any key press.

---

## Endgame Projection

In AI modes, when ≤14 empty squares remain, show projected outcome:

```
│ ● 7 moves  ○ 4 moves  Last: E6   │
│ Your move (●)  Final: ● 35 - 29  │  <- Perfect play projection
```

Only shown when the AI has solved the position to completion.

---

## Feedback Systems

### Vibration (Toggle)

When enabled:
- **Disc placement**: Short pulse (50ms)
- **Invalid move attempt**: Double pulse
- **Game over**: Long pulse (200ms)
- **Your turn** (after CPU moves): Short pulse

```rust
// Using LLIO vibration API
llio.vibe(VibePattern::Short);   // 50ms
llio.vibe(VibePattern::Double);  // Invalid
llio.vibe(VibePattern::Long);    // Game over
```

### Sound (Toggle)

When enabled:
- **Disc placement**: Click sound
- **Flip**: Soft pop
- **Invalid move**: Low buzz
- **Game over win**: Ascending tone
- **Game over loss**: Descending tone

*Note: Requires investigation of Precursor audio capabilities*

---

## Key Bindings

### Universal (All States)

| Key | Action |
|-----|--------|
| F1 | Open context menu |
| F4 | Exit/Back (with confirmation if needed) |

### Main Menu

| Key | Action |
|-----|--------|
| ↑↓ | Navigate menu |
| Enter | Select item |
| Q | Quit app |

### Playing

| Key | Action |
|-----|--------|
| F2 | Show hint (AI best move) |
| F3 | Pass turn (only when must pass) |
| ↑↓←→ | Move cursor |
| Enter | Place disc |
| H | Toggle hint overlay |
| U | Undo last move (if enabled) |

### Game Over

| Key | Action |
|-----|--------|
| Enter | New game (same mode) |
| W | Enter What If mode |
| N | New game (select mode) |

### What If

| Key | Action |
|-----|--------|
| ←→ | Step backward/forward in history |
| Home | Jump to game start |
| End | Jump to game end |
| ↑↓ | Move cursor (when branching) |
| Enter | Play alternate move (branches) |

### Move History

| Key | Action |
|-----|--------|
| ↑↓ | Scroll list |
| Home | Jump to start |
| End | Jump to end |

### Settings

| Key | Action |
|-----|--------|
| ↑↓ | Navigate options |
| Enter/Space | Toggle selected option |

---

## AI System

### Difficulty Levels

| Level | Search Depth | Evaluation | Time Budget | Strength |
|-------|--------------|------------|-------------|----------|
| Easy | 2 ply | Disc count only | Instant | Beginner |
| Medium | 4 ply | Basic positional | <500ms | Club player |
| Hard | 6 ply + endgame | Full evaluation | 2s | Strong amateur |
| Expert | Iterative deepening + book | Full + mobility | 3s | Tournament |

### Evaluation Function Components

#### 1. Corner Control (Weight: Very High)

```
Corners (positions 0, 7, 56, 63):
  Own corner:      +100
  Opponent corner: -100

X-squares (diagonal to empty corner):
  Positions 9, 14, 49, 54 when adjacent corner empty:
  Own disc:        -25
  Opponent disc:   +25

C-squares (adjacent to empty corner):
  Positions 1, 6, 8, 15, 48, 55, 57, 62 when adjacent corner empty:
  Own disc:        -10
  Opponent disc:   +10
```

#### 2. Edge Stability (Weight: High)

```
Stable disc (cannot be flipped):     +10
Semi-stable disc:                     +5
Unstable edge disc:                    0
```

#### 3. Mobility (Weight: Medium-High)

```
Current player legal moves:    +3 per move
Opponent legal moves:          -3 per move
```

#### 4. Frontier Discs (Weight: Medium)

```
Discs adjacent to empty squares:  -1 per disc
(Fewer frontier discs = better)
```

#### 5. Disc Count (Weight: Varies)

```
Early game (>44 empty):   Weight near 0 (ignore disc count)
Mid game (20-44 empty):   Weight 0.5
Late game (<20 empty):    Weight 2.0
Endgame (<10 empty):      Weight 5.0 (primary factor)
```

#### 6. Parity (Weight: Late Game)

```
When odd number of empty regions:
  Player to move in odd region: +10
```

### Endgame Solver

When ≤14 empty squares:
- Switch to perfect minimax (no depth limit)
- Alpha-beta with move ordering
- Solve to completion
- Cache result for projection display

Expected performance on 100MHz:
- 14 empty: ~2-3 seconds worst case
- 12 empty: ~500ms
- 10 empty: ~100ms

### Opening Book (Expert Mode)

Standard openings stored as position → best move mappings:

```rust
struct OpeningBook {
    entries: HashMap<u64, u8>,  // board_hash -> best_move_pos
}
```

~50 positions covering:
- Diagonal opening
- Perpendicular opening  
- Parallel opening
- Common responses through move 10-12

Size: ~2KB embedded in binary.

### AI Thinking Indicator

When AI Think Anim setting is enabled:
1. Show "CPU thinking..." in status
2. Animate ellipsis: `.` → `..` → `...` → `.` (300ms cycle)

When AI Delay setting is enabled:
- Minimum 500ms before AI move (even if instant)
- Gives player time to see the board

---

## Data Persistence (PDDB)

### Dictionary: `othello.settings`

```rust
pub struct Settings {
    pub show_coordinates: bool,    // default: false
    pub show_valid_moves: bool,    // default: true
    pub allow_undo: bool,          // default: true
    pub danger_zones: bool,        // default: false
    pub export_enabled: bool,      // default: false
    pub flip_animation: bool,      // default: true
    pub ai_think_animation: bool,  // default: true
    pub ai_delay: bool,            // default: true
    pub vibration: bool,           // default: true
    pub sound: bool,               // default: true
    pub last_difficulty: u8,       // 0-3 (Easy-Expert)
}

// Serialization: 11 bytes, one per field
```

### Dictionary: `othello.stats`

```rust
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

// Serialization: 26 bytes (13 × u16 little-endian)
```

### Dictionary: `othello.save`

```rust
pub struct SaveGame {
    pub board_black: u64,          // Black disc bitboard
    pub board_white: u64,          // White disc bitboard
    pub current_player: u8,        // 0 = Black, 1 = White
    pub player_color: u8,          // Which color the human plays (vs CPU)
    pub mode: u8,                  // 0-3 = CPU Easy-Expert, 4 = TwoPlayer
    pub move_count: u16,           // Number of moves in history
    pub history: Vec<(u8, u64)>,   // (position, flipped_bitboard) pairs
}

// Serialization: 20 + (move_count × 9) bytes
// Max game ~60 moves = ~560 bytes
```

---

## Export Format

When export is enabled, games can be exported via TCP (port 7880):

```
[Othello Game Record]
Date: 2026-01-27
Mode: vs CPU Hard
Player: White
Result: Player wins 38-26

Moves:
1. D3 C3
2. C4 E3
3. C2 B3
...
29. H8 --
30. G8

Final: ● 26 - ○ 38
```

Or compact notation only:
```
D3 C3 C4 E3 C2 B3 D2 C5 F4 E2 ...
```

---

## Module Structure

```
precursor-othello/
├── Cargo.toml
├── README.md
├── PLAN.md                  # This document
└── src/
    ├── main.rs              # Entry point, event loop, Xous integration
    ├── app.rs               # OthelloApp struct, state machine transitions
    ├── ui.rs                # All drawing functions (board, menus, dialogs)
    ├── menu.rs              # Menu definitions and navigation logic
    ├── help.rs              # Help screen content for each context
    ├── storage.rs           # PDDB read/write for settings, stats, saves
    ├── review.rs            # What If mode state management
    ├── feedback.rs          # Vibration and sound handling
    └── export.rs            # TCP export functionality

libs/othello-core/
├── Cargo.toml
├── src/
│   ├── lib.rs               # Public API exports
│   ├── board.rs             # Bitboard representation and operations
│   ├── moves.rs             # Move generation, validation, flip calculation
│   ├── game.rs              # GameState with history and turn management
│   ├── ai.rs                # Minimax, alpha-beta, iterative deepening
│   ├── eval.rs              # Position evaluation function
│   └── opening.rs           # Opening book data and lookup
└── tests/
    ├── board_tests.rs       # Bitboard operation correctness
    ├── flip_tests.rs        # Flip calculation for all 8 directions
    ├── perft_tests.rs       # Move generation node count verification
    ├── eval_tests.rs        # Known position evaluation checks
    ├── endgame_tests.rs     # Perfect play verification
    └── game_tests.rs        # GameState logic and history
```

---

## Implementation Phases

### Phase 1: Core Engine (`othello-core`)

**Goal:** Complete, tested game logic library

1. `board.rs` - Bitboard type with basic operations
2. `moves.rs` - Move generation with flip calculation
3. `game.rs` - GameState with history tracking
4. Unit tests for all of the above
5. Perft tests (move generation verification)

**Deliverable:** Library that can play a complete game via tests

### Phase 2: Basic UI

**Goal:** Playable two-player game on Precursor

1. `main.rs` - Xous app skeleton with event loop
2. `ui.rs` - Board rendering, disc drawing
3. `app.rs` - Basic Playing state
4. Cursor movement and disc placement
5. Turn switching, game over detection

**Deliverable:** Two humans can play Othello

### Phase 3: AI Integration

**Goal:** Single-player vs CPU

1. `eval.rs` - Position evaluation function
2. `ai.rs` - Minimax with alpha-beta
3. Easy/Medium difficulty working
4. Hard difficulty with endgame solver
5. Expert with iterative deepening
6. Random player color assignment

**Deliverable:** All 4 difficulty levels playable

### Phase 4: Full UI/UX

**Goal:** Complete menu system and polish

1. `menu.rs` - F1 menus for all states
2. `help.rs` - Context-sensitive help screens
3. Settings menu with all toggles
4. Statistics screen
5. Pass notification modal
6. Game over screen with results
7. Flip animation (if enabled)
8. Valid move indicators
9. Last move highlighting
10. Hint system (F2)
11. Undo functionality
12. `feedback.rs` - Vibration and sound

**Deliverable:** Full-featured game matching spec

### Phase 5: Persistence & What If

**Goal:** Save/load and game analysis

1. `storage.rs` - PDDB integration
2. Settings persistence
3. Statistics tracking and persistence
4. Save/resume game
5. `review.rs` - What If mode
6. Move history screen
7. Opening book for Expert

**Deliverable:** Complete game with all features

### Phase 6: Polish & Export

**Goal:** Final touches

1. `export.rs` - TCP game record export
2. Danger zone indicators (X/C squares)
3. Coordinate display toggle
4. Endgame projection display
5. AI thinking animation
6. Performance optimization
7. README documentation
8. Renode testing automation

**Deliverable:** Ship-ready App #5

---

## Testing Strategy

### Unit Tests (othello-core)

```rust
// board_tests.rs
#[test]
fn test_starting_position() {
    let board = Board::new();
    assert_eq!(board.count(Player::Black), 2);
    assert_eq!(board.count(Player::White), 2);
}

// flip_tests.rs
#[test]
fn test_flip_horizontal() {
    // Set up position, verify correct discs flip
}

// perft_tests.rs - move generation node counts
#[test]
fn test_perft_depth_1() {
    let game = GameState::new();
    assert_eq!(perft(&game, 1), 4);  // 4 legal opening moves
}

#[test]
fn test_perft_depth_5() {
    let game = GameState::new();
    assert_eq!(perft(&game, 5), KNOWN_PERFT_5);  // From literature
}

// endgame_tests.rs
#[test]
fn test_endgame_solve() {
    // Known endgame position, verify perfect play result
}
```

### Integration Tests (Renode)

Using `renode_capture.py` pattern:
1. Build image with `cargo xtask renode-image othello`
2. Boot Renode, unlock PDDB
3. Navigate to Othello app
4. Automated game play via key injection
5. Screenshot capture at key states
6. Verify final score

### Manual Test Checklist

- [ ] All F1 menus accessible from each state
- [ ] F4 exits correctly with confirmations
- [ ] All difficulty levels complete a game
- [ ] Two-player mode works
- [ ] Random color assignment working (play 10 games, verify mix)
- [ ] Save/resume preserves exact state
- [ ] Statistics update correctly
- [ ] Settings persist across app restarts
- [ ] What If navigation works
- [ ] What If branching works
- [ ] Undo works at all points
- [ ] Pass detection correct
- [ ] Game over detection correct
- [ ] Flip animation smooth (if enabled)
- [ ] Vibration triggers correctly (if enabled)
- [ ] Sound triggers correctly (if enabled)
- [ ] Export produces valid game record

---

## Dependencies

### precursor-othello/Cargo.toml

```toml
[package]
name = "othello"
version = "0.1.0"
authors = ["Tyler Colby"]
edition = "2021"
description = "Tournament-quality Othello with AI opponent for Precursor"

[dependencies]
# Core Xous
xous = "0.9.69"
xous-ipc = "0.10.9"
log = "0.4.14"
log-server = { package = "xous-api-log", version = "0.1.68" }
xous-names = { package = "xous-api-names", version = "0.9.70" }

# Graphics
gam = { path = "../../services/gam" }

# Timing
ticktimer-server = { package = "xous-api-ticktimer", version = "0.9.68" }

# Storage
pddb = { path = "../../services/pddb" }

# Hardware (vibration, TRNG)
llio = { path = "../../services/llio" }
trng = { path = "../../services/trng" }

# Enum serialization
num-derive = { version = "0.4.2", default-features = false }
num-traits = { version = "0.2.14", default-features = false }

# Game engine
othello-core = { path = "../../libs/othello-core" }
```

### libs/othello-core/Cargo.toml

```toml
[package]
name = "othello-core"
version = "0.1.0"
authors = ["Tyler Colby"]
edition = "2021"
description = "Othello game engine with AI"

[dependencies]
# None! Pure Rust, no_std compatible

[dev-dependencies]
# For host testing
rand = "0.8"
```

---

## File Manifest

After implementation, the repo should contain:

```
precursor-othello/
├── Cargo.toml
├── README.md
├── PLAN.md
├── screenshots/
│   ├── main_menu.png
│   ├── playing.png
│   ├── game_over.png
│   ├── what_if.png
│   └── settings.png
└── src/
    ├── main.rs          (~200 lines)
    ├── app.rs           (~450 lines)
    ├── ui.rs            (~550 lines)
    ├── menu.rs          (~200 lines)
    ├── help.rs          (~100 lines)
    ├── storage.rs       (~150 lines)
    ├── review.rs        (~200 lines)
    ├── feedback.rs      (~80 lines)
    └── export.rs        (~100 lines)

libs/othello-core/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs           (~50 lines)
    ├── board.rs         (~200 lines)
    ├── moves.rs         (~300 lines)
    ├── game.rs          (~250 lines)
    ├── ai.rs            (~350 lines)
    ├── eval.rs          (~200 lines)
    └── opening.rs       (~100 lines + data)
```

**Estimated total:** ~3,500 lines of Rust

---

## Success Criteria

The app is complete when:

1. **Correctness:** All Othello rules implemented perfectly
2. **AI Quality:** Expert mode provides genuine challenge
3. **Performance:** No perceptible lag on 100MHz (except AI thinking)
4. **UX Polish:** Smooth, intuitive, follows F1/F4 standards
5. **Reliability:** No crashes, proper error handling
6. **Persistence:** Settings/stats/saves work flawlessly
7. **Analysis:** What If mode enables deep game review
8. **Feedback:** Vibration and sound enhance gameplay
9. **Fairness:** Random color assignment works correctly
10. **Documentation:** README covers all features and controls

---

*Plan created: January 27, 2026*
*Updated: January 27, 2026 - Corrected pixel math, added feedback toggles, random player color*
*Ready for implementation*
