use position::position::Position;
use types::chess_move::Move;

use crate::eval::{self, Score, constants::NEG_INF};

/// TODO: add alpha-beta pruning
pub fn negamax(pos: &mut Position, depth: u8) -> (Score, Option<Move>) {
    let moves = pos.legal_moves();

    if depth == 0 {
        return (eval::static_eval(pos), None);
    }

    let mut best_score = Score::Mate(0);
    let mut best_move = None;

    for mv in moves {
        let undo = pos.do_move_inner(mv);

        let (child_score, _) = negamax(pos, depth - 1);

        pos.undo_move(undo);

        let score = (-child_score).step();

        if score > best_score {
            best_score = score;
            best_move = Some(mv);
        }
    }

    (best_score, best_move)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use position::fen::Fen;

    use super::*;

    #[test]
    fn negamax_test() {
        let mut pos = Position::new();

        pos.uci_move_checked("d2d4");
        pos.uci_move_checked("d7d5");
        pos.uci_move_checked("g1f3");
        pos.uci_move_checked("b8c6");

        let x = negamax(&mut pos, 5);

        dbg!(x);
    }

    #[test]
    fn negamax_sus_fen() {
        let raw_fen = "8/4P3/8/1k1K4/6P1/P1Q1B3/8/8 b - - 0 74";
        let fen = Fen::from_str(raw_fen).unwrap();
        let mut pos = fen.into_position().unwrap();

        let a = negamax(&mut pos, 4);
        println!("negamax result -> {a:#?}");
    }
}
