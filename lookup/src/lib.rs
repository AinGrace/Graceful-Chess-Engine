#![expect(
    long_running_const_eval,
    reason = "Lookup tables take some time to generate during compile time"
)]

use std::{arch::x86_64::_pext_u64, hint::unreachable_unchecked};

use types::{
    bitboard::{
        Bitboard,
        masks::{NOT_FILE_A, NOT_FILE_AB, NOT_FILE_GH, NOT_FILE_H, OUTER_LAYER, RANK_1, RANK_8},
    },
    color::Color,
    direction::Direction,
    piece::Piece,
    role::Role,
    square::Square,
};

// ------------------------------------------- //

static NO_RAYS: [u64; 64] = init_rays(Direction::North);
static NW_RAYS: [u64; 64] = init_rays(Direction::NorthWest);
static NE_RAYS: [u64; 64] = init_rays(Direction::NorthEast);

static WE_RAYS: [u64; 64] = init_rays(Direction::West);
static EA_RAYS: [u64; 64] = init_rays(Direction::East);

static SO_RAYS: [u64; 64] = init_rays(Direction::South);
static SW_RAYS: [u64; 64] = init_rays(Direction::SouthWest);
static SE_RAYS: [u64; 64] = init_rays(Direction::SouthEast);

// ------------------------------------------- //

static W_PAWN_ATTACKS: [u64; 64] = init_pawn_attacks(Color::White);
static B_PAWN_ATTACKS: [u64; 64] = init_pawn_attacks(Color::Black);

static W_PAWN_PUSHES: [u64; 64] = init_pawn_pushes(Color::White);
static B_PAWN_PUSHES: [u64; 64] = init_pawn_pushes(Color::Black);

static W_PAWN_DOUBLE_PUSHES: [u64; 64] = init_pawn_double_pushes(Color::White);
static B_PAWN_DOUBLE_PUSHES: [u64; 64] = init_pawn_double_pushes(Color::Black);

static KNIGHT_ATTACKS: [u64; 64] = init_knight_attacks();

static KING_ATTACKS: [u64; 64] = init_king_attacks();

static BETWEEN: [[u64; 64]; 64] = init_between_table();

// ------------------------------------------- //

const ROOK_TABLE_SIZE: usize = 102400;

static ROOK_MASKS: [u64; 64] = init_rook_masks();
static ROOK_DATA: ([u64; ROOK_TABLE_SIZE], [usize; 64]) = init_rook_attack_table_flat();
static ROOK_ATTACKS: [u64; ROOK_TABLE_SIZE] = ROOK_DATA.0;
static ROOK_OFFSETS: [usize; 64] = ROOK_DATA.1;

// ------------------------------------------- //

const BISHOP_TABLE_SIZE: usize = 5248;

static BISHOP_MASKS: [u64; 64] = init_bishop_masks();
static BISHOP_DATA: ([u64; BISHOP_TABLE_SIZE], [usize; 64]) = init_bishop_attack_table_flat();
static BISHOP_ATTACKS: [u64; BISHOP_TABLE_SIZE] = BISHOP_DATA.0;
static BISHOP_OFFSETS: [usize; 64] = BISHOP_DATA.1;

pub const KING_PENALTY_FACTOR: i16 = 70;

pub const MAX_PHASE: u8 = 24;

///Piece-Square Tables (PSTs) are a simple evaluation technique that assigns a score to a piece depending on which square it occupies.
///The idea is:
///A knight in the center is usually stronger than a knight on the edge.
///A pawn advanced to the 6th rank is often more valuable than one on the 2nd rank.
///
///These values are part of the evaluation score
#[rustfmt::skip]
pub static MG_PAWN_PST: [i16; 64] = [
    0,   0,   0,   0,   0,   0,  0,   0,
   98, 134,  61,  95,  68, 126, 34, -11,
   -6,   7,  26,  31,  65,  56, 25, -20,
  -14,  13,   6,  21,  23,  12, 17, -23,
  -27,  -2,  -5,  12,  17,   6, 10, -25,
  -26,  -4,  -4, -10,   3,   3, 33, -12,
  -35,  -1, -20, -23, -15,  24, 38, -22,
    0,   0,   0,   0,   0,   0,  0,   0,
];

