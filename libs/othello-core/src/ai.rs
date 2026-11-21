//! AI opponent with multiple difficulty levels.
//!
//! Negamax with alpha-beta pruning, iterative deepening, an exact endgame
//! solver, and a node/stop budget so a search can always be abandoned: the
//! shell runs it on a worker thread and cancels it when the app is backgrounded.
//!
//! No opening book: the old one never matched a real position (audit O-12).

use crate::eval::{evaluate, Score, SCORE_LOSS, SCORE_WIN};
use crate::moves::{count_moves, generate_moves};
use crate::{Board, MoveList, Player, Position};

/// AI difficulty levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Expert,
}

impl Difficulty {
    /// Maximum search depth in plies (iterative deepening stops earlier when the budget runs out).
    pub const fn depth(&self) -> u8 {
        match self {
            Difficulty::Easy => 2,
            Difficulty::Medium => 4,
            Difficulty::Hard => 6,
            Difficulty::Expert => 8,
        }
    }

    /// Whether to use endgame solving
    pub const fn use_endgame_solver(&self) -> bool {
        matches!(self, Difficulty::Hard | Difficulty::Expert)
    }

    /// Endgame solver threshold (empty squares). Kept small: the solver is
    /// exact, and on a 100 MHz core anything bigger would not finish.
    pub const fn endgame_threshold(&self) -> u32 {
        match self {
            Difficulty::Easy | Difficulty::Medium => 0,
            Difficulty::Hard => 8,
            Difficulty::Expert => 10,
        }
    }

    /// Default node budget for one search at this level.
    pub const fn node_budget(&self) -> u32 {
        match self {
            Difficulty::Easy => 2_000,
            Difficulty::Medium => 20_000,
            Difficulty::Hard => 60_000,
            Difficulty::Expert => 150_000,
        }
    }
}

/// Bounds on one search.
pub struct Limits<'a> {
    /// Abandon the search after this many nodes.
    pub max_nodes: u32,
    /// Polled every 128 nodes; return true to abandon (cancel flag, wall clock).
    pub stop: &'a dyn Fn() -> bool,
}

struct Search<'a> {
    nodes: u32,
    aborted: bool,
    limits: &'a Limits<'a>,
}

impl Search<'_> {
    /// Count a node; returns true if the search must unwind.
    fn tick(&mut self) -> bool {
        self.nodes += 1;
        if self.nodes >= self.limits.max_nodes || (self.nodes % 128 == 0 && (self.limits.stop)()) {
            self.aborted = true;
        }
        self.aborted
    }
}

fn apply_move(board: &Board, player: Player, pos: Position, flipped: u64) -> Board {
    let mut nb = *board;
    nb.place(player, pos);
    nb.flip(player.opponent(), flipped);
    nb
}

const MAXM: usize = crate::moves::MAX_MOVES_PER_POSITION;

/// Order moves for alpha-beta; `with_mobility` adds the (costly) reply count.
fn order_moves(board: &Board, player: Player, moves: &MoveList, with_mobility: bool) -> [usize; MAXM] {
    let mut indices: [usize; MAXM] = core::array::from_fn(|i| i);
    let mut scores: [Score; MAXM] = [0; MAXM];

    for (i, m) in moves.iter().enumerate() {
        let mut score = 0i32;
        if m.pos == 0 || m.pos == 7 || m.pos == 56 || m.pos == 63 {
            score += 1000; // corner
        } else if m.pos == 9 || m.pos == 14 || m.pos == 49 || m.pos == 54 {
            score -= 500; // X-square
        } else if [1, 6, 8, 15, 48, 55, 57, 62].contains(&m.pos) {
            score -= 200; // C-square
        } else if m.pos < 8 || m.pos >= 56 || m.pos % 8 == 0 || m.pos % 8 == 7 {
            score += 100; // edge
        }
        score += m.flip_count() as i32 * 5;
        if with_mobility {
            let nb = apply_move(board, player, m.pos, m.flipped);
            score -= count_moves(&nb, player.opponent()) as i32 * 3;
        }
        scores[i] = score;
    }
    // Insertion sort, descending.
    for i in 1..moves.len() {
        let mut j = i;
        while j > 0 && scores[indices[j]] > scores[indices[j - 1]] {
            indices.swap(j, j - 1);
            j -= 1;
        }
    }
    indices
}

