use position::position::Position;
use types::{
    color::{self, Color},
    piece::{self, Piece},
    square::{self, Square},
};

use crate::eval::constants::{
    BISHOP_PST, BISHOP_VALUE, KING_MIDDLE_GAME_PST, KNIGHT_PST, KNIGHT_VALUE, PAWN_PST, PAWN_VALUE,
    QUEEN_PST, QUEEN_VALUE, ROOK_PST, ROOK_VALUE,
};

#[rustfmt::skip]
mod constants {
    
    pub const W_MATE_0: i32 = 100_000; 
    pub const B_MATE_0: i32 = -100_000; 

    pub const W_MATE_1: i32 = 10_000; 
    pub const B_MATE_1: i32 = -10_000; 
    
    pub const W_MATE_2: i32 = 9999; 
    pub const B_MATE_2: i32 = -9999; 
    
    pub const W_MATE_3: i32 = 9998; 
    pub const B_MATE_3: i32 = -9998; 
    
    pub const W_MATE_4: i32 = 9997; 
    pub const B_MATE_4: i32 = -9997; 
    
    pub const W_MATE_5: i32 = 9996; 
    pub const B_MATE_5: i32 = -9996; 
    
    /// assign 100 as default pawn value instead of 1 in order to avoid floating point calculations
    pub const PAWN_VALUE    :   u32 = 100;
    pub const KNIGHT_VALUE  :   u32 = 340;
    pub const BISHOP_VALUE  :   u32 = 350;
    pub const ROOK_VALUE    :   u32 = 500;
    pub const QUEEN_VALUE   :   u32 = 900;

    ///Piece-Square Tables (PSTs) are a simple evaluation technique that assigns a score to a piece depending on which square it occupies.
    ///The idea is:
    ///A knight in the center is usually stronger than a knight on the edge.
    ///A pawn advanced to the 6th rank is often more valuable than one on the 2nd rank.
    ///
    ///These values are part of the evaluation score
    pub const PAWN_PST: [i32; 64] = [
        0,   0,  0,  0,  0,  0,  0,  0,
        50, 50, 50, 50, 50, 50, 50, 50,
        10, 10, 20, 30, 30, 20, 10, 10,
         5,  5, 10, 25, 25, 10,  5,  5,
         0,  0,  0, 20, 20,  0,  0,  0,
         5, -5,-10,  0,  0,-10, -5,  5,
         5, 10, 10,-20,-20, 10, 10,  5,
         0,  0,  0,  0,  0,  0,  0,  0
    ];


    pub const KNIGHT_PST: [i32; 64] = [
       -50,-40,-30,-30,-30,-30,-40,-50,
       -40,-20,  0,  0,  0,  0,-20,-40,
       -30,  0, 10, 15, 15, 10,  0,-30,
       -30,  5, 15, 20, 20, 15,  5,-30,
       -30,  0, 15, 20, 20, 15,  0,-30,
       -30,  5, 10, 15, 15, 10,  5,-30,
       -40,-20,  0,  5,  5,  0,-20,-40,
       -50,-40,-30,-30,-30,-30,-40,-50,
    ];


    pub const BISHOP_PST: [i32; 64] = [
       -20,-10,-10,-10,-10,-10,-10,-20,
       -10,  0,  0,  0,  0,  0,  0,-10,
       -10,  0,  5, 10, 10,  5,  0,-10,
       -10,  5,  5, 10, 10,  5,  5,-10,
       -10,  0, 10, 10, 10, 10,  0,-10,
       -10, 10, 10, 10, 10, 10, 10,-10,
       -10,  5,  0,  0,  0,  0,  5,-10,
       -20,-10,-10,-10,-10,-10,-10,-20
    ];


    pub const ROOK_PST: [i32; 64] = [
        0,  0,  0,  0,  0,  0,  0,  0,
        5,  0,  0,  0,  0,  0,  0, -5,
       -5,  0,  0,  0,  0,  0,  0, -5,
       -5,  0,  0,  0,  0,  0,  0, -5,
       -5,  0,  0,  0,  0,  0,  0, -5,
       -5,  0,  0,  0,  0,  0,  0, -5,
       -5,  0,  0,  0,  0,  0,  0, -5,
        0,  0,  0,  5,  5,  0,  0,  0
    ];


    pub const QUEEN_PST: [i32; 64] = [
        -20,-10,-10, -5, -5,-10,-10,-20,
        -10,  0,  0,  0,  0,  0,  0,-10,
        -10,  0,  5,  5,  5,  5,  0,-10,
         -5,  0,  5,  5,  5,  5,  0, -5,
          0,  0,  5,  5,  5,  5,  0, -5,
        -10,  5,  5,  5,  5,  5,  0,-10,
        -10,  0,  5,  0,  0,  0,  0,-10,
        -20,-10,-10, -5, -5,-10,-10,-20
    ];