#[rustfmt::skip]
pub static EG_PAWN_PST: [i16; 64] = [
    0,   0,   0,   0,   0,   0,   0,   0,
  178, 173, 158, 134, 147, 132, 165, 187,
   94, 100,  85,  67,  56,  53,  82,  84,
   32,  24,  13,   5,  -2,   4,  17,  17,
   13,   9,  -3,  -7,  -7,  -8,   3,  -1,
    4,   7,  -6,   1,   0,  -5,  -1,  -8,
   13,   8,   8,  10,  13,   0,   2,  -7,
    0,   0,   0,   0,   0,   0,   0,   0,
];

#[rustfmt::skip]
pub static MG_KNIGHT_PST: [i16; 64] = [
    -167, -89, -34, -49,  61, -97, -15, -107,
     -73, -41,  72,  36,  23,  62,   7,  -17,
     -47,  60,  37,  65,  84, 129,  73,   44,
      -9,  17,  19,  53,  37,  69,  18,   22,
     -13,   4,  16,  13,  28,  19,  21,   -8,
     -23,  -9,  12,  10,  19,  17,  25,  -16,
     -29, -53, -12,  -3,  -1,  18, -14,  -19,
    -105, -21, -58, -33, -17, -28, -19,  -23,
];

#[rustfmt::skip]
pub static EG_KNIGHT_PST: [i16; 64] = [
    -58, -38, -13, -28, -31, -27, -63, -99,
    -25,  -8, -25,  -2,  -9, -25, -24, -52,
    -24, -20,  10,   9,  -1,  -9, -19, -41,
    -17,   3,  22,  22,  22,  11,   8, -18,
    -18,  -6,  16,  25,  16,  17,   4, -18,
    -23,  -3,  -1,  15,  10,  -3, -20, -22,
    -42, -20, -10,  -5,  -2, -20, -23, -44,
    -29, -51, -23, -15, -22, -18, -50, -64,
];

#[rustfmt::skip]
pub static MG_BISHOP_PST: [i16; 64] = [
    -29,   4, -82, -37, -25, -42,   7,  -8,
    -26,  16, -18, -13,  30,  59,  18, -47,
    -16,  37,  43,  40,  35,  50,  37,  -2,
     -4,   5,  19,  50,  37,  37,   7,  -2,
     -6,  13,  13,  26,  34,  12,  10,   4,
      0,  15,  15,  15,  14,  27,  18,  10,
      4,  15,  16,   0,   7,  21,  33,   1,
    -33,  -3, -14, -21, -13, -12, -39, -21,
];

#[rustfmt::skip]
pub static EG_BISHOP_PST: [i16; 64] = [
    -14, -21, -11,  -8, -7,  -9, -17, -24,
     -8,  -4,   7, -12, -3, -13,  -4, -14,
      2,  -8,   0,  -1, -2,   6,   0,   4,
     -3,   9,  12,   9, 14,  10,   3,   2,
     -6,   3,  13,  19,  7,  10,  -3,  -9,
    -12,  -3,   8,  10, 13,   3,  -7, -15,
    -14, -18,  -7,  -1,  4,  -9, -15, -27,
    -23,  -9, -23,  -5, -9, -16,  -5, -17,
];

#[rustfmt::skip]
pub static MG_ROOK_PST: [i16; 64] = [
    32,  42,  32,  51, 63,  9,  31,  43,
    27,  32,  58,  62, 80, 67,  26,  44,
    -5,  19,  26,  36, 17, 45,  61,  16,
   -24, -11,   7,  26, 24, 35,  -8, -20,
   -36, -26, -12,  -1,  9, -7,   6, -23,
   -45, -25, -16, -17,  3,  0,  -5, -33,
   -44, -16, -20,  -9, -1, 11,  -6, -71,
   -19, -13,   1,  17, 16,  7, -37, -26,
];

