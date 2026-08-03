use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use position::position::Position;
use types::chess_move::Move;

use crate::{
    eval::{self, Score},
    mvv_lva,
    tt::TT,
};

const MAX_DEPTH: u8 = 128;

#[derive(Default, Debug)]
pub struct SearchResult {
    pub score: Score,
    pub best_move: Option<Move>,
}

pub enum TTOptions {
    Enabled(Arc<Mutex<TT>>),
    Disabled,
}

pub struct SearchOptions<'a> {
    pub pos: &'a mut Position,
    pub search_depth: Option<u8>,
    pub tt: TTOptions,
    pub stop_flag: &'a Arc<AtomicBool>,
}

pub fn search(opts: SearchOptions) -> SearchResult {
    let mut res = SearchResult::default();

    let pos = opts.pos;
    let depth = opts.search_depth.unwrap_or(MAX_DEPTH);
    let tt_opts = opts.tt;
    let stop_flag = &opts.stop_flag;

    let alpha = Score::Mate(-1);
    let beta = Score::Mate(1);

    for i in 1..=depth {
        let (score, best_move) = negamax(pos, i, &tt_opts, alpha, beta, stop_flag);

        if matches!(score, Score::Stopped) {
            return res;
        }

        res.score = score;
        res.best_move = best_move;
    }

    res
}

fn negamax(
    pos: &mut Position,
    depth: u8,
    tt_opts: &TTOptions,
    mut alpha: Score,
    beta: Score,
    stop_flag: &Arc<AtomicBool>,
) -> (Score, Option<Move>) {
    if stop_flag.load(Ordering::Relaxed) {
        stop_flag.store(false, Ordering::Relaxed);
        return (Score::Stopped, None);
    }

    if depth == 0 {
        return (eval::static_eval(pos), None);
    }

    match tt_opts {
        TTOptions::Enabled(tt) => {
            let tt_handle = tt.lock().expect("unable to acquire lock on TT mutex");
            if let Some(entry) = tt_handle.get(pos.zobrist_hash(), depth)
                && entry.hash == pos.zobrist_hash()
                && entry.depth >= depth
            {
                return (entry.score, entry.best_move);
            }
        }
        TTOptions::Disabled => (),
    };

    let moves = pos.legal_moves();

    let mut best_score = Score::Mate(0);
    let mut best_move = None;

    let mut scored_moves = mvv_lva::score_moves(moves);

    for i in 0..scored_moves.len() {
        mvv_lva::bubble_high_scored_move(&mut scored_moves, i);
        let current_move = scored_moves[i].0;

        let undo = pos.do_move_inner(current_move);

        let (child_score, _) = negamax(pos, depth - 1, tt_opts, -beta, -alpha, stop_flag);

        pos.undo_move(undo);

        let score = (-child_score).step();

        if score > best_score {
            best_score = score;
            best_move = Some(current_move);
        }

        if score > beta {
            break;
        }

        alpha = alpha.max(score);
    }

    match tt_opts {
        TTOptions::Enabled(tt) => {
            let mut tt_handle = tt.lock().expect("unable to acquire lock on TT mutex");
            tt_handle.insert(pos.zobrist_hash(), depth, best_score, best_move);
        }
        TTOptions::Disabled => (),
    }

    (best_score, best_move)
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex, atomic::AtomicBool};

    use position::position::Position;

    use crate::{
        search::{self, SearchOptions, TTOptions},
        tt::TT,
    };

    #[test]
    fn search_standart_pos() {
        let mut pos = Position::new();

        let tt_opts = TTOptions::Enabled(Arc::new(Mutex::new(TT::new(256))));

        let search_opts = SearchOptions {
            pos: &mut pos,
            search_depth: Some(8),
            tt: tt_opts,
            stop_flag: &Arc::new(AtomicBool::new(false)),
        };

        let res = search::search(search_opts);

        println!("{:#?}", res)
    }
}
