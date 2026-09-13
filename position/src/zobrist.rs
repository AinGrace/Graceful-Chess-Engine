use std::hint::unreachable_unchecked;

use types::{
    castlings::Castlings,
    chess_move::{Move, MoveFlag},
    color::Color,
    piece::Piece,
    role::Role,
    square::Square,
};

use crate::position::Position;

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

pub fn compute_hash(pos: &Position) -> u64 {
    let mut hash = 0;

    pos.board().occupied().for_each(|sqr| {
        // Occupied guarantees the existence of piece
        let piece = unsafe { pos.board().peek_unchecked(sqr) };
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
    moving_piece: Piece,
    captured_role: Option<Role>,
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

    let from = mv.from();
    let to = mv.to();

    match mv.flag() {
        MoveFlag::KingCastle | MoveFlag::QueenCastle => {
            let (rook_from, rook_to) = match to {
                Square::C1 => (Square::A1, Square::D1),
                Square::C8 => (Square::A8, Square::D8),
                Square::G1 => (Square::H1, Square::F1),
                Square::G8 => (Square::H8, Square::F8),

                _ => unsafe { unreachable_unchecked() },
            };
            *hash ^= ZOBRIST.piece(moving_piece, from);
            *hash ^= ZOBRIST.piece(moving_piece, to);
            *hash ^= ZOBRIST.piece(moving_piece, rook_from);
            *hash ^= ZOBRIST.piece(moving_piece, rook_to);
        }

        MoveFlag::EnPassant => {
            let captured_sqr = Square::of(to.file(), from.rank());

            *hash ^= ZOBRIST.piece(moving_piece, from);
            *hash ^= ZOBRIST.piece(Piece::of(Role::Pawn, !us), captured_sqr);
            *hash ^= ZOBRIST.piece(moving_piece, to);
        }

        rest => {
            let dest_piece = match rest {
                MoveFlag::PromoN | MoveFlag::PromoCapN => Piece::of(Role::Knight, us),
                MoveFlag::PromoB | MoveFlag::PromoCapB => Piece::of(Role::Bishop, us),
                MoveFlag::PromoR | MoveFlag::PromoCapR => Piece::of(Role::Rook, us),
                MoveFlag::PromoQ | MoveFlag::PromoCapQ => Piece::of(Role::Queen, us),

                _no_prom => moving_piece,
            };

            *hash ^= ZOBRIST.piece(moving_piece, from);

            if mv.is_capture() {
                *hash ^= ZOBRIST.piece(
                    Piece::of(unsafe { captured_role.unwrap_unchecked() }, !us),
                    to,
                );
            }

            *hash ^= ZOBRIST.piece(dest_piece, to);
        }
    }

    if let Some(ep) = new_ep {
        *hash ^= ZOBRIST.ep[ep.file().to_usize()];
    }

    *hash ^= ZOBRIST.castlings[new_castling.as_usize()];
    *hash ^= ZOBRIST.black_to_move;
}