#[rustfmt::skip]
pub static EG_ROOK_PST: [i16; 64] = [
    13, 10, 18, 15, 12,  12,   8,   5,
    11, 13, 13, 11, -3,   3,   8,   3,
     7,  7,  7,  5,  4,  -3,  -5,  -3,
     4,  3, 13,  1,  2,   1,  -1,   2,
     3,  5,  8,  4, -5,  -6,  -8, -11,
    -4,  0, -5, -1, -7, -12,  -8, -16,
    -6, -6,  0,  2, -9,  -9, -11,  -3,
    -9,  2,  3, -1, -5, -13,   4, -20,
];

#[rustfmt::skip]
pub static MG_QUEEN_PST: [i16; 64] = [
    -28,   0,  29,  12,  59,  44,  43,  45,
    -24, -39,  -5,   1, -16,  57,  28,  54,
    -13, -17,   7,   8,  29,  56,  47,  57,
    -27, -27, -16, -16,  -1,  17,  -2,   1,
     -9, -26,  -9, -10,  -2,  -4,   3,  -3,
    -14,   2, -11,  -2,  -5,   2,  14,   5,
    -35,  -8,  11,   2,   8,  15,  -3,   1,
     -1, -18,  -9,  10, -15, -25, -31, -50,
];

#[rustfmt::skip]
pub static EG_QUEEN_PST: [i16; 64] = [
    -9,  22,  22,  27,  27,  19,  10,  20,
   -17,  20,  32,  41,  58,  25,  30,   0,
   -20,   6,   9,  49,  47,  35,  19,   9,
     3,  22,  24,  45,  57,  40,  57,  36,
   -18,  28,  19,  47,  31,  34,  39,  23,
   -16, -27,  15,   6,   9,  17,  10,   5,
   -22, -23, -30, -16, -16, -23, -36, -32,
   -33, -28, -22, -43,  -5, -32, -20, -41,
];

#[rustfmt::skip]
pub static MG_KING_PST: [i16; 64] = [
    -65,  23,  16, -15, -56, -34,   2,  13,
     29,  -1, -20,  -7,  -8,  -4, -38, -29,
     -9,  24,   2, -16, -20,   6,  22, -22,
    -17, -20, -12, -27, -30, -25, -14, -36,
    -49,  -1, -27, -39, -46, -44, -33, -51,
    -14, -14, -22, -46, -44, -30, -15, -27,
      1,   7,  -8, -64, -43, -16,   9,   8,
    -15,  36,  12, -54,   8, -28,  24,  14,
];

#[rustfmt::skip]
pub static EG_KING_PST: [i16; 64] = [
    -74, -35, -18, -18, -11,  15,   4, -17,
    -12,  17,  14,  17,  17,  38,  23,  11,
     10,  17,  23,  15,  20,  45,  44,  13,
     -8,  22,  24,  27,  26,  33,  26,   3,
    -18,  -4,  21,  24,  27,  23,   9, -11,
    -19,  -3,  11,  21,  23,  16,   7,  -9,
    -27, -11,   4,  13,  14,   4,  -5, -17,
    -53, -34, -21, -11, -28, -14, -24, -43
];

pub static MATERIAL_MG: [i16; 6] = [82, 337, 365, 477, 1025, 0];
pub static MATERIAL_EG: [i16; 6] = [94, 281, 297, 512, 936, 0];

pub static PSQT: Psqt = Psqt::init();

#[derive(Debug)]
pub struct Psqt {
    mg: [[i16; 64]; 12],
    eg: [[i16; 64]; 12],
}

