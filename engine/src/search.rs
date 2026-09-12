use std::{
    cmp::{max, min},
    ops::Div,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use position::position::Position;
use types::chess_move::Move;

use crate::{
    eval::{self, Score},
    mvv_lva,
    time_control::TimeControl,
    tt::TTOptions,
};

pub const MAX_DEPTH: u8 = 128;

#[derive(Debug, Clone, Copy)]
pub enum Bound {
    Exact,
    Lower,
    Upper,
}

impl Default for Bound {
    fn default() -> Self {
        Self::Exact
    }
}

#[derive(Default, Debug, Clone)]
pub struct SearchResult {
    pub depth: u8,
    pub score: Score,
    pub best_move: Option<Move>,
    pub nodes: u64,
    pub nps: u64,
    pub elapsed_millis: u128,
    pub next_depth_prediction: u64,
}

impl SearchResult {
    #[track_caller]
    fn of(
        NegamaxResult {
            score,
            best_move,
            nodes,
        }: NegamaxResult,
        depth: u8,
        elapsed: Duration,
        next_depth_prediction: u64,
    ) -> Self {
        let elapsed_millis = elapsed.as_millis();
        let elapsed_secs = elapsed.as_secs_f64();
        let nps = (nodes as f64 / elapsed_secs) as u64;

        Self {
            depth,
            score,
            best_move,
            nodes,
            nps,
            elapsed_millis,
            next_depth_prediction,
        }
    }

    pub fn is_aborted(&self) -> bool {
        matches!(self.score, Score::Abort)
    }
}

#[derive(Default, Debug)]
struct NegamaxResult {
    pub score: Score,
    pub best_move: Option<Move>,
    pub nodes: u64,
}

impl NegamaxResult {
    pub fn new_abort(nodes: u64) -> Self {
        Self::new(Score::Abort, None, nodes)
    }

    pub fn new(score: Score, best_move: Option<Move>, nodes: u64) -> Self {
        Self {
            score,
            best_move,
            nodes,
        }
    }

    pub fn is_aborted(&self) -> bool {
        matches!(self.score, Score::Abort)
    }
}

pub struct SearchOptions<'a> {
    pub pos: &'a mut Position,
    pub search_depth: u8,
    pub tt_opts: TTOptions,
    pub stop_thinking: &'a Arc<AtomicBool>,
    pub time_control: TimeControl,
}

pub fn search<F>(
    SearchOptions {
        pos,
        search_depth,
        tt_opts,
        stop_thinking: stop_flag,
        time_control,
    }: SearchOptions,
    mut f: F,
) -> SearchResult
where
    F: FnMut(&SearchResult),
{
    struct DepthInfo {
        nodes: u64,
    }

    let mut depths: Vec<DepthInfo> = vec![];

    let mut next_predicted_time = 0;

    let mut result = SearchResult::default();

    let alpha = Score::Mate(-1);
    let beta = Score::Mate(1);

    for curr_depth in 1..=search_depth {
        if time_control.soft_expired()
            || (next_predicted_time > time_control.soft_limit.as_millis() as u64 && curr_depth >= 6)
        {
            return result;
        }

        let current_result = negamax(
            pos,
            curr_depth,
            &tt_opts,
            alpha,
            beta,
            stop_flag,
            &time_control,
            &mut 0,
        );

        if current_result.is_aborted() {
            return result;
        }

        result = SearchResult::of(
            current_result,
            curr_depth,
            time_control.elapsed_from_start(),
            next_predicted_time,
        );

        depths.push(DepthInfo {
            nodes: result.nodes,
        });

        f(&result);

        if curr_depth >= 3 && time_control.soft_limit != Duration::MAX {
            let mut total_time = time_control.elapsed_from_start().as_millis() as u64;
            if total_time == 0 {
                total_time = 1;
            }

            let mut branching_factor = 0.0;

            let total_nodes: u64 = depths.iter().map(|d| d.nodes).sum();

            const DEPTH_PREDICTION_WINDOW: usize = 4;
            let depth_len = depths.len();

            for i in depth_len - DEPTH_PREDICTION_WINDOW..depth_len - 1 {
                let d_nodes = depths[i + 1].nodes;
                let prev_d_nodes = depths[i].nodes;

                branching_factor += d_nodes.div(prev_d_nodes) as f64;
            }

            // Exclude zero nodes/time depth from calculations
            branching_factor = branching_factor / depths.len() as f64;

            let next_predicted_nodes = (result.nodes as f64 * branching_factor) as u64;
            next_predicted_time = (next_predicted_nodes * total_time) / total_nodes;
        }

        if matches!(result.score, Score::Mate(_)) {
            return result;
        }

        if time_control.soft_expired() {
            depths.push(DepthInfo {
                nodes: result.nodes,
            });
            return result;
        }
    }

    result
}

