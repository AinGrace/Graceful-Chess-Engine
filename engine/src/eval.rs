use std::{
    fmt::Display,
    ops::{Neg, Sub},
};

use lookup::{
    MG_BISHOP_PST, BISHOP_VALUE, EG_KING_PST, MG_KING_PST, KING_PENALTY_FACTOR, MG_KNIGHT_PST, KNIGHT_VALUE, MG_PAWN_PST, PAWN_VALUE, MG_QUEEN_PST, QUEEN_VALUE, MG_ROOK_PST, ROOK_VALUE, piece_val,
};
use position::{board::Board, position::Position};
use types::{color::Color, piece::Piece, role::Role, square::Square};

// TODO: compact this one
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Score {
    Centipawn(i16),
    Mate(i8),
    Draw,
    Abort,
}

impl Score {
    pub fn value(self) -> i16 {
        match self {
            Score::Mate(val) if val > 0 => 10_000 - val as i16,
            Score::Mate(val) => -10_000 - val as i16,
            Score::Centipawn(val) => val,
            Score::Draw => 0,
            Score::Abort => 0,
        }
    }

    pub fn step(self) -> Self {
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
            Score::Abort => Score::Abort,
        }
    }
}

impl Sub for Score {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Centipawn(self.value() - rhs.value())
    }
}

impl Sub<i16> for Score {
    type Output = Self;

    fn sub(self, rhs: i16) -> Self::Output {
        Self::Centipawn(self.value() - rhs)
    }
}

impl Display for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Score::Centipawn(val) => format!("cp {}", val),
                Score::Mate(val) => format!("mate {}", val),
                Score::Draw => format!("cp 0"),
                Score::Abort => format!("ABORTED"),
            }
        )
    }
}

impl Default for Score {
    fn default() -> Self {
        Self::Mate(-1)
    }
}

pub fn static_eval(pos: &Position) -> Score {
        if pos.is_checkmate() {
        return Score::Mate(0);
    } else if pos.is_stalemate() {
        return Score::Draw;
    }

    let mobility = mobility(pos);
    let tapered_score = pos.tapered_score();

    let score = mobility + tapered_score;

    if pos.turn() == Color::White {
        Score::Centipawn(score)
    } else {
        Score::Centipawn(-score)
    }
}

// TODO
pub fn eval_see(board: &mut Board, dest: Square, us: Color) -> i16 {
    let mut eval = 0;

    if board.peek(dest).is_some()
        && let Some(attk) = board.lva_to(dest, us)
    {
        let attacker = board.take_piece_at_checked(attk);
        let defender = board
            .replace_piece_at(attacker, dest)
            .expect("defender exists");

        let opponent_gain = eval_see(board, dest, !us);

        eval = piece_val(defender) - opponent_gain;

        let attacker = board.take_piece_at_checked(dest);
        board.set_piece_at(attacker, attk);
        board.set_piece_at(defender, dest);
    }

    eval
}

fn mobility(pos: &Position) -> i16 {
    let us = pos.legal_moves_for(pos.turn());
    let them = pos.legal_moves_for(!pos.turn());

    (us.len() as isize - them.len() as isize) as i16
}

fn is_endgame(board: &Board) -> bool {
    let pieces = board.non_king_pieces_of(Color::White) & board.non_king_pieces_of(Color::Black);

    if pieces.popcnt() <= 4
        || (material_score_of_white(board) < 1300 && material_score_of_black(board) < 1300)
    {
        true
    } else {
        false
    }
}

#[rustfmt::skip]
fn material_score_of_white(board: &Board) -> i16 {
    let us = Color::White;

    let pawns   = board.pawns(us);
    let knights = board.knights(us);
    let bishops = board.bishops(us);
    let rooks   = board.rooks(us);
    let queens  = board.queens(us);

    let pawns_score   = pawns.popcnt()   as i16 * PAWN_VALUE;
    let knights_score = knights.popcnt() as i16 * KNIGHT_VALUE;
    let bishops_score = bishops.popcnt() as i16 * BISHOP_VALUE;
    let rooks_score   = rooks.popcnt()   as i16 * ROOK_VALUE;
    let queens_score  = queens.popcnt()  as i16 * QUEEN_VALUE;

    pawns_score + knights_score + bishops_score + rooks_score + queens_score
}

#[rustfmt::skip]
fn material_score_of_black(board: &Board) -> i16 {
    let us = Color::Black;

    let pawns   = board.pawns(us);
    let knights = board.knights(us);
    let bishops = board.bishops(us);
    let rooks   = board.rooks(us);
    let queens  = board.queens(us);

    let pawns_score   = pawns.popcnt()   as i16 * PAWN_VALUE;
    let knights_score = knights.popcnt() as i16 * KNIGHT_VALUE;
    let bishops_score = bishops.popcnt() as i16 * BISHOP_VALUE;
    let rooks_score   = rooks.popcnt()   as i16 * ROOK_VALUE;
    let queens_score  = queens.popcnt()  as i16 * QUEEN_VALUE;

    pawns_score + knights_score + bishops_score + rooks_score + queens_score
}