impl Psqt {
    const fn init() -> Self {
        let mut mg = [[0i16; 64]; 12];
        let mut eg = [[0i16; 64]; 12];

        let mut role = 0;
        while role < 12 {
            let mut sq = 0;
            while sq < 64 {
                let table_idx = if role > 5 { sq } else { sq ^ 56 };

                let (m_mg, m_eg) = piece_pst(role, table_idx);

                if role > 5 {
                    mg[role][sq] = material_lookup(role, MATERIAL_MG) + m_mg;
                    eg[role][sq] = material_lookup(role, MATERIAL_EG) + m_eg;
                } else {
                    mg[role][sq] = material_lookup(role, MATERIAL_MG) + m_mg;
                    eg[role][sq] = material_lookup(role, MATERIAL_EG) + m_eg;
                }

                sq += 1;
            }
            role += 1;
        }
        Psqt { mg, eg }
    }

    pub fn mg(&self, piece: Piece, sqr: usize) -> i16 {
        let piece = piece.as_usize();
        self.mg[piece][sqr]
    }

    pub fn eg(&self, piece: Piece, sqr: usize) -> i16 {
        let piece = piece.as_usize();
        self.eg[piece][sqr]
    }
}

// ------------------------------------------- //

/// assign 100 as default pawn value instead of 1 in order to avoid floating point calculations
pub const PAWN_VALUE: i16 = 100;
pub const KNIGHT_VALUE: i16 = 340;
pub const BISHOP_VALUE: i16 = 350;
pub const ROOK_VALUE: i16 = 500;
pub const QUEEN_VALUE: i16 = 900;

pub fn piece_val(piece: Piece) -> i16 {
    match piece.role() {
        Role::Pawn => PAWN_VALUE,
        Role::Knight => KNIGHT_VALUE,
        Role::Bishop => BISHOP_VALUE,
        Role::Rook => ROOK_VALUE,
        Role::Queen => QUEEN_VALUE,
        Role::King => i16::MAX,
    }
}

const fn material_lookup(role: usize, table: [i16; 6]) -> i16 {
    return match role {
        0 | 6 => table[0],
        1 | 7 => table[1],
        2 | 8 => table[2],
        3 | 9 => table[3],
        4 | 10 => table[4],
        5 | 11 => table[5],
        _ => unsafe { unreachable_unchecked() },
    };
}

const fn piece_pst(role: usize, sqr: usize) -> (i16, i16) {
    return match role {
        0 | 6 => (MG_PAWN_PST[sqr], EG_PAWN_PST[sqr]),
        1 | 7 => (MG_KNIGHT_PST[sqr], EG_KNIGHT_PST[sqr]),
        2 | 8 => (MG_BISHOP_PST[sqr], EG_BISHOP_PST[sqr]),
        3 | 9 => (MG_ROOK_PST[sqr], EG_ROOK_PST[sqr]),
        4 | 10 => (MG_QUEEN_PST[sqr], EG_QUEEN_PST[sqr]),
        5 | 11 => (MG_KING_PST[sqr], EG_KING_PST[sqr]),
        _ => unsafe { unreachable_unchecked() },
    };
}

#[inline(always)]
pub fn phase_weight(role: Role) -> u8 {
    match role {
        Role::Pawn => 0,
        Role::Knight => 1,
        Role::Bishop => 1,
        Role::Rook => 2,
        Role::Queen => 4,
        Role::King => 0,
    }
}

#[inline(always)]
pub fn pawn_attacks(color: Color, sqr: Square) -> u64 {
    // SAFETY: usize returned from Square::to_usize is always between 0 and 63
    unsafe {
        match color {
            Color::White => *W_PAWN_ATTACKS.get_unchecked(sqr.as_usize()),
            Color::Black => *B_PAWN_ATTACKS.get_unchecked(sqr.as_usize()),
        }
    }
}

#[inline(always)]
pub fn pawn_pushes(color: Color, sqr: Square) -> u64 {
    // SAFETY: usize returned from Square::to_usize is always between 0 and 63
    unsafe {
        match color {
            Color::White => *W_PAWN_PUSHES.get_unchecked(sqr.as_usize()),
            Color::Black => *B_PAWN_PUSHES.get_unchecked(sqr.as_usize()),
        }
    }
}