fn should_extend(prev_score: &Score, cur_score: &Score, best_move_changed: bool) -> f64 {
    let mut factor = 1.0;

    if *cur_score < *prev_score - 100 {
        factor *= 1.3;
    }

    if best_move_changed {
        factor *= 1.25;
    }

    factor
}

fn negamax(
    pos: &mut Position,
    depth: u8,
    tt_opts: &TTOptions,
    mut alpha: Score,
    mut beta: Score,
    stop_flag: &Arc<AtomicBool>,
    time_control: &TimeControl,
    nodes: &mut u64,
) -> NegamaxResult {
    if nodes.trailing_zeros() == 15 && time_control.hard_expired() {
        return NegamaxResult::new_abort(*nodes);
    }

    if stop_flag
        .compare_exchange(true, false, Ordering::Relaxed, Ordering::Relaxed)
        .is_ok()
    {
        return NegamaxResult::new_abort(*nodes);
    }

    let mut bound = Bound::Upper;

    if depth == 0 {
        return NegamaxResult::new(eval::static_eval(pos), None, *nodes);
    }

    let mut tt_move = None;

    if let Some(entry) = tt_opts.probe(pos.zobrist_hash(), depth) {
        tt_move = entry.best_move;

        match entry.bound {
            Bound::Exact => {
                return NegamaxResult::new(entry.score, entry.best_move, *nodes);
            }
            Bound::Lower => alpha = max(alpha, entry.score),
            Bound::Upper => beta = min(beta, entry.score),
        }

        if alpha >= beta {
            return NegamaxResult::new(entry.score, entry.best_move, *nodes);
        }
    }

    let moves = pos.legal_moves();

    if moves.is_empty() {
        let score = if pos.in_check() {
            Score::Mate(0)
        } else {
            Score::Draw
        };
        return NegamaxResult {
            score,
            best_move: None,
            nodes: *nodes,
        };
    }

    let mut result = NegamaxResult::new(Score::Mate(0), None, *nodes);

    let mut scored_moves = mvv_lva::score_moves(moves, tt_move);

    for i in 0..scored_moves.len() {
        mvv_lva::bubble_high_scored_move(&mut scored_moves, i);
        let current_move = scored_moves[i].0;

        let undo = pos.do_move_inner(current_move);

        *nodes += 1;

        let search_result = negamax(
            pos,
            depth - 1,
            tt_opts,
            -beta,
            -alpha,
            stop_flag,
            time_control,
            nodes,
        );

        pos.undo_move(undo);

        if matches!(search_result.score, Score::Abort) {
            return search_result;
        }

        let inverted_score_step = (-search_result.score).step();

        if inverted_score_step > result.score {
            result.score = inverted_score_step;
            result.best_move = Some(current_move);
        }

        if inverted_score_step > alpha {
            alpha = inverted_score_step;
            bound = Bound::Exact;
        }

        if inverted_score_step > beta {
            bound = Bound::Lower;
            break;
        }
    }

    tt_opts.insert(
        pos.zobrist_hash(),
        depth,
        result.score,
        result.best_move,
        bound,
    );

    result.nodes = *nodes;
    result
}

