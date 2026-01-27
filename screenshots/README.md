# Screenshots

Screenshots are captured from the Renode emulator after integrating the app into xous-core.

## Capturing Screenshots

Once Othello is integrated into xous-core, capture screenshots with:

```bash
# Build the Renode image
cd xous-core
cargo xtask renode-image othello

# Run the capture script
python3 /path/to/xous-dev-toolkit/scripts/renode_capture.py \
    --app othello \
    --app-index N \
    --screenshots /path/to/precursor-othello/screenshots

# Where N is Othello's position in the app submenu (alphabetical order)
```

## Expected Screenshots

| File | Description |
|------|-------------|
| `main_menu.png` | Initial main menu screen |
| `new_game_menu.png` | Difficulty selection |
| `playing.png` | Active game with board and cursor |
| `game_over.png` | Game over with final score |
| `what_if.png` | What If mode reviewing game |
| `settings.png` | Settings menu |
| `statistics.png` | Win/loss statistics |