#[inline(always)]
pub fn pawn_double_pushes(color: Color, sqr: Square) -> u64 {
    // SAFETY: usize returned from Square::to_usize is always between 0 and 63
    unsafe {
        match color {
            Color::White => *W_PAWN_DOUBLE_PUSHES.get_unchecked(sqr.as_usize()),
            Color::Black => *B_PAWN_DOUBLE_PUSHES.get_unchecked(sqr.as_usize()),
        }
    }
}

#[inline(always)]
pub fn knight_attacks(sqr: Square) -> u64 {
    // SAFETY: usize returned from Square::to_usize is always between 0 and 63
    unsafe { *KNIGHT_ATTACKS.get_unchecked(sqr.as_usize()) }
}

#[inline(always)]
pub fn bishop_attacks(sq: Square, occupied: u64) -> u64 {
    // SAFETY: usize returned from Square::to_usize is always between 0 and 63
    // crate is quaranteed to be compiled and run against targets with bmi2
    unsafe {
        let index = pext(occupied, BISHOP_MASKS[sq.as_usize()]) as usize;
        *BISHOP_ATTACKS.get_unchecked(BISHOP_OFFSETS[sq.as_usize()] + index)
    }
}

#[inline(always)]
pub fn rook_attacks(sq: Square, occupied: u64) -> u64 {
    // SAFETY: usize returned from Square::to_usize is always between 0 and 63
    // crate is quaranteed to be compiled and run against targets with bmi2
    unsafe {
        let index = pext(occupied, ROOK_MASKS[sq.as_usize()]) as usize;
        *ROOK_ATTACKS.get_unchecked(ROOK_OFFSETS[sq.as_usize()] + index)
    }
}

#[inline(always)]
pub fn queen_attacks(sqr: Square, occupied: u64) -> u64 {
    rook_attacks(sqr, occupied) | bishop_attacks(sqr, occupied)
}

#[inline(always)]
pub fn king_attacks(sqr: Square) -> u64 {
    // SAFETY: usize returned from Square::to_usize is always between 0 and 63
    unsafe { *KING_ATTACKS.get_unchecked(sqr.as_usize()) }
}

#[inline(always)]
pub fn diagonal_rays_from(sqr: Square) -> u64 {
    let index = sqr.as_usize();

    // SAFETY: usize returned from Square::to_usize is always between 0 and 63
    unsafe {
        NW_RAYS.get_unchecked(index)
            | NE_RAYS.get_unchecked(index)
            | SW_RAYS.get_unchecked(index)
            | SE_RAYS.get_unchecked(index)
    }
}

#[inline(always)]
pub fn orthogonal_rays_from(sqr: Square) -> u64 {
    let index = sqr.as_usize();

    // SAFETY: usize returned from Square::to_usize is always between 0 and 63
    unsafe {
        NO_RAYS.get_unchecked(index)
            | SO_RAYS.get_unchecked(index)
            | WE_RAYS.get_unchecked(index)
            | EA_RAYS.get_unchecked(index)
    }
}

#[inline(always)]
pub fn ray_between(from: Square, to: Square) -> u64 {
    // SAFETY: usize returned from Square::to_usize is always between 0 and 63
    unsafe {
        *BETWEEN
            .get_unchecked(from.as_usize())
            .get_unchecked(to.as_usize())
    }
}

#[inline(always)]
pub fn ray_between_inclusive(from: Square, to: Square) -> u64 {
    ray_between(from, to) | from.as_mask() | to.as_mask()
}

const fn init_pawn_attacks(color: Color) -> [u64; 64] {
    let mut table = [0; 64];

    let mut idx = 0;
    while idx < 64 {
        let sqr = 1 << idx;
        table[idx] = match color {
            Color::White => (sqr << 7 & NOT_FILE_H) | (sqr << 9 & NOT_FILE_A),
            Color::Black => (sqr >> 9 & NOT_FILE_H) | (sqr >> 7 & NOT_FILE_A),
        };
        idx += 1;
    }

    table
}

