use position::position::Position;
use types::chess_move::Move;

use crate::eval::{
    self, constants::{DRAW_SCORE, MATE_SCORE, NEG_INF}
};

/// TODO: add alpha-beta pruning
pub fn negamax(pos: &mut Position, depth: u8, ply: u8) -> (i32, Option<Move>) {
    let moves = pos.legal_moves();

    if moves.is_empty() {
        if pos.checkers(pos.turn()).present() {
            return (-(MATE_SCORE - ply as i32), None);
        } else {
            return (DRAW_SCORE, None);
        }
    }

    if depth == 0 {
        return (eval::static_eval(pos), None);
    }

    let mut best_score = NEG_INF;
    let mut best_move = None;

    for mv in moves {
        let undo = pos.do_move_inner(mv);

        let (child_score, _) = negamax(pos, depth - 1, ply + 1);

        pos.undo_move(undo);

        let score = -child_score;

        if score > best_score {
            best_score = score;
            best_move = Some(mv);
        }
    }
    return (best_score, best_move);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negamax_test() {
        let mut pos = Position::new();

        pos.uci_move_checked("d2d4");
        pos.uci_move_checked("d7d5");
        pos.uci_move_checked("g1f3");
        pos.uci_move_checked("b8c6");

        let x = negamax(&mut pos, 5, 0);

        dbg!(x);
    }
}
