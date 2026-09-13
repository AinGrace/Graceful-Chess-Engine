use std::collections::HashSet;

use rayon::prelude::*;
use shakmaty::{Chess, Move as TheirMove, Position as TheirPosition, fen::Fen as TheirFen};
use types::{MoveList, chess_move::Move, role::Role, square::Square};

use crate::{fen::Fen, position::Position};

#[test]
#[ignore = "to be onvoked manually for debugging"]
fn perft_comparing() {
    let raw_fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let our_fen: Fen = raw_fen.parse().unwrap();
    let their_fen = TheirFen::from_ascii(raw_fen.as_bytes()).unwrap();

    let other_chessboard = their_fen
        .into_position::<Chess>(shakmaty::CastlingMode::Standard)
        .unwrap();
    let chessboard: Position = our_fen.try_to_position().unwrap();

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
    let mut chessboard: Position = fen.try_to_position().unwrap();

    dbg!(&chessboard);

    chessboard
        .do_move(Move::capture(Square::G2, Square::H3))
        .unwrap();
    println!("after g2h3: {:?}", &chessboard);
    chessboard
        .do_move(Move::capture(Square::E6, Square::D5))
        .unwrap();
    println!("after e6d5: {:?}", &chessboard);
    chessboard
        .do_move(Move::capture(Square::E4, Square::D5))
        .unwrap();
    println!("after e4d5: {:?}", &chessboard);
    chessboard
        .do_move(Move::capture(Square::B4, Square::C3))
        .unwrap();
    println!("after b4c3: {:?}", &chessboard);
    chessboard
        .do_move(Move::queen_castle(Square::E1, Square::C1))
        .unwrap();
    println!("after a2a4: {:?}", &chessboard);
}

#[test]
fn perft_depth_0_equals_1() {
    let chessboard = Position::new();
    let res = perft(&chessboard, 0);
    assert_eq!(res, 1);
}

#[test]
fn perft_depth_1_equals_20() {
    let chessboard = Position::new();
    let res = perft(&chessboard, 1);
    assert_eq!(res, 20);
}

#[test]
fn perft_depth_2_equals_400() {
    let chessboard = Position::new();
    let res = perft(&chessboard, 2);
    assert_eq!(res, 400);
}

#[test]
fn perft_depth_3_equals_8_902() {
    let chessboard = Position::new();
    let res = perft(&chessboard, 3);
    assert_eq!(res, 8902);
}

#[test]
fn perft_depth_4_equals_197_281() {
    let chessboard = Position::new();
    let res = perft(&chessboard, 4);
    assert_eq!(res, 197281);
}

#[test]
fn perft_depth_5_equals_4_865_609() {
    let chessboard = Position::new();
    let res = perft(&chessboard, 5);
    assert_eq!(res, 4865609);
}

#[test]
fn perft_depth_6_equals_119_060_324() {
    let chessboard = Position::new();
    let res = perft(&chessboard, 6);
    assert_eq!(res, 119060324);
}

#[test]
fn perft_depth_7_equals_3_195_901_860() {
    let mut chessboard = Position::new();
    let res = perft(&mut chessboard, 7);
    assert_eq!(res, 3195901860);
}

#[test]
#[ignore]
fn perft_depth_8_equals_84_998_978_956() {
    let chessboard = Position::new();
    let res = perft_parallel(&chessboard, 8);
    assert_eq!(res, 84_998_978_956);
}

#[test]
fn perft_custom_position_1() {
    let raw_fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";

    let chessboard: Position = Fen::new(raw_fen).unwrap().try_to_position().unwrap();

    assert_eq!(perft(&chessboard.clone(), 1), 48);
    assert_eq!(perft(&chessboard.clone(), 2), 2039);
    assert_eq!(perft(&chessboard.clone(), 3), 97862);
    assert_eq!(perft(&chessboard.clone(), 4), 4085603);
    assert_eq!(perft(&chessboard.clone(), 5), 193690690);
    assert_eq!(perft(&chessboard.clone(), 6), 8031647685);
}

#[test]
fn perft_custom_position_2() {
    let raw_fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1 ";

    let chessboard = Fen::new(raw_fen).unwrap().try_to_position().unwrap();

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

    let chessboard = Fen::new(raw_fen).unwrap().try_to_position().unwrap();

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

    let chessboard = Fen::new(raw_fen).unwrap().try_to_position().unwrap();

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

    let chessboard = Fen::new(raw_fen).unwrap().try_to_position().unwrap();

    assert_eq!(perft(&chessboard.clone(), 1), 46);
    assert_eq!(perft(&chessboard.clone(), 2), 2079);
    assert_eq!(perft(&chessboard.clone(), 3), 89890);
    assert_eq!(perft(&chessboard.clone(), 4), 3894594);
    assert_eq!(perft(&chessboard.clone(), 5), 164075551);
    assert_eq!(perft(&chessboard.clone(), 6), 6923051137);
    assert_eq!(perft(&chessboard.clone(), 7), 287188994746);
}