const fn init_pawn_pushes(color: Color) -> [u64; 64] {
    let mut table = [0; 64];

    let mut idx = 0;
    while idx < 64 {
        let sqr = 1 << idx;

        table[idx] = if idx < 8 || idx > 56 {
            0
        } else {
            match color {
                Color::White => sqr << 8,
                Color::Black => sqr >> 8,
            }
        };
        idx += 1;
    }

    table
}

const fn init_pawn_double_pushes(color: Color) -> [u64; 64] {
    let mut table = [0; 64];

    let mut idx = 0;
    while idx < 64 {
        let sqr = 1 << idx;

        table[idx] = match color {
            Color::White => {
                if idx > 7 && idx < 16 {
                    sqr << 16
                } else {
                    0
                }
            }
            Color::Black => {
                if idx > 47 && idx < 56 {
                    sqr >> 16
                } else {
                    0
                }
            }
        };

        idx += 1;
    }

    table
}

#[rustfmt::skip]
const fn init_knight_attacks() -> [u64; 64] {
    let mut table = [0; 64];

    let mut idx = 0;
    while idx < 64 {
        let sqr = 1 << idx;

        table[idx] = sqr << 15 & NOT_FILE_H
            | sqr << 17 & NOT_FILE_A
            | sqr << 6  & NOT_FILE_GH
            | sqr << 10 & NOT_FILE_AB
            | sqr >> 17 & NOT_FILE_H
            | sqr >> 15 & NOT_FILE_A
            | sqr >> 10 & NOT_FILE_GH
            | sqr >> 6  & NOT_FILE_AB;

        idx += 1;
    }

    table
}

const fn init_bishop_masks() -> [u64; 64] {
    let mut table = [0; 64];

    let mut idx = 0;
    while idx < 64 {
        let north_we = NW_RAYS[idx];
        let north_ea = NE_RAYS[idx];
        let south_we = SW_RAYS[idx];
        let south_ea = SE_RAYS[idx];

        table[idx] = (north_we | north_ea | south_we | south_ea) & !OUTER_LAYER.as_u64();

        idx += 1;
    }

    table
}

const fn init_rook_masks() -> [u64; 64] {
    let mut table = [0; 64];

    let mut idx = 0;
    while idx < 64 {
        let north = NO_RAYS[idx] & !RANK_8.as_u64();
        let south = SO_RAYS[idx] & !RANK_1.as_u64();
        let west = WE_RAYS[idx] & NOT_FILE_A;
        let east = EA_RAYS[idx] & NOT_FILE_H;

        table[idx] = north | south | west | east;

        idx += 1;
    }

    table
}

const fn init_bishop_attack_table_flat() -> ([u64; BISHOP_TABLE_SIZE], [usize; 64]) {
    let mut table = [0; BISHOP_TABLE_SIZE];
    let mut offsets = [0usize; 64];

    let mut offset = 0usize;
    let mut sq = 0usize;

    while sq < 64 {
        offsets[sq] = offset;

        let mask = BISHOP_MASKS[sq];
        let bitcount = mask.count_ones() as usize;
        let subset_count = 1usize << bitcount;

        let mut subset = 0usize;
        while subset < subset_count {
            let raw_occupied = deposit_bits(subset as u64, mask);
            let occupied = Bitboard::from_u64(raw_occupied);

            let attacks = bishop_rays(Square::from_u32_checked(sq as u32), occupied);

            let index = pext_const(raw_occupied, mask) as usize;

            table[offset + index] = attacks;

            subset += 1;
        }

        offset += subset_count;
        sq += 1;
    }

    (table, offsets)
}

