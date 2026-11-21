# Othello build notes

- Engine, AI, match flow (`Match`), save/settings/stats encodings and `WhatIf` are in `libs/othello-core` (host-tested; run `cargo test` there). Keep `moves.rs` direction masks as pre-shift masks; the perft test guards them.
- Shell: `src/main.rs` routes GAM messages; `src/app.rs` maps `Key` to core operations and draws via `appkit::screen::Screen`; `src/ai.rs` runs one thread per search and posts `AppOp::AiDone` (stale generations ignored).
- Every screen exits on Backspace/F4. `∴` is never delivered. Leave with `kit::leave`, never `terminate_process`.
- Gate: `cd ../xous-dev-toolkit && bin/integrate --xous-root <xous-core> --grade -v ../precursor-othello`