/// Negamax alpha-beta: score from the point of view of `me`, who is to move.
fn negamax(board: &Board, me: Player, depth: u8, mut alpha: Score, beta: Score, s: &mut Search) -> Score {
    if s.tick() {
        return 0;
    }
    if depth == 0 {
        return evaluate(board, me);
    }
    let moves = generate_moves(board, me);
    if moves.is_empty() {
        if count_moves(board, me.opponent()) == 0 {
            return evaluate(board, me); // game over
        }
        return -negamax(board, me.opponent(), depth, -beta, -alpha, s);
    }
    let order = order_moves(board, me, &moves, depth >= 3);
    let mut best = SCORE_LOSS;
    for &idx in &order[..moves.len()] {
        let m = moves.get(idx).unwrap();
        let nb = apply_move(board, me, m.pos, m.flipped);
        let v = -negamax(&nb, me.opponent(), depth - 1, -beta, -alpha, s);
        if s.aborted {
            return 0;
        }
        best = best.max(v);
        alpha = alpha.max(v);
        if alpha >= beta {
            break;
        }
    }
    best
}

/// Exact endgame search; the score is the final disc differential for `me`.
fn solve(board: &Board, me: Player, mut alpha: Score, beta: Score, s: &mut Search) -> Score {
    if s.tick() {
        return 0;
    }
    let moves = generate_moves(board, me);
    if moves.is_empty() {
        if count_moves(board, me.opponent()) == 0 {
            return board.count(me) as Score - board.count(me.opponent()) as Score;
        }
        return -solve(board, me.opponent(), -beta, -alpha, s);
    }
    let order = order_moves(board, me, &moves, board.empty_count() > 6);
    let mut best = -65;
    for &idx in &order[..moves.len()] {
        let m = moves.get(idx).unwrap();
        let nb = apply_move(board, me, m.pos, m.flipped);
        let v = -solve(&nb, me.opponent(), -beta, -alpha, s);
        if s.aborted {
            return 0;
        }
        best = best.max(v);
        alpha = alpha.max(v);
        if alpha >= beta {
            break;
        }
    }
    best
}

/// Search every root move to `depth`, best-first. Returns None if aborted.
fn root_search(
    board: &Board,
    player: Player,
    moves: &MoveList,
    first: Option<Position>,
    depth: u8,
    solver: bool,
    s: &mut Search,
) -> Option<Position> {
    let mut order = order_moves(board, player, moves, true);
    if let Some(f) = first {
        if let Some(i) = order[..moves.len()].iter().position(|&ix| moves.get(ix).unwrap().pos == f) {
            order[..=i].rotate_right(1);
        }
    }
    let (lo, hi) = if solver { (-65, 65) } else { (SCORE_LOSS, SCORE_WIN) };
    let mut alpha = lo;
    let mut best_pos = None;
    for &idx in &order[..moves.len()] {
        let m = moves.get(idx).unwrap();
        let nb = apply_move(board, player, m.pos, m.flipped);
        let v = if solver {
            -solve(&nb, player.opponent(), -hi, -alpha, s)
        } else {
            -negamax(&nb, player.opponent(), depth - 1, -hi, -alpha, s)
        };
        if s.aborted {
            return None;
        }
        if best_pos.is_none() || v > alpha {
            alpha = alpha.max(v);
            best_pos = Some(m.pos);
        }
    }
    best_pos
}

/// Find the best move within `limits`. Always returns a legal move if one
/// exists: when the budget runs out the result of the last completed depth is
/// used, falling back to the best-ordered move.
pub fn find_best_move_limited(
    board: &Board,
    player: Player,
    difficulty: Difficulty,
    limits: &Limits,
) -> Option<Position> {
    let moves = generate_moves(board, player);
    if moves.is_empty() {
        return None;
    }
    if moves.len() == 1 {
        return Some(moves.get(0).unwrap().pos);
    }
    let mut s = Search { nodes: 0, aborted: false, limits };
    // Fallback: best statically-ordered move.
    let order = order_moves(board, player, &moves, true);
    let mut best = moves.get(order[0]).unwrap().pos;

    if difficulty.use_endgame_solver() && board.empty_count() <= difficulty.endgame_threshold() {
        if let Some(p) = root_search(board, player, &moves, None, 0, true, &mut s) {
            return Some(p);
        }
        s.aborted = false; // solver ran out of budget: fall back to depth search
        s.nodes = 0;
    }

    for depth in 1..=difficulty.depth() {
        match root_search(board, player, &moves, Some(best), depth, false, &mut s) {
            Some(p) => best = p,
            None => break,
        }
    }
    Some(best)
}

/// Find the best move for the given difficulty with its default node budget.
pub fn find_best_move(board: &Board, player: Player, difficulty: Difficulty) -> Option<Position> {
    let never = || false;
    find_best_move_limited(board, player, difficulty, &Limits { max_nodes: difficulty.node_budget(), stop: &never })
}