const fn init_rook_attack_table_flat() -> ([u64; ROOK_TABLE_SIZE], [usize; 64]) {
    let mut table = [0; ROOK_TABLE_SIZE];
    let mut offsets = [0usize; 64];

    let mut offset = 0usize;
    let mut sq = 0usize;

    while sq < 64 {
        offsets[sq] = offset;

        let mask = ROOK_MASKS[sq];
        let bitcount = mask.count_ones() as usize;
        let subset_count = 1usize << bitcount;

        let mut subset = 0usize;
        while subset < subset_count {
            let raw_occupied = deposit_bits(subset as u64, mask);
            let occupied = Bitboard::from_u64(raw_occupied);

            let attacks = rook_rays(Square::from_u32_checked(sq as u32), occupied);

            let index = pext_const(raw_occupied, mask) as usize;

            table[offset + index] = attacks;

            subset += 1;
        }

        offset += subset_count;
        sq += 1;
    }

    (table, offsets)
}

const fn init_king_attacks() -> [u64; 64] {
    let mut table = [0; 64];

    let mut idx = 0;
    while idx < 64 {
        let sqr = 1 << idx;

        table[idx] = sqr << 8
            | sqr << 7 & NOT_FILE_H
            | sqr << 9 & NOT_FILE_A
            | sqr >> 1 & NOT_FILE_H
            | sqr << 1 & NOT_FILE_A
            | sqr >> 8
            | sqr >> 9 & NOT_FILE_H
            | sqr >> 7 & NOT_FILE_A;

        idx += 1;
    }

    table
}

const fn init_rays(dir: Direction) -> [u64; 64] {
    let mut table = [0; 64];

    let (mask, vert_dir, horiz_dir, vert_inv, horiz_inv) = match dir {
        Direction::NorthWest => (
            0x0102_0408_1020_4000,
            Direction::North,
            Direction::West,
            false,
            true,
        ),
        Direction::NorthEast => (
            0x8040_2010_0804_0200,
            Direction::North,
            Direction::East,
            false,
            false,
        ),
        Direction::SouthWest => (
            0x0040_2010_0804_0201,
            Direction::South,
            Direction::West,
            true,
            true,
        ),
        Direction::SouthEast => (
            0x0002_0408_1020_4080,
            Direction::South,
            Direction::East,
            true,
            false,
        ),
        Direction::North => (
            0x0101_0101_0101_0100,
            Direction::North,
            Direction::East,
            false,
            false,
        ),
        Direction::South => (
            0x0080_8080_8080_8080,
            Direction::South,
            Direction::West,
            true,
            true,
        ),
        Direction::East => (
            0x0000_0000_0000_00fe,
            Direction::North,
            Direction::East,
            false,
            false,
        ),
        Direction::West => (
            0x7f00_0000_0000_0000,
            Direction::South,
            Direction::West,
            true,
            true,
        ),
    };

    let base_mask = Bitboard::from_u64(mask);

    let mut idx = 0;
    while idx < 64 {
        let rank = idx >> 3;
        let file = idx & 7;

        let r = if vert_inv { 7 - rank } else { rank };
        let f = if horiz_inv { 7 - file } else { file };

        let base = base_mask.shift_dir_repeat(vert_dir, r);
        table[idx as usize] = base.shift_dir_repeat(horiz_dir, f).as_u64();

        idx += 1;
    }
    table
}

const fn init_between_table() -> [[u64; 64]; 64] {
    let mut table = [[0; 64]; 64];

    let mut from = 0;
    while from < 64 {
        let mut to = 0;
        while to < 64 {
            let ray = compute_between(
                Square::from_u32_checked(from as u32),
                Square::from_u32_checked(to as u32),
            );

            table[from][to] = ray;

            to += 1;
        }

        from += 1;
    }

    table
}

const fn bishop_rays(attacks_from: Square, occupied: Bitboard) -> u64 {
    ray_attacks_dir(&NW_RAYS, attacks_from, occupied, Direction::NorthWest)
        | ray_attacks_dir(&NE_RAYS, attacks_from, occupied, Direction::NorthEast)
        | ray_attacks_dir(&SW_RAYS, attacks_from, occupied, Direction::SouthWest)
        | ray_attacks_dir(&SE_RAYS, attacks_from, occupied, Direction::SouthEast)
}

