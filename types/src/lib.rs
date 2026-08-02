use arrayvec::ArrayVec;

use crate::chess_move::Move;

pub mod bitboard;
pub mod by_color;
pub mod by_role;
pub mod castlings;
pub mod chess_move;
pub mod color;
pub mod direction;
pub mod file;
pub mod piece;
pub mod rank;
pub mod role;
pub mod square;

/// General use move list
pub type MoveList = ArrayVec<Move, 218>;

/// This one is specifically used for MVV-LVA in negamax
pub type ScoredMoveList = ArrayVec<(Move, u8), 218>;
