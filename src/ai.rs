//! Search worker: runs the AI on its own thread so the event loop keeps
//! servicing redraws and keys, and so a search can be cancelled when the app
//! goes to the background.
//!
//! One short-lived thread per search. The result goes back to the main server
//! as a non-blocking scalar `(position | 255, generation, is_hint)`; the app
//! drops results whose generation is stale.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use othello_core::{find_best_move_limited, Board, Difficulty, Limits, Player};
use xous::{Message, CID};

/// Wall-clock cap per search. Untested on hardware: tune once measured.
fn time_budget(d: Difficulty) -> Duration {
    Duration::from_secs(match d {
        Difficulty::Easy => 2,
        Difficulty::Medium => 4,
        Difficulty::Hard => 8,
        Difficulty::Expert => 15,
    })
}

pub struct Worker {
    main_cid: CID,
    opcode: usize,
    cancel: Arc<AtomicBool>,
    generation: usize,
}

impl Worker {
    pub fn new(main_cid: CID, opcode: usize) -> Self {
        Self { main_cid, opcode, cancel: Arc::new(AtomicBool::new(false)), generation: 0 }
    }

    /// Abandon any search in flight; its result (if it still arrives) is stale.
    pub fn cancel(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
        self.generation = self.generation.wrapping_add(1);
    }

    pub fn is_current(&self, generation: usize) -> bool { generation == self.generation }

    /// Start a search. Cancels the previous one. Returns nothing; the answer
    /// arrives as an `AiDone` message.
    pub fn start(&mut self, board: Board, player: Player, difficulty: Difficulty, hint: bool) {
        self.cancel();
        self.cancel = Arc::new(AtomicBool::new(false));
        let cancel = self.cancel.clone();
        let generation = self.generation;
        let (cid, op) = (self.main_cid, self.opcode);
        let spawned = std::thread::Builder::new().name("othello-ai".into()).spawn(move || {
            let started = Instant::now();
            let budget = time_budget(difficulty);
            let stop = || cancel.load(Ordering::Relaxed) || started.elapsed() > budget;
            let limits = Limits { max_nodes: difficulty.node_budget(), stop: &stop };
            let pos = find_best_move_limited(&board, player, difficulty, &limits);
            if cancel.load(Ordering::Relaxed) {
                return; // abandoned: nobody is waiting
            }
            let _ = xous::send_message(
                cid,
                Message::new_scalar(op, pos.map(|p| p as usize).unwrap_or(255), generation, hint as usize, 0),
            );
        });
        if spawned.is_err() {
            log::error!("couldn't start AI thread");
            // Deliver "no move" so the UI doesn't wait forever.
            let _ = xous::send_message(cid, Message::new_scalar(op, 255, generation, hint as usize, 0));
        }
    }
}