const fn rook_rays(sqr: Square, occupied: Bitboard) -> u64 {
    ray_attacks_dir(&NO_RAYS, sqr, occupied, Direction::North)
        | ray_attacks_dir(&WE_RAYS, sqr, occupied, Direction::West)
        | ray_attacks_dir(&EA_RAYS, sqr, occupied, Direction::East)
        | ray_attacks_dir(&SO_RAYS, sqr, occupied, Direction::South)
}

const fn compute_between(from: Square, to: Square) -> u64 {
    if u32::abs_diff(from.as_u32(), to.as_u32()) == 0 {
        return 0;
    }

    let from_rank = from.rank().to_u32() as i32;
    let from_file = from.file().to_u32() as i32;

    let to_rank = to.rank().to_u32() as i32;
    let to_file = to.file().to_u32() as i32;

    let dr = (to_rank - from_rank).signum();
    let df = (to_file - from_file).signum();

    let is_diagonal = (from_rank - to_rank).abs() == (from_file - to_file).abs();
    let is_orthogonal = (from_rank - to_rank == 0 && from_file - to_file != 0)
        | (from_file - to_file == 0 && from_rank - to_rank != 0);

    if !is_diagonal && !is_orthogonal {
        return 0;
    }

    let mut r = from_rank + dr;
    let mut f = from_file + df;

    let mut result = 0;

    while r != to_rank || f != to_file {
        let sq = (r * 8 + f) as u32;
        result |= 1u64 << sq;
        r += dr;
        f += df;
    }

    result
}

const fn ray_attacks_dir(
    rays: &[u64; 64],
    attacks_from: Square,
    occupied: Bitboard,
    direction: Direction,
) -> u64 {
    let attacks = rays[attacks_from.as_usize()];
    let blockers = attacks & occupied.as_u64();

    let first_blocker = if direction.is_anti() {
        let zeros = blockers.trailing_zeros();
        if zeros == 64 { 0 } else { zeros }
    } else {
        let zeros = blockers.leading_zeros();
        if zeros == 64 { 0 } else { 63 - zeros }
    };

    if first_blocker != 0 {
        attacks ^ rays[first_blocker as usize]
    } else {
        attacks
    }
}

/// inverse of pext
const fn deposit_bits(mut subset: u64, mut mask: u64) -> u64 {
    let mut res = 0;

    while mask != 0 {
        let lsb = mask & mask.wrapping_neg();
        mask ^= lsb;

        if subset & 1 != 0 {
            res |= lsb;
        }

        subset >>= 1;
    }

    res
}

#[target_feature(enable = "bmi2")]
unsafe fn pext(value: u64, mask: u64) -> u64 {
    _pext_u64(value, mask)
}

/// a software level emulation of PEXT bmi2 instruction that can be used in const context
pub const fn pext_const(value: u64, mut mask: u64) -> u64 {
    let mut result = 0;
    let mut bit = 1;

    while mask != 0 {
        let lsb = mask & mask.wrapping_neg();

        if value & lsb != 0 {
            result |= bit;
        }

        mask ^= lsb;
        bit <<= 1;
    }

    result
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn psqt_mirror_symmetry_test() {
        for role in 0..6 {
            for sq in 0..64 {
                let mirrored = sq ^ 56;
                let white_idx = role; // adjust to however Piece::as_usize() actually orders White roles
                let black_idx = role + 6; // and Black roles
                assert_eq!(
                    PSQT.mg[white_idx][sq], PSQT.mg[black_idx][mirrored],
                    "role {role}, sq {sq}: white/black mirror mismatch"
                );
                assert_eq!(
                    PSQT.eg[white_idx][sq], PSQT.eg[black_idx][mirrored],
                    "role {role}, sq {sq}: white/black mirror mismatch (eg)"
                );
            }
        }
    }
}