#[rustfmt::skip]
fn material_score(board: &Board) -> i16 {
    material_score_of_white(board) - material_score_of_black(board)
}

fn calculate_pst_score(board: &Board) -> i16 {
    let white = Color::White;
    let black = Color::Black;

    let mut score = 0;

    // ---- WHITE ----
    board
        .pawns(white)
        .for_each(|pawn| score += piece_pst(&MG_PAWN_PST, pawn, white));

    board
        .knights(white)
        .for_each(|knight| score += piece_pst(&MG_KNIGHT_PST, knight, white));

    board
        .bishops(white)
        .for_each(|bishop| score += piece_pst(&MG_BISHOP_PST, bishop, white));

    board
        .rooks(white)
        .for_each(|rook| score += piece_pst(&MG_ROOK_PST, rook, white));

    board
        .queens(white)
        .for_each(|queen| score += piece_pst(&MG_QUEEN_PST, queen, white));

    board.king(white).for_each(|king| {
        if is_endgame(board) {
            score += piece_pst(&EG_KING_PST, king, white);
            score += king_dist_eval(board, white);
        } else {
            score += piece_pst(&MG_KING_PST, king, white);
        }
    });

    // ---- BLACK ----
    board
        .pawns(black)
        .for_each(|pawn| score += piece_pst(&MG_PAWN_PST, pawn, black));

    board
        .knights(black)
        .for_each(|knight| score += piece_pst(&MG_KNIGHT_PST, knight, black));

    board
        .bishops(black)
        .for_each(|bishop| score += piece_pst(&MG_BISHOP_PST, bishop, black));

    board
        .rooks(black)
        .for_each(|rook| score += piece_pst(&MG_ROOK_PST, rook, black));

    board
        .queens(black)
        .for_each(|queen| score += piece_pst(&MG_QUEEN_PST, queen, black));

    board.king(black).for_each(|king| {
        if is_endgame(board) {
            score += piece_pst(&EG_KING_PST, king, white);
            score += king_dist_eval(board, black);
        } else {
            score += piece_pst(&MG_KING_PST, king, white);
        }
    });

    score
}

/// in endgame where only a few pieces remain kings are encouraged to be close to each other
///
/// apply score penalty otherwise
fn king_dist_eval(board: &Board, us: Color) -> i16 {
    let non_king_pieces =
        board.non_king_pieces_of(Color::White) | board.non_king_pieces_of(Color::Black);
    let pawns = board.pawns(Color::White) | board.pawns(Color::Black);

    if non_king_pieces.popcnt() == 1 && !non_king_pieces.intersects(pawns) {
        let distance = board.dist_between_kings();
        let advantage = if board.by_color(us).popcnt() == 2 {
            1
        } else {
            -1
        };

        let king_penalty = -distance * advantage * KING_PENALTY_FACTOR;

        king_penalty
    } else {
        0
    }
}

fn piece_pst(table: &[i16; 64], square: Square, side: Color) -> i16 {
    match side {
        Color::White => table[square.mirror_vertical().as_usize()],
        Color::Black => -table[square.as_usize()],
    }
}

#[cfg(test)]
mod tests {

    use position::fen::Fen;

    use super::*;

    fn pos_from_fen(fen: &str) -> Position {
        let fen = fen.parse::<Fen>().expect("valid FEN");
        Position::from_fen(fen).expect("valid position")
    }

    mod see {
        use lookup::{BISHOP_VALUE, KNIGHT_VALUE, PAWN_VALUE, QUEEN_VALUE, ROOK_VALUE};
        use types::{color::Color, square::Square};

        use crate::eval::{eval_see, tests::pos_from_fen};

        fn see(fen: &str, dest: Square, them: Color) -> i16 {
            let pos = pos_from_fen(fen);
            let mut board = pos.board().clone();
            println!("{:#?}", pos);

            eval_see(&mut board, dest, them)
        }
        #[test]
        fn see_pawn_takes_pawn() {
            let score = see(
                "4k3/8/8/3p4/4P3/8/8/4K3 w - - 0 1",
                Square::D5,
                Color::White,
            );

            assert_eq!(score, PAWN_VALUE);
        }

        #[test]
        fn see_pawn_takes_knight() {
            let score = see(
                "4k3/8/8/3n4/4P3/8/8/4K3 w - - 0 1",
                Square::D5,
                Color::White,
            );

            assert_eq!(score, KNIGHT_VALUE);
        }

