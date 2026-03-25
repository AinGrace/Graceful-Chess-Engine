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

pub type MoveList = ArrayVec<Move, 218>;