fn quiesce(
    pos: &mut Position,
    tt_opts: &TTOptions,
    mut alpha: Score,
    mut beta: Score,
    stop_flag: &Arc<AtomicBool>,
    time_control: &TimeControl,
    nodes: &mut u64,
) -> NegamaxResult {
    if nodes.trailing_zeros() == 15 && time_control.hard_expired() {
        return NegamaxResult::new_abort(*nodes);
    }

    if stop_flag
        .compare_exchange(true, false, Ordering::Relaxed, Ordering::Relaxed)
        .is_ok()
    {
        return NegamaxResult::new_abort(*nodes);
    }

    let mut bound = Bound::Upper;
    let mut tt_move = None;

    match tt_opts {
        TTOptions::Enabled(tt) => {
            let tt = tt.lock();

            // TODO: explanation comments
            if let Some(entry) = tt.get(pos.zobrist_hash(), 0)
                && entry.hash == pos.zobrist_hash()
            {
                tt_move = entry.best_move;

                match entry.bound {
                    Bound::Exact => {
                        return NegamaxResult::new(entry.score, tt_move, *nodes);
                    }
                    Bound::Lower => alpha = max(alpha, entry.score),
                    Bound::Upper => beta = min(beta, entry.score),
                }

                if alpha > beta {
                    return NegamaxResult::new(entry.score, tt_move, *nodes);
                }
            }
        }
        TTOptions::Disabled => (),
    }

    let in_check = pos.in_check();

    let mut best_score = if in_check {
        Score::Mate(0)
    } else {
        let stand_pat = eval::static_eval(pos);

        if stand_pat >= beta {
            // TODO: store pos stand+pat into TT
            return NegamaxResult::new(stand_pat, None, *nodes);
        }

        if stand_pat > alpha {
            alpha = stand_pat;
            bound = Bound::Exact;
        }

        stand_pat
    };

    let moves = if in_check {
        pos.legal_moves()
    } else {
        // TODO: movegen for captures
        todo!()
    };

    let mut best_move = None;
    let mut scored_moves = mvv_lva::score_moves(moves, tt_move);

    for i in 0..scored_moves.len() {
        mvv_lva::bubble_high_scored_move(&mut scored_moves, i);
        let current_move = scored_moves[i].0;

        if !in_check {
            let see_score = {
                let turn = pos.turn();
                let mut board = pos.board_owned();

                eval::eval_see(&mut board, current_move.to(), turn)
            };

            if see_score == 0 {
                continue;
            }

            // 2 centipawns of margin
            const DELTA_MARGIN: i16 = 200;
            if best_score.value() + see_score + DELTA_MARGIN < alpha.value() {
                continue;
            }
        }

        let undo = pos.do_move_inner(current_move);
        *nodes += 1;

        let search_result = quiesce(pos, tt_opts, -beta, -alpha, stop_flag, time_control, nodes);

        pos.undo_move(undo);

        if matches!(search_result.score, Score::Abort) {
            return search_result;
        }

        let inverted_score_step = (-search_result.score).step();

        if inverted_score_step > best_score {
            best_score = inverted_score_step;
            best_move = Some(current_move)
        }

        if inverted_score_step > alpha {
            alpha = inverted_score_step;
            bound = Bound::Exact;
        }

        if inverted_score_step > beta {
            bound = Bound::Lower;
            break;
        }
    }

    match tt_opts {
        TTOptions::Enabled(tt) => {
            let mut tt = tt.lock();
            tt.insert(pos.zobrist_hash(), 0, best_score, best_move, bound);
        }
        TTOptions::Disabled => (),
    }

    return NegamaxResult::new(best_score, best_move, *nodes);
}
