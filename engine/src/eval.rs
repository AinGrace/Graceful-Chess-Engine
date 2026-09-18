use lookup::{KING_PENALTY_FACTOR, piece_val};
use position::{board::Board, position::Position};
use types::{bitboard::ToBitboard, color::Color, score::Score, square::Square};

// TODO: compact this one

#[inline(always)]
pub fn static_eval(pos: &Position) -> Score {
    if pos.is_checkmate() {
        return Score::Mate(0);
    } else if pos.is_stalemate() {
        return Score::Draw;
    }

    let mobility = mobility(pos);
    let tapered_score = pos.tapered_score();

    let score = mobility + (tapered_score as i16);

    if pos.turn() == Color::White {
        Score::Centipawn(score)
    } else {
        Score::Centipawn(-score)
    }
}

#[inline(always)]
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

        eval = (piece_val(defender) - opponent_gain).max(0);

        let attacker = board.take_piece_at_checked(dest);
        board.set_piece_at(attacker, attk);
        board.set_piece_at(defender, dest);
    }

    eval
}

#[inline(always)]
fn mobility(pos: &Position) -> i16 {
    let board = pos.board();
    let occupied = board.occupied().as_u64();

    let mut score = 0i16;

    for color in [Color::White, Color::Black] {
        let sign = if color == Color::White { 1 } else { -1 };
        let own = board.by_color(color);

        for sq in board.knights(color) {
            score += sign * (lookup::knight_attacks(sq).to_bb() & !own).popcnt() as i16;
        }

        for sq in board.bishops(color) {
            score += sign * (lookup::bishop_attacks(sq, occupied).to_bb() & !own).popcnt() as i16;
        }

        for sq in board.rooks(color) {
            score += sign * (lookup::rook_attacks(sq, occupied).to_bb() & !own).popcnt() as i16;
        }

        for sq in board.queens(color) {
            score += sign * (lookup::queen_attacks(sq, occupied).to_bb() & !own).popcnt() as i16;
        }
    }

    score
}

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

#[cfg(test)]
mod tests {

    use position::fen::Fen;

    use super::*;

    fn pos_from_fen(fen: &str) -> Position {
        let fen = fen.parse::<Fen>().expect("valid FEN");
        Position::from_fen(fen).expect("valid position")
    }

    #[test]
    fn static_eval_example() {
        let pos = Position::new();
        let eval = static_eval(&pos);

        dbg!(eval);
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
