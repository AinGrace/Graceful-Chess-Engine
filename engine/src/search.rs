use position::position::{Position, Undo};
use types::chess_move::Move;

use crate::eval::full_eval;

pub fn negamax(pos: &mut Position, depth: u8) -> (i32, Option<Move>) {
    if depth == 0 {
        return (full_eval(pos), None)
    }

    let moves = pos.legal_moves();

    if moves.is_empty() {
        return (full_eval(pos), None)
    }
    
    let mut best_score = i32::MIN;
    let mut best_move = None;

    for mv in pos.legal_moves() {
      let undo = pos.do_move_inner(mv);   
      let (opponent_score, _opponent_move) = negamax(pos, depth - 1);
      pos.undo_move(undo);

      let our_score = -opponent_score;

      if our_score > best_score {
          best_score = our_score;
          best_move = Some(mv);
      }
    }
    return (best_score, best_move)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negamax_test() {
        let mut pos = Position::new();

        let x = negamax(&mut pos, 5);

        dbg!(x);
    }
}