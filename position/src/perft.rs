use arrayvec::ArrayVec;
use shakmaty::{Chess, Move as TheirMove, Position, fen::Fen as TheirFen};
use types::{chess_move::Move, role::Role, square::Square};

use crate::{chessboard::ChessBoard, fen::Fen};

#[derive(Debug, Clone)]
struct HistoryChessBoard {
    inner: ChessBoard,
    history: Vec<Move>,
}

impl HistoryChessBoard {
    fn legal_moves(&self) -> ArrayVec<Move, 218> {
        self.inner.legal_moves()
    }

    fn do_move_inner_checked(&mut self, mv: Move) {
        self.inner.do_move_inner_checked(mv);
        self.history.push(mv);
    }
}

pub fn perft_classic(chessboard: &ChessBoard, dep: u32) -> u64 {
    let mut nodes = 0;

    if dep == 0 {
        return 1;
    }

    let moves = chessboard.legal_moves();

    if dep == 1 {
        return moves.len() as u64;
    }

    for mv in moves {
        let mut board_clone = chessboard.clone();
        board_clone.do_move_inner_checked(mv);
        nodes += perft_classic(&board_clone, dep - 1);
    }

    nodes
}

pub fn perft(chessboard: &ChessBoard, dep: u32) -> u64 {
    if dep == 0 {
        return 1;
    }

    let moves = chessboard.legal_moves();

    if dep == 1 {
        return moves.len() as u64;
    }

    moves
        .iter()
        .map(|move_| {
            let mut board_clone = chessboard.clone();
            board_clone.do_move_inner_checked(*move_);
            perft(&board_clone, dep - 1)
        })
        .sum()
}

fn perft_history(chessboard: &HistoryChessBoard, dep: u32) -> u64 {
    if dep == 0 {
        return 1;
    }

    let moves = chessboard.legal_moves();

    if dep == 1 {
        return moves.len() as u64;
    }

    moves
        .iter()
        .map(|move_| {
            let mut board_clone = chessboard.clone();
            if matches!(
                move_,
                Move::Standart {
                    role: Role::Pawn,
                    from: Square::H2,
                    to: Square::H1,
                    capture: Some(Role::Rook),
                    promotion: Some(Role::Queen)
                }
            ) {
                println!("----------------------");
                println!("history -> {:#?}", board_clone.history);
                println!("potential");
            }
            let _res = board_clone.do_move_inner_checked(*move_);
            perft_history(&board_clone, dep - 1)
        })
        .sum()
}
fn perft_comparing_inner(our: HistoryChessBoard, their: Chess, dep: u32) -> u64 {
    if dep == 0 {
        return 1;
    }

    let our_moves = our.legal_moves();
    let their_moves = their.legal_moves();

    if our_moves.len() != their_moves.len() {
        println!("We generated -> {}", our_moves.len());
        println!("our moves -> {:#?}", our_moves);

        println!("They generated -> {}", their_moves.len());
        println!("their moves -> {:#?}", their_moves);

        println!("Move history -> {:#?}", our.history);

        println!("OUR board -> {:#?}", our.inner);

        panic!("Move mismatch");
    }

    their_moves
        .iter()
        .map(|move_| {
            let mut their_board_clone = their.clone();
            their_board_clone.play_unchecked(*move_);

            let mut our_board_clone = our.clone();
            our_board_clone.do_move_inner_checked(translate_move(*move_));

            perft_comparing_inner(our_board_clone, their_board_clone, dep - 1)
        })
        .sum()
}

fn translate_move(their_move: TheirMove) -> Move {
    match their_move {
        TheirMove::Normal {
            role,
            from,
            capture,
            to,
            promotion,
        } => Move::Standart {
            role: Role::new(role.char()).unwrap(),
            from: Square::from_u32_checked(from.to_u32()),
            to: Square::from_u32_checked(to.to_u32()),
            capture: capture.map_or(None, |r| Role::new(r.char())),
            promotion: promotion.map_or(None, |p| Some(Role::new(p.char()).unwrap())),
        },
        TheirMove::EnPassant { from, to } => Move::EnPassant {
            from: Square::from_u32_checked(from.to_u32()),
            to: Square::from_u32_checked(to.to_u32()),
        },
        TheirMove::Castle { king, rook } => Move::Castling {
            king: Square::from_u32_checked(king.to_u32()),
            rook: Square::from_u32_checked(rook.to_u32()),
        },

        TheirMove::Put { .. } => panic!("IMPOSSIBLE"),
    }
}

#[test]
#[ignore = "to be onvoked manually for debugging"]
fn perft_comparing() {
    let raw_fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let our_fen: Fen = raw_fen.parse().unwrap();
    let their_fen = TheirFen::from_ascii(raw_fen.as_bytes()).unwrap();

    let other_chessboard = their_fen
        .into_position::<Chess>(shakmaty::CastlingMode::Standard)
        .unwrap();
    let chessboard: ChessBoard = our_fen.into_chessboard().unwrap();

    let res = perft_comparing_inner(
        HistoryChessBoard {
            inner: chessboard,
            history: vec![],
        },
        other_chessboard,
        6,
    );

    println!("{res}");
}