    pub const KING_MIDDLE_GAME_PST: [i32; 64] = [
        -30,-40,-40,-50,-50,-40,-40,-30,
        -30,-40,-40,-50,-50,-40,-40,-30,
        -30,-40,-40,-50,-50,-40,-40,-30,
        -30,-40,-40,-50,-50,-40,-40,-30,
        -20,-30,-30,-40,-40,-30,-30,-20,
        -10,-20,-20,-20,-20,-20,-20,-10,
         20, 20,  0,  0,  0,  0, 20, 20,
         20, 30, 10,  0,  0, 10, 30, 20
    ];

    pub const KING_END_GAME_PST: [i32; 64] = [
        -50,-40,-30,-20,-20,-30,-40,-50,
        -30,-20,-10,  0,  0,-10,-20,-30,
        -30,-10, 20, 30, 30, 20,-10,-30,
        -30,-10, 30, 40, 40, 30,-10,-30,
        -30,-10, 30, 40, 40, 30,-10,-30,
        -30,-10, 20, 30, 30, 20,-10,-30,
        -30,-30,  0,  0,  0,  0,-30,-30,
        -50,-30,-30,-30,-30,-30,-30,-50
    ];
}

pub fn full_eval(pos: &Position) -> i32 {
    let mobility = mobility(pos);
    let material = material_score(pos);
    let pst = calculate_pst_score(pos);

    #[cfg(test)]
    {
        // println!("Mobility -> {mobility} | Material -> {material} | PST -> {pst}");
    }

    let score = mobility + material + pst;

    if pos.turn() == Color::White {
        score as i32
    } else {
        -(score as i32)
    }
}

pub fn incremental_eval(pos: &Position, score: i32) -> i32 {
    todo!()
}

fn mobility(pos: &Position) -> i32 {
    // TODO: good mobility algorithm requires ChessBoard::legal_moves()
    // to be able to generate moves for both sides
    // not only for side to move
    0
}

fn material_score(pos: &Position) -> i32 {
    let white = Color::White;
    let black = Color::Black;

    let white_score = pos.board().pawns(white).popcnt() * PAWN_VALUE
        + pos.board().knights(white).popcnt() * KNIGHT_VALUE
        + pos.board().bishops(white).popcnt() * BISHOP_VALUE
        + pos.board().rooks(white).popcnt() * ROOK_VALUE
        + pos.board().queens(white).popcnt() * QUEEN_VALUE;

    let black_score = pos.board().pawns(black).popcnt() * PAWN_VALUE
        + pos.board().knights(black).popcnt() * KNIGHT_VALUE
        + pos.board().bishops(black).popcnt() * BISHOP_VALUE
        + pos.board().rooks(black).popcnt() * ROOK_VALUE
        + pos.board().queens(black).popcnt() * QUEEN_VALUE;

    (white_score as i32) - (black_score as i32)
}

fn calculate_pst_score(pos: &Position) -> i32 {
    let white = Color::White;
    let black = Color::Black;
    let board = pos.board();

    let mut score = 0;

    // ---- WHITE ----
    board
        .pawns(white)
        .for_each(|pawn| score += calculate_piece_pst(&PAWN_PST, pawn, white));

    board
        .knights(white)
        .for_each(|knight| score += calculate_piece_pst(&KNIGHT_PST, knight, white));

    board
        .bishops(white)
        .for_each(|bishop| score += calculate_piece_pst(&BISHOP_PST, bishop, white));

    board
        .rooks(white)
        .for_each(|rook| score += calculate_piece_pst(&ROOK_PST, rook, white));

    board
        .queens(white)
        .for_each(|queen| score += calculate_piece_pst(&QUEEN_PST, queen, white));

    board
        .king(white)
        .for_each(|king| score += calculate_piece_pst(&KING_MIDDLE_GAME_PST, king, white));

    // ---- BLACK ----
    board
        .pawns(black)
        .for_each(|pawn| score += calculate_piece_pst(&PAWN_PST, pawn, black));

    board
        .knights(black)
        .for_each(|knight| score += calculate_piece_pst(&KNIGHT_PST, knight, black));

    board
        .bishops(black)
        .for_each(|bishop| score += calculate_piece_pst(&BISHOP_PST, bishop, black));

    board
        .rooks(black)
        .for_each(|rook| score += calculate_piece_pst(&ROOK_PST, rook, black));

    board
        .queens(black)
        .for_each(|queen| score += calculate_piece_pst(&QUEEN_PST, queen, black));

    board
        .king(black)
        .for_each(|king| score += calculate_piece_pst(&KING_MIDDLE_GAME_PST, king, black));

    score
}

fn calculate_piece_pst(table: &[i32; 64], square: Square, side: Color) -> i32 {
    match side {
        Color::White => table[square.mirror_vertical().as_usize()],
        Color::Black => -table[square.as_usize()],
    }
}

#[cfg(test)]
mod tests {
    use types::{
        chess_move::{CastlingSide, Move},
        role::Role,
    };

    use super::*;

