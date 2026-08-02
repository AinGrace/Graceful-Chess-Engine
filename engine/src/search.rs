use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use position::position::Position;
use types::chess_move::Move;

use crate::eval::{self, Score};

/// TODO: add alpha-beta pruning
pub fn negamax(
    pos: &mut Position,
    depth: u8,
    mut alpha: Score,
    beta: Score,
    stop_flag: &Arc<AtomicBool>,
) -> (Score, Option<Move>) {
    let moves = pos.legal_moves();

    if depth == 0 {
        return (eval::static_eval(pos), None);
    }

    let mut best_score = Score::Mate(0);
    let mut best_move = None;

    for mv in moves {
        let undo = pos.do_move_inner(mv);

        let (child_score, _) = negamax(pos, depth - 1, -beta, -alpha, stop_flag);

        pos.undo_move(undo);

        let score = (-child_score).step();

        if score > best_score {
            best_score = score;
            best_move = Some(mv);
        }

        if score > beta {
            break;
        }

        alpha = alpha.max(score);

        if stop_flag.load(Ordering::Relaxed) {
            return (best_score, best_move);
        }
    }

    (best_score, best_move)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use position::fen::Fen;

    use super::*;
}
