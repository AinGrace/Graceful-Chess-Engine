use std::{fmt::Display, ops::Neg};

use position::position::Position;
use types::chess_move::Move;

use crate::eval::{self, constants::NEG_INF};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Score {
    Centipawn(i32),
    Mate(i16),
    Draw,
}

impl Score {
    fn value(self) -> i32 {
        match self {
            Score::Mate(val) if val > 0 => 100_000 - val as i32,
            Score::Mate(val) => -100_000 - val as i32,
            Score::Centipawn(val) => val,
            Score::Draw => 0,
        }
    }

    fn step(self) -> Self {
        match self {
            Score::Mate(val) if val >= 0 => Self::Mate(val + 1),
            Score::Mate(val) => Self::Mate(val - 1),
            rest => rest,
        }
    }
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value().cmp(&other.value())
    }
}

impl Neg for Score {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Score::Centipawn(val) => Score::Centipawn(-val),
            Score::Mate(val) => Score::Mate(-val),
            Score::Draw => Score::Draw,
        }
    }
}

impl Display for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Score::Centipawn(val) => format!("{}", val),
                Score::Mate(val) => format!("MATE IN {}", val),
                Score::Draw => format!("DRAW"),
            }
        )
    }
}

/// TODO: add alpha-beta pruning
pub fn negamax(pos: &mut Position, depth: u8) -> (Score, Option<Move>) {
    let moves = pos.legal_moves();

    if moves.is_empty() {
        if pos.checkers_to(pos.turn()).present() {
            return (Score::Mate(-1), None);
        } else {
            return (Score::Draw, None);
        }
    }

    if depth == 0 {
        return (Score::Centipawn(eval::static_eval(pos)), None);
    }

    let mut best_score = Score::Centipawn(NEG_INF);
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
}