        #[test]
        fn see_knight_takes_queen() {
            let score = see(
                "4k3/3q4/8/4N3/8/8/8/4K3 w - - 0 1",
                Square::D7,
                Color::White,
            );

            assert_eq!(score, QUEEN_VALUE - KNIGHT_VALUE);
        }

        #[test]
        fn see_equal_pawn_exchange_is_zero() {
            let score = see(
                "4k3/8/4p3/3p4/4P3/8/8/4K3 w - - 0 1",
                Square::D5,
                Color::White,
            );

            assert_eq!(score, 0);
        }

        #[test]
        fn see_knight_takes_pawn_and_is_recaptured() {
            let score = see(
                "4k3/5n2/3p4/8/4N3/8/8/4K3 w - - 0 1",
                Square::D6,
                Color::White,
            );

            assert_eq!(score, 0);
        }

        #[test]
        fn see_rook_takes_rook_and_is_recaptured() {
            let score = see(
                "4r1k1/8/8/4r3/8/8/8/4R1K1 w - - 0 1",
                Square::E5,
                Color::White,
            );

            assert_eq!(score, 0);
        }

        #[test]
        fn see_winning_exchange() {
            let score = see(
                "4k3/5n2/8/4q3/8/8/8/4R1K1 w - - 0 1",
                Square::E5,
                Color::White,
            );

            assert_eq!(score, QUEEN_VALUE - ROOK_VALUE);
        }

        #[test]
        fn see_losing_exchange_returns_zero() {
            let score = see(
                "4rk2/8/8/4p3/8/8/8/4Q1K1 w - - 0 1",
                Square::E5,
                Color::White,
            );

            assert_eq!(score, 0);
        }

        #[test]
        fn see_bishop_takes_rook_and_gets_recaptured_by_knight() {
            let score = see(
                "4k3/5n2/8/4r3/8/8/8/B5K1 w - - 0 1",
                Square::E5,
                Color::White,
            );

            assert_eq!(score, ROOK_VALUE - BISHOP_VALUE);
        }

        #[test]
        fn see_uses_least_valuable_attacker() {
            let score = see(
                "4k3/8/8/3n4/2p5/8/8/3R2K1 w - - 0 1",
                Square::D5,
                Color::White,
            );

            assert_eq!(score, KNIGHT_VALUE);
        }

        #[test]
        fn see_prefers_knight_over_bishop() {
            let score = see(
                "4k3/1b6/8/3r4/8/4n3/8/3R2K1 w - - 0 1",
                Square::D5,
                Color::White,
            );

            assert_eq!(score, 0);
        }

        #[test]
        fn see_handles_multiple_exchange_sequence() {
            let score = see(
                "4k3/5n2/8/4q3/8/8/2B5/4R1K1 w - - 0 1",
                Square::E5,
                Color::White,
            );

            assert_eq!(score, QUEEN_VALUE - KNIGHT_VALUE + BISHOP_VALUE);
        }

        #[test]
        fn see_returns_zero_when_no_attacker_exists() {
            let score = see("4k3/3q4/8/8/8/8/8/4K3 w - - 0 1", Square::D7, Color::White);

            assert_eq!(score, 0);
        }

        #[test]
        fn see_returns_zero_on_empty_destination() {
            let score = see("4k3/8/8/8/4P3/8/8/4K3 w - - 0 1", Square::D5, Color::White);

            assert_eq!(score, 0);
        }

        #[test]
        fn see_king_capture_is_not_used_as_normal_material() {
            let score = see("4k3/8/8/3K4/8/8/8/8 w - - 0 1", Square::E8, Color::White);

            assert_eq!(score, 0);
        }

        #[test]
        fn see_does_not_modify_board() {
            let pos = pos_from_fen("4k3/5n2/8/4q3/8/8/2B5/4R1K1 w - - 0 1");

            let mut board = pos.board().clone();
            let before = board.clone();

            let _ = eval_see(&mut board, Square::E5, Color::White);

            assert_eq!(board, before);
        }

        #[test]
        fn see_does_not_modify_board_after_deep_exchange() {
            let pos = pos_from_fen("4rk2/5n2/8/4q3/8/8/2B5/4R1K1 w - - 0 1");

            let mut board = pos.board().clone();
            let before = board.clone();

            let _ = eval_see(&mut board, Square::E5, Color::White);

            assert_eq!(board, before);
        }

        #[test]
        fn see_is_symmetric_for_white_and_black() {
            let white_score = see(
                "4k3/8/8/3p4/4P3/8/8/4K3 w - - 0 1",
                Square::D5,
                Color::White,
            );

            let black_score = see(
                "4k3/8/8/3p4/4P3/8/8/4K3 b - - 0 1",
                Square::E4,
                Color::Black,
            );

            assert_eq!(white_score, black_score);
        }
    }
}