#[test]
fn perft_suite() {
    let suite = PerftSuite::new();

    suite.entries.into_iter().for_each(|entry| {
        let fen = entry.fen;
        let chessboard = fen.clone().try_to_position().expect("Fen should be valid");

        entry
            .depth_values
            .iter()
            .enumerate()
            .for_each(|(depth, nodes_count)| {
                assert_eq!(
                    perft(&chessboard.clone(), (depth + 1) as u32),
                    *nodes_count,
                    "the nodes cound of depth {depth} for fen {fen} should be {nodes_count}"
                );
            });
    });
}

/// a wrapper around ChessBoard that preserves the move history
#[derive(Debug, Clone)]
struct HistoryChessBoard {
    inner: Position,
    history: Vec<Move>,
}

impl HistoryChessBoard {
    fn legal_moves(&self) -> MoveList {
        self.inner.legal_moves()
    }

    fn do_move_inner_checked(&mut self, mv: Move) {
        match self.inner.do_move(mv) {
            Ok(_) => (),
            Err(e) => {
                println!("{e}");
                println!("fen -> {}", self.inner.to_fen());
                panic!()
            }
        }
        self.history.push(mv);
    }
}

struct PerftTranspositions {
    entries: Vec<PerftTTEntry>,
    size: usize,
}

#[derive(Clone, Copy)]
struct PerftTTEntry {
    hash: u64,
    depth: u32,
    count: u64,
}

impl PerftTranspositions {
    fn new(mb: usize) -> Self {
        let size = (mb * 1024 * 1024) / size_of::<PerftTTEntry>();

        Self {
            entries: vec![
                PerftTTEntry {
                    hash: 0,
                    depth: 0,
                    count: 0
                };
                size
            ],
            size,
        }
    }

    fn index(&self, hash: u64) -> usize {
        (hash as usize) % self.size
    }

    pub fn get(&self, hash: u64, depth: u32) -> Option<u64> {
        let entry = &self.entries[self.index(hash)];
        if entry.hash == hash && entry.depth == depth {
            Some(entry.count)
        } else {
            None
        }
    }

    pub fn insert(&mut self, hash: u64, depth: u32, count: u64) {
        let idx = self.index(hash);
        self.entries[idx] = PerftTTEntry { hash, depth, count };
    }
}

#[derive(Debug)]
struct PerftSuite {
    entries: Vec<PerftSuiteEntry>,
}

#[derive(Debug)]
struct PerftSuiteEntry {
    fen: Fen,
    depth_values: Vec<u64>,
}

impl PerftSuite {
    fn new() -> Self {
        let raw_suite = include_str!("../../perftsuite.epd");
        assert!(raw_suite.len() > 0, "perftsuite should not be empty");

        let mut entries = vec![];
        raw_suite.lines().for_each(|line| {
            let mut spliterator = line.split(';');

            let raw_fen = spliterator.next().expect("first split entry should be fen");
            let fen = Fen::new(raw_fen).expect(&format!("{raw_fen} should be valid fen"));

            let depth_values = spliterator
                .map(|prefixed_depth_value| {
                    let (_ignored_prefix, depth_value) = prefixed_depth_value.split_at(3);
                    let depth_value = depth_value
                        .trim()
                        .parse::<u64>()
                        .expect(&format!("{depth_value} should be valid for u64"));

                    depth_value
                })
                .collect();

            entries.push(PerftSuiteEntry { fen, depth_values });
        });

        PerftSuite { entries }
    }
}

// a perft functions with transposition table
fn perft_tt(chessboard: &Position, dep: u32, tt: &mut PerftTranspositions) -> u64 {
    if dep == 0 {
        return 1;
    }

    let moves = chessboard.legal_moves();

    if dep == 1 {
        return moves.len() as u64;
    }

    if let Some(count) = tt.get(chessboard.zobrist_hash(), dep) {
        return count;
    }

    let count = moves
        .iter()
        .map(|move_| {
            let mut board_clone = chessboard.clone();
            //SAFETY move_ is safely generated through legal_moves()
            unsafe { board_clone.do_move_unchecked(*move_) };
            perft_tt(&board_clone, dep - 1, tt)
        })
        .sum();

    tt.insert(chessboard.zobrist_hash(), dep, count);

    count
}

// a parallel perft
fn perft_parallel(chessboard: &Position, dep: u32) -> u64 {
    if dep <= 1 {
        return perft_tt(chessboard, dep, &mut PerftTranspositions::new(64));
    }

    chessboard
        .legal_moves()
        .par_iter()
        .map(|move_| {
            let mut board_clone = chessboard.clone();
            //SAFETY move_ is safely generated through legal_moves()
            unsafe { board_clone.do_move_unchecked(*move_) };
            let mut tt = PerftTranspositions::new(128);
            perft_tt(&board_clone, dep - 1, &mut tt)
        })
        .sum()
}

// regular perft function
pub fn perft_make_unmake(pos: &mut Position, dep: u32) -> u64 {
    if dep == 0 {
        return 1;
    }

    let moves = pos.legal_moves();

    if dep == 1 {
        return moves.len() as u64;
    }

    moves
        .iter()
        .map(|move_| {
            //SAFETY move_ is safely generated through legal_moves()
            let undo = unsafe { pos.do_move_unchecked(*move_) };
            let nodes = perft(pos, dep - 1);
            unsafe { pos.undo_move(undo) };

            nodes
        })
        .sum()
}