    #[test]
    fn eval_test() {
        let mut pos = Position::new();
        let moves = vec![
            Move::quiet(Role::Pawn, Square::D2, Square::D4),
            Move::quiet(Role::Pawn, Square::D7, Square::D5),
            Move::quiet(Role::Knight, Square::G1, Square::F3),
            Move::quiet(Role::Knight, Square::B8, Square::C6),
            Move::quiet(Role::Knight, Square::B1, Square::C3),
            Move::quiet(Role::Knight, Square::G8, Square::F6),
            Move::quiet(Role::Pawn, Square::E2, Square::E3),
            Move::quiet(Role::Pawn, Square::E7, Square::E6),
            Move::quiet(Role::Pawn, Square::A2, Square::A3),
            Move::quiet(Role::Pawn, Square::G7, Square::G6),
            Move::quiet(Role::Bishop, Square::F1, Square::B5),
            Move::quiet(Role::Pawn, Square::A7, Square::A6),
            Move::capture(Role::Bishop, Square::B5, Square::C6, Role::Knight),
            // Move::capture(Role::Pawn, Square::B7, Square::C6, Role::Bishop),
            // Move::quiet(Role::Knight, Square::F3, Square::E5),
            // Move::quiet(Role::Queen, Square::D8, Square::D6),
            // Move::quiet(Role::Pawn, Square::F2, Square::F3),
            // Move::quiet(Role::Pawn, Square::C6, Square::C5),
            // Move::castling(CastlingSide::WShort),
            // Move::quiet(Role::Pawn, Square::C5, Square::C4),
            // Move::quiet(Role::Pawn, Square::E3, Square::E4),
            // Move::quiet(Role::Pawn, Square::C7, Square::C6),
            // Move::quiet(Role::Bishop, Square::C1, Square::F4),
            // Move::quiet(Role::Pawn, Square::A6, Square::A5),
            // Move::quiet(Role::Knight, Square::C3, Square::A4),
            // Move::quiet(Role::Knight, Square::F6, Square::H5),
            // Move::quiet(Role::Queen, Square::D1, Square::D2),
            // Move::quiet(Role::Pawn, Square::F7, Square::F5),
            // Move::capture(Role::Pawn, Square::E4, Square::F5, Role::Pawn),
            // Move::capture(Role::Pawn, Square::E6, Square::F5, Role::Pawn),
            // Move::quiet(Role::Rook, Square::F1, Square::E1),
            // Move::quiet(Role::Bishop, Square::C8, Square::D7),
            // Move::capture(Role::Knight, Square::E5, Square::C6, Role::Pawn),
            // Move::quiet(Role::King, Square::E8, Square::F7),
            // Move::capture(Role::Bishop, Square::F4, Square::D6, Role::Queen),
            // Move::capture(Role::Bishop, Square::F8, Square::D6, Role::Bishop),
            // Move::quiet(Role::Knight, Square::C6, Square::E5),
            // Move::quiet(Role::King, Square::F7, Square::G7),
            // Move::capture(Role::Knight, Square::E5, Square::D7, Role::Bishop),
            // Move::quiet(Role::Pawn, Square::H7, Square::H6),
            // Move::quiet(Role::Knight, Square::A4, Square::B6),
            // Move::quiet(Role::Rook, Square::A8, Square::A7),
            // Move::quiet(Role::Knight, Square::D7, Square::C5),
            // Move::quiet(Role::Bishop, Square::D6, Square::F4),
            // Move::quiet(Role::Knight, Square::C5, Square::E6),
            // Move::quiet(Role::King, Square::G7, Square::F6),
            // Move::capture(Role::Knight, Square::E6, Square::F4, Role::Bishop),
            // Move::capture(Role::Knight, Square::H5, Square::F4, Role::Knight),
            // Move::capture(Role::Queen, Square::D2, Square::F4, Role::Knight),
            // Move::quiet(Role::Pawn, Square::G6, Square::G5),
            // Move::quiet(Role::Queen, Square::F4, Square::E5),
            // Move::quiet(Role::King, Square::F6, Square::F7),
            // Move::capture(Role::Queen, Square::E5, Square::H8, Role::Rook),
            // Move::quiet(Role::King, Square::F7, Square::G6),
            // Move::quiet(Role::Pawn, Square::G2, Square::G4),
            // Move::quiet(Role::Rook, Square::A7, Square::H7),
            // Move::capture(Role::Queen, Square::H8, Square::H7, Role::Rook),
            // Move::capture(Role::King, Square::G6, Square::H7, Role::Queen),
            // Move::quiet(Role::Rook, Square::E1, Square::E7),
            // Move::quiet(Role::King, Square::H7, Square::G6),
            // Move::quiet(Role::Rook, Square::A1, Square::E1),
            // Move::quiet(Role::King, Square::G6, Square::F6),
            // Move::quiet(Role::Rook, Square::E1, Square::E6),
        ];

        for mv in moves.into_iter() {
            println!("Making move -> {mv:?}");
            let _undo = pos.do_move(mv).unwrap();
            dbg!(&pos);
            dbg!(full_eval(&pos));
            println!();
            println!();
            println!();
        }
    }
}
