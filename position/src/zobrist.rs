use types::{
    castlings::Castlings, chess_move::Move, color::Color, piece::Piece, role::Role, square::Square,
};

use crate::chessboard::ChessBoard;

static ZOBRIST: ZobristTable = init_zobrist_table();

struct ZobristTable {
    pieces: [u64; 768],
    castlings: [u64; 16],
    ep: [u64; 8],
    black_to_move: u64,
}

impl ZobristTable {
    fn piece(&self, piece: Piece, square: Square) -> u64 {
        ZOBRIST.pieces[piece.as_usize() * 64 + square.as_usize()]
    }
}

#[rustfmt::skip]
const fn init_zobrist_table() -> ZobristTable {
    let side_seed = 0xF0F0_F0F0_F0F0_F0F0;
    let mut seed  = 0xCAFE_BABE_DEAD_BEEF;

    let pieces    = init_table::<768>(&mut seed);
    let castlings = init_table::<16>(&mut seed);
    let ep        = init_table::<8>(&mut seed);

    ZobristTable {
        pieces,
        castlings,
        ep,
        black_to_move: side_seed,
    }
}

const fn init_table<const SIZE: usize>(seed: &mut u64) -> [u64; SIZE] {
    let mut table = [0; SIZE];

    let mut idx = 0;
    while idx < SIZE {
        table[idx] = xorshift64(seed);
        idx += 1;
    }

    table
}

/// simple and deterministic pRNG
const fn xorshift64(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;

    *state
}

pub fn compute_hash(pos: &ChessBoard) -> u64 {
    let mut hash = 0;

    pos.board().occupied().for_each(|sqr| {
        let piece = pos.board().peek_checked(sqr);
        hash ^= ZOBRIST.piece(piece, sqr);
    });

    hash ^= ZOBRIST.castlings[pos.castling_rights().as_usize()];

    if let Some(ep) = pos.ep_square() {
        hash ^= ZOBRIST.ep[ep.file().to_usize()]
    }

    if pos.turn() == Color::Black {
        hash ^= ZOBRIST.black_to_move;
    }

    hash
}

pub fn update_hash(
    hash: &mut u64,
    mv: Move,
    us: Color,
    old_ep: Option<Square>,
    old_castling: Castlings,
    new_ep: Option<Square>,
    new_castling: Castlings,
) {
    if let Some(ep) = old_ep {
        *hash ^= ZOBRIST.ep[ep.file().to_usize()];
    }
    *hash ^= ZOBRIST.castlings[old_castling.as_usize()];

    match mv {
        Move::Standard {
            role,
            from,
            to,
            capture,
            promotion,
        } => {
            let moving_piece = Piece::of(role, us);
            let dest_piece = match promotion {
                Some(promo) => Piece::of(promo, us),
                None => moving_piece,
            };

            *hash ^= ZOBRIST.piece(moving_piece, from);

            if let Some(captured) = capture {
                *hash ^= ZOBRIST.piece(Piece::of(captured, !us), to);
            }

            *hash ^= ZOBRIST.piece(dest_piece, to);
        }

        Move::EnPassant { from, to } => {
            let our_pawn = Piece::of(Role::Pawn, us);
            let captured_sqr = Square::of(to.file(), from.rank());

            *hash ^= ZOBRIST.piece(our_pawn, from);
            *hash ^= ZOBRIST.piece(Piece::of(Role::Pawn, !us), captured_sqr);
            *hash ^= ZOBRIST.piece(our_pawn, to);
        }

        Move::Castling { king, rook } => {
            let (king_dest, rook_dest) = match rook {
                Square::H1 => (Square::G1, Square::F1),
                Square::A1 => (Square::C1, Square::D1),
                Square::H8 => (Square::G8, Square::F8),
                Square::A8 => (Square::C8, Square::D8),
                _illegal => unreachable!("Invalid rook square for castling in zobrist update hash"),
            };

            *hash ^= ZOBRIST.piece(Piece::of(Role::King, us), king);
            *hash ^= ZOBRIST.piece(Piece::of(Role::Rook, us), rook);
            *hash ^= ZOBRIST.piece(Piece::of(Role::King, us), king_dest);
            *hash ^= ZOBRIST.piece(Piece::of(Role::Rook, us), rook_dest);
        }
    }

    if let Some(ep) = new_ep {
        *hash ^= ZOBRIST.ep[ep.file().to_usize()];
    }

    *hash ^= ZOBRIST.castlings[new_castling.as_usize()];
    *hash ^= ZOBRIST.black_to_move;
}