#[test]
#[ignore = "to be onvoked manually for debugging"]
fn mismatch_test() {
    let raw_fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let fen: Fen = raw_fen.parse().unwrap();
    let mut chessboard: ChessBoard = fen.into_chessboard().unwrap();

    chessboard.do_move_inner_checked(Move::capture(
        Role::Pawn,
        Square::G2,
        Square::H3,
        Role::Pawn,
    ));
    chessboard.do_move_inner_checked(Move::capture(
        Role::Pawn,
        Square::E6,
        Square::D5,
        Role::Pawn,
    ));
    chessboard.do_move_inner_checked(Move::capture(
        Role::Pawn,
        Square::E4,
        Square::D5,
        Role::Pawn,
    ));
    chessboard.do_move_inner_checked(Move::quiet(Role::King, Square::E8, Square::D8));
    chessboard.do_move_inner_checked(Move::quiet(Role::Knight, Square::E5, Square::C6));

    dbg!(&chessboard);
    let moves = chessboard.legal_moves();
    dbg!(moves);
}

#[test]
fn perft_depth_0_equals_1() {
    let chessboard = ChessBoard::new();
    let res = perft(&chessboard, 0);
    assert_eq!(res, 1);
}

#[test]
fn perft_depth_1_equals_20() {
    let chessboard = ChessBoard::new();
    let res = perft(&chessboard, 1);
    assert_eq!(res, 20);
}

#[test]
fn perft_depth_2_equals_400() {
    let chessboard = ChessBoard::new();
    let res = perft(&chessboard, 2);
    assert_eq!(res, 400);
}

#[test]
fn perft_depth_3_equals_8_902() {
    let chessboard = ChessBoard::new();
    let res = perft(&chessboard, 3);
    assert_eq!(res, 8902);
}

#[test]
fn perft_depth_4_equals_197_281() {
    let chessboard = ChessBoard::new();
    let res = perft(&chessboard, 4);
    assert_eq!(res, 197281);
}

#[test]
fn perft_depth_5_equals_4_865_609() {
    let chessboard = ChessBoard::new();
    let res = perft(&chessboard, 5);
    assert_eq!(res, 4865609);
}

#[test]
fn perft_depth_6_equals_119_060_324() {
    let chessboard = ChessBoard::new();
    let res = perft(&chessboard, 6);
    assert_eq!(res, 119060324);
}

#[test]
fn perft_depth_7_equals_3_195_901_860() {
    let chessboard = ChessBoard::new();
    let res = perft(&chessboard, 7);
    assert_eq!(res, 3195901860);
}

#[test]
#[ignore]
fn perft_depth_8_equals_84_998_978_956() {
    let chessboard = ChessBoard::new();
    let res = perft(&chessboard, 8);
    assert_eq!(res, 84_998_978_956);
}

#[test]
fn perft_custom_position_1() {
    let raw_fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";

    let chessboard: ChessBoard = Fen::new(raw_fen).unwrap().into_chessboard().unwrap();

    assert_eq!(perft(&chessboard.clone(), 1), 48);
    assert_eq!(perft(&chessboard.clone(), 2), 2039);
    assert_eq!(perft(&chessboard.clone(), 3), 97862);
    assert_eq!(perft(&chessboard.clone(), 4), 4085603);
    assert_eq!(perft(&chessboard.clone(), 5), 193690690);
    // assert_eq!(perft(&chessboard.clone(), 6), 8031647685);
}

#[test]
fn perft_custom_position_2() {
    let raw_fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1 ";

    let chessboard = Fen::new(raw_fen).unwrap().into_chessboard().unwrap();

    assert_eq!(perft(&chessboard.clone(), 1), 14);
    assert_eq!(perft(&chessboard.clone(), 2), 191);
    assert_eq!(perft(&chessboard.clone(), 3), 2812);
    assert_eq!(perft(&chessboard.clone(), 4), 43238);
    assert_eq!(perft(&chessboard.clone(), 5), 674624);
    assert_eq!(perft(&chessboard.clone(), 6), 11030083);
    assert_eq!(perft(&chessboard.clone(), 7), 178633661);
    assert_eq!(perft(&chessboard.clone(), 8), 3009794393);
}

#[test]
fn perft_custom_position_3() {
    let raw_fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";

    let chessboard = Fen::new(raw_fen).unwrap().into_chessboard().unwrap();

    assert_eq!(perft(&chessboard.clone(), 1), 6);
    assert_eq!(perft(&chessboard.clone(), 2), 264);
    assert_eq!(perft(&chessboard.clone(), 3), 9467);
    assert_eq!(perft(&chessboard.clone(), 4), 422333);
    assert_eq!(perft(&chessboard.clone(), 5), 15833292);
    assert_eq!(perft(&chessboard.clone(), 6), 706045033);
}

#[test]
fn perft_custom_position_4() {
    let raw_fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";

    let chessboard = Fen::new(raw_fen).unwrap().into_chessboard().unwrap();

    assert_eq!(perft(&chessboard.clone(), 1), 44);
    assert_eq!(perft(&chessboard.clone(), 2), 1486);
    assert_eq!(perft(&chessboard.clone(), 3), 62379);
    assert_eq!(perft(&chessboard.clone(), 4), 2103487);
    assert_eq!(perft(&chessboard.clone(), 5), 89941194);
}

#[test]
#[ignore = "to be manually invoked, because current move generator is not fast enough"]
fn perft_custom_position_5() {
    let raw_fen = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10";

    let chessboard = Fen::new(raw_fen).unwrap().into_chessboard().unwrap();

    assert_eq!(perft(&chessboard.clone(), 1), 46);
    assert_eq!(perft(&chessboard.clone(), 2), 2079);
    assert_eq!(perft(&chessboard.clone(), 3), 89890);
    assert_eq!(perft(&chessboard.clone(), 4), 3894594);
    assert_eq!(perft(&chessboard.clone(), 5), 164075551);
    assert_eq!(perft(&chessboard.clone(), 6), 6923051137);
    assert_eq!(perft(&chessboard.clone(), 7), 287188994746);
}
