use position::chessboard::{ChessBoard, Undo};
use types::chess_move::Move;

use crate::eval::full_eval;

pub fn negamax(pos: &ChessBoard, depth: u8) -> (i32, Option<Move>) {
    if depth == 0 {
        return (full_eval(pos), None)
    }

    let mut best_score = i32::MIN;
    // let mut best_move = None;

    for mv in pos.legal_moves() {
        
    }
    todo!()
}