/// Get a hint (best move) for the player, at Hard strength.
pub fn get_hint(board: &Board, player: Player) -> Option<Position> {
    find_best_move(board, player, Difficulty::Hard)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moves::calculate_flips;

    fn board_from(black: &[Position], white: &[Position]) -> Board {
        let mut b = Board::empty();
        for &p in black {
            b.place(Player::Black, p);
        }
        for &p in white {
            b.place(Player::White, p);
        }
        b
    }

    #[test]
    fn all_difficulties_return_a_legal_opening_move() {
        let board = Board::new();
        for d in [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard, Difficulty::Expert] {
            let p = find_best_move(&board, Player::Black, d).expect("a move");
            assert!(calculate_flips(&board, Player::Black, p) != 0, "{d:?}");
        }
    }

    #[test]
    fn forced_single_move_is_returned() {
        // Black A1... White B1, Black D1: only C... Black must play? Use: Black C1? set
        // Black at 3 (D1), White at 2 (C1): Black's only legal move is B1 (1).
        let b = board_from(&[3], &[2]);
        for d in [Difficulty::Easy, Difficulty::Expert] {
            assert_eq!(find_best_move(&b, Player::Black, d), Some(1));
        }
    }

    #[test]
    fn takes_available_corner() {
        // Regression for audit test-quality: the old test never asserted anything.
        // White B1, Black C1: Black can play A1 (corner) -- and also has other moves.
        let b = board_from(&[2, 27, 36], &[1, 28, 35]);
        assert!(calculate_flips(&b, Player::Black, 0) != 0);
        assert_eq!(find_best_move(&b, Player::Black, Difficulty::Medium), Some(0));
    }

    #[test]
    fn no_moves_returns_none() {
        let b = board_from(&[0], &[]);
        assert_eq!(find_best_move(&b, Player::White, Difficulty::Easy), None);
    }

    #[test]
    fn endgame_solver_finds_the_winning_move() {
        // Row 1 filled except A1/B1... Build: 6 empties near the end, check the solver's
        // choice yields at least as good a final differential as every other root move.
        let mut b = Board::new();
        let mut player = Player::Black;
        let mut seed = 7u64;
        while b.empty_count() > 8 {
            let moves = generate_moves(&b, player);
            if moves.is_empty() {
                player = player.opponent();
                if generate_moves(&b, player).is_empty() {
                    break;
                }
                continue;
            }
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let m = *moves.get((seed % moves.len() as u64) as usize).unwrap();
            b = apply_move(&b, player, m.pos, m.flipped);
            player = player.opponent();
        }
        let never = || false;
        let lim = Limits { max_nodes: u32::MAX, stop: &never };
        let moves = generate_moves(&b, player);
        if moves.len() > 1 {
            let chosen = find_best_move_limited(&b, player, Difficulty::Hard, &lim).unwrap();
            let mut s = Search { nodes: 0, aborted: false, limits: &lim };
            let mut best = -65;
            let mut chosen_v = -65;
            for m in moves.iter() {
                let nb = apply_move(&b, player, m.pos, m.flipped);
                let v = -solve(&nb, player.opponent(), -65, 65, &mut s);
                best = best.max(v);
                if m.pos == chosen {
                    chosen_v = v;
                }
            }
            assert_eq!(chosen_v, best);
        }
    }

    #[test]
    fn stop_callback_abandons_search_but_still_returns_a_legal_move() {
        let board = Board::new();
        let stop_now = || true;
        let lim = Limits { max_nodes: u32::MAX, stop: &stop_now };
        let p = find_best_move_limited(&board, Player::Black, Difficulty::Expert, &lim).unwrap();
        assert!(calculate_flips(&board, Player::Black, p) != 0);
    }

    #[test]
    fn node_budget_bounds_the_work() {
        let board = Board::new();
        let never = || false;
        let lim = Limits { max_nodes: 500, stop: &never };
        let mut s = Search { nodes: 0, aborted: false, limits: &lim };
        let _ = negamax(&board, Player::Black, 8, SCORE_LOSS, SCORE_WIN, &mut s);
        assert!(s.aborted);
        assert!(s.nodes <= 501);
    }

    #[test]
    fn ordering_puts_corner_first() {
        let b = board_from(&[2, 27, 36], &[1, 28, 35]);
        let moves = generate_moves(&b, Player::Black);
        let ord = order_moves(&b, Player::Black, &moves, true);
        assert_eq!(moves.get(ord[0]).unwrap().pos, 0);
    }
}
