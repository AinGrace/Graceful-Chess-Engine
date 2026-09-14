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

        if curr_depth >= 5 && time_control.soft_limit != Duration::MAX {
            let mut total_time = time_control.elapsed_from_start().as_millis() as u64;
            if total_time == 0 {
                total_time = 1;
            }

            let mut branching_factor = 0.0;

            let total_nodes: u64 = depths.iter().map(|d| d.nodes).sum();

            if total_nodes > 0 {
                const DEPTH_PREDICTION_WINDOW: usize = 4;
                let depth_len = depths.len();

                for i in depth_len - DEPTH_PREDICTION_WINDOW..depth_len - 1 {
                    let d_nodes = depths[i + 1].nodes;
                    let prev_d_nodes = depths[i].nodes;

                    if prev_d_nodes == 0 {
                        break;
                    }

                    branching_factor += d_nodes.div(prev_d_nodes) as f64;
                }

                // Exclude zero nodes/time depth from calculations
                branching_factor = branching_factor / depths.len() as f64;

                let next_predicted_nodes = (result.nodes as f64 * branching_factor) as u64;
                next_predicted_time = (next_predicted_nodes * total_time) / total_nodes;
            }
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

    // if depth == 0 {
    //     return NegamaxResult::new(eval::static_eval(pos), None, *nodes);
    // }

    if depth == 0 {
        return quiesce(pos, 7, tt_opts, alpha, beta, stop_flag, time_control, nodes);
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

    let mut scored_moves = mvv_lva::score_moves(pos.board(), moves, tt_move);

    for i in 0..scored_moves.len() {
        mvv_lva::bubble_high_scored_move(&mut scored_moves, i);
        let current_move = scored_moves[i].0;

        //SAFETY: current_move is part of legal MoveList
        let undo = unsafe { pos.do_move_unchecked(current_move) };

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

        // SAFETY: undo is product of the previous do_move_uncheced call
        unsafe { pos.undo_move(undo) };

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

    if depth == 0 {
        return NegamaxResult::new(eval::static_eval(pos), None, *nodes);
    }

    let mut tt_move = None;

    if let Some(entry) = tt_opts.probe(pos.zobrist_hash(), 0) {
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

    let in_check = if depth == 7 { pos.in_check() } else { false };

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
        }

        stand_pat
    };

    let moves = if in_check {
        pos.legal_moves()
    } else {
        pos.legal_captures()
    };

    let mut best_move = None;
    let mut scored_moves = mvv_lva::score_moves(pos.board(), moves, tt_move);

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

        //SAFETY: current_move is part of legal MoveList
        let undo = unsafe { pos.do_move_unchecked(current_move) };
        *nodes += 1;

        let search_result = quiesce(
            pos,
            depth - 1,
            tt_opts,
            -beta,
            -alpha,
            stop_flag,
            time_control,
            nodes,
        );

        //SAFETY: undo is the product of do_move_unchecked call above
        unsafe { pos.undo_move(undo) };

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
        }

        if inverted_score_step > beta {
            break;
        }
    }

    return NegamaxResult::new(best_score, best_move, *nodes);
}

#[cfg(test)]
mod tests {
    use crate::{search, time_control};

    use super::*;

    #[test]
    fn search_test() {
        let mut pos = Position::new();
        pos.uci_moves_checked(
            "
            e2e4 e7e5 b1c3 g8f6 g1f3 d7d6 d2d4 b8d7 c1g5
            f8e7 f1b5 a7a6 g5f6 e7f6 b5d7 d8d7 c3d5 d7d8
            d4e5 f6e5 d1b1 f7f6 h1f1 e8g8 h2h3 c8e6 d5b4
            d8e8 g2g3 e8f7 f1h1 f7h5 f3e5 h5e5 c2c3 a8e8
            b1d3 e5h5 g3g4 e6g4 b4d5 g4f3 d5f4 f3e4 f4h5
            e4d3 e1d2 d3g6 h5f4 g6e4 h1e1 g8f7 c3c4 e8e5
            c4c5 g7g5 f4d3 e4d3 d2d3 f8d8 c5d6 e5d5",
        );

        let fen = pos.to_fen();

        println!("{fen}");

        let stop_thinking = Arc::new(AtomicBool::default());

        search::search(
            SearchOptions {
                pos: &mut pos,
                search_depth: 7,
                tt_opts: TTOptions::Disabled,
                stop_thinking: &stop_thinking,
                time_control: TimeControl::new_infinite(),
            },
            |r| (),
        );
    }

    #[test]
    fn self_play() {
        let mut pos = Position::new();
        let stop_thinking = Arc::new(AtomicBool::default());
        let time_control = TimeControl::new_infinite();

        loop {
            let search_opts = SearchOptions {
                pos: &mut pos,
                search_depth: 7,
                tt_opts: TTOptions::Disabled,
                stop_thinking: &stop_thinking,
                time_control: time_control.clone(),
            };

            let res = search(search_opts, |mv| ());

            if let Some(mov) = res.best_move {
                pos.do_move(mov).unwrap();
            } else {
                break;
            }
        }
    }
}