// regular perft function
pub fn perft(chessboard: &Position, dep: u32) -> u64 {
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
            //SAFETY move_ is safely generated through legal_moves()
            unsafe { board_clone.do_move_unchecked(*move_) };
            perft(&board_clone, dep - 1)
        })
        .sum()
}

fn perft_comparing_inner(our: HistoryChessBoard, their: Chess, dep: u32) -> u64 {
    if dep == 0 {
        return 1;
    }

    let our_moves = our.legal_moves();
    let their_moves = their.legal_moves();

    let their_fen = TheirFen::from_position(&their, shakmaty::EnPassantMode::Always).to_string();
    let our_fen = our.inner.to_fen().to_string();

    if their_fen != our_fen {
        println!("Move history -> {:#?}", our.history);

        let our_set: HashSet<Move> = HashSet::from_iter(our_moves.iter().copied());
        let their_set = HashSet::from_iter(their_moves.iter().map(|m| translate_move(*m)));

        let differences: Vec<&Move> = our_set.symmetric_difference(&their_set).collect();
        
        println!("OUR board -> {:#?}", our.inner);
        println!("THEIR board -> {:#?}", their);

        println!("We generated -> {}", our_moves.len());
        println!("They generated -> {}", their_moves.len());
        println!("diff -> {differences:#?}");

        println!("{their_fen}");
        println!("{our_fen}");

        panic!("FEN mismatch");
    }

    if our_moves.len() != their_moves.len() {
        println!("our moves -> {:#?}", our_moves);
        println!("their moves -> {:#?}", their_moves);

        println!("Move history -> {:#?}", our.history);

        println!("OUR board -> {:#?}", our.inner);
        println!("our fen -> {}", our.inner.to_fen());

        let our_set: HashSet<Move> = HashSet::from_iter(our_moves.iter().copied());
        let their_set = HashSet::from_iter(their_moves.iter().map(|m| translate_move(*m)));

        let differences: Vec<&Move> = our_set.symmetric_difference(&their_set).collect();

        println!("We generated -> {}", our_moves.len());
        println!("They generated -> {}", their_moves.len());
        println!("diff -> {differences:#?}");

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
        } => {
            if role == shakmaty::Role::Pawn
                && ((from.rank() == shakmaty::Rank::Second && to.rank() == shakmaty::Rank::Fourth)
                    || (from.rank() == shakmaty::Rank::Seventh
                        && to.rank() == shakmaty::Rank::Fifth))
            {
                return Move::double_push(
                    Square::from_u32_checked(from.to_u32()),
                    Square::from_u32_checked(to.to_u32()),
                );
            }

            if capture.is_none() && promotion.is_none() {
                return Move::quiet(
                    Square::from_u32_checked(from.to_u32()),
                    Square::from_u32_checked(to.to_u32()),
                );
            }

            if capture.is_some() && promotion.is_none() {
                return Move::capture(
                    Square::from_u32_checked(from.to_u32()),
                    Square::from_u32_checked(to.to_u32()),
                );
            }

            if capture.is_none() && promotion.is_some() {
                return Move::promotion(
                    Square::from_u32_checked(from.to_u32()),
                    Square::from_u32_checked(to.to_u32()),
                    match promotion.unwrap() {
                        shakmaty::Role::Pawn => Role::Pawn,
                        shakmaty::Role::Knight => Role::Knight,
                        shakmaty::Role::Bishop => Role::Bishop,
                        shakmaty::Role::Rook => Role::Rook,
                        shakmaty::Role::Queen => Role::Queen,
                        _ => unreachable!(),
                    },
                    false,
                );
            }

            if capture.is_some() && promotion.is_some() {
                return Move::promotion(
                    Square::from_u32_checked(from.to_u32()),
                    Square::from_u32_checked(to.to_u32()),
                    match promotion.unwrap() {
                        shakmaty::Role::Pawn => Role::Pawn,
                        shakmaty::Role::Knight => Role::Knight,
                        shakmaty::Role::Bishop => Role::Bishop,
                        shakmaty::Role::Rook => Role::Rook,
                        shakmaty::Role::Queen => Role::Queen,
                        _ => unreachable!(),
                    },
                    true,
                );
            }

            panic!()
        }
        TheirMove::EnPassant { from, to } => Move::en_passant(
            Square::from_u32_checked(from.to_u32()),
            Square::from_u32_checked(to.to_u32()),
        ),
        TheirMove::Castle { king, rook } => match rook {
            shakmaty::Square::A1 => Move::queen_castle(Square::E1, Square::C1),
            shakmaty::Square::H1 => Move::king_castle(Square::E1, Square::G1),
            shakmaty::Square::A8 => Move::queen_castle(Square::E8, Square::C8),
            shakmaty::Square::H8 => Move::king_castle(Square::E8, Square::G8),

            _ => unreachable!(),
        },

        TheirMove::Put { .. } => panic!("IMPOSSIBLE"),
    }
}
