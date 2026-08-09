use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use position::position::Position;
use types::chess_move::Move;

use crate::{
    eval::{self, Score},
    mvv_lva,
    time_control::TimeControl,
    tt::TT,
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

#[derive(Default, Debug)]
pub struct SearchResult {
    pub score: Score,
    pub best_move: Option<Move>,
    pub nodes: u64,
}

impl SearchResult {
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
    pub search_depth: Option<u8>,
    pub tt_opts: TTOptions,
    pub stop_flag: &'a Arc<AtomicBool>,
    pub time_control: TimeControl,
}

pub enum TTOptions {
    Enabled(Arc<Mutex<TT>>),
    Disabled,
}

pub fn search<F>(
    SearchOptions {
        pos,
        search_depth,
        tt_opts,
        stop_flag,
        time_control,
    }: SearchOptions,
    mut f: F,
) -> SearchResult
where
    F: FnMut(&str),
{
    let mut result = SearchResult::default();

    let depth = search_depth.unwrap_or(MAX_DEPTH);

    let alpha = Score::Mate(-1);
    let beta = Score::Mate(1);

    for curr_depth in 1..=depth {
        if time_control.soft_expired() {
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

        let searched_nodes = current_result.nodes;
        let nps = searched_nodes as f64 / time_control.elapsed_secs_f64();

        f(&format!(
            "info depth {curr_depth} {} nodes {} nps {} time {}",
            current_result.score,
            searched_nodes,
            nps.trunc(),
            time_control.elapsed_from_start().as_millis()
        ));

        result = current_result;

        if time_control.soft_expired() {
            return result;
        }
    }

    result
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
) -> SearchResult {
    if nodes.trailing_zeros() == 16 {
        if time_control.hard_expired() {
            return SearchResult::new_abort(*nodes);
        }
    }

    if stop_flag.load(Ordering::Relaxed) {
        stop_flag.store(false, Ordering::Relaxed);
        return SearchResult::new_abort(*nodes);
    }

    let mut bound = Bound::Upper;

    if depth == 0 {
        return SearchResult::new(eval::static_eval(pos), None, *nodes);
    }

    let mut tt_move = None;

    match tt_opts {
        TTOptions::Enabled(tt) => {
            let tt_handle = tt.lock().expect("unable to acquire lock on TT mutex");
            if let Some(entry) = tt_handle.get(pos.zobrist_hash(), depth)
                && entry.hash == pos.zobrist_hash()
            {
                tt_move = entry.best_move;

                if entry.depth >= depth {
                    match entry.bound {
                        Bound::Exact => {
                            return SearchResult::new(entry.score, entry.best_move, *nodes);
                        }
                        Bound::Lower => {
                            alpha = if entry.score > alpha {
                                entry.score
                            } else {
                                alpha
                            }
                        }
                        Bound::Upper => {
                            beta = if entry.score < beta {
                                entry.score
                            } else {
                                beta
                            }
                        }
                    }

                    if alpha >= beta {
                        return SearchResult::new(entry.score, entry.best_move, *nodes);
                    }
                }
            }
        }
        TTOptions::Disabled => (),
    };

    let moves = pos.legal_moves();

    let mut result = SearchResult::new(Score::Mate(0), None, *nodes);

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

    match tt_opts {
        TTOptions::Enabled(tt) => {
            let mut tt_handle = tt.lock().expect("FATAL");
            tt_handle.insert(
                pos.zobrist_hash(),
                depth,
                result.score,
                result.best_move,
                bound,
            );
        }
        TTOptions::Disabled => (),
    }

    result.nodes = *nodes;
    result
}
