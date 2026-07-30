#![expect(
    long_running_const_eval,
    reason = "Lookup tables take some time to generate during compile time"
)]
use std::arch::x86_64::_pext_u64;

use types::{
    bitboard::{
        Bitboard,
        masks::{NOT_FILE_A, NOT_FILE_AB, NOT_FILE_GH, NOT_FILE_H, OUTER_LAYER, RANK_1, RANK_8},
    },
    color::Color,
    direction::Direction,
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

// ------------------------------------------- //

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

#[cfg(not(miri))]
#[target_feature(enable = "bmi2")]
unsafe fn pext(value: u64, mask: u64) -> u64 {
    _pext_u64(value, mask)
}

#[inline(always)]
#[cfg(miri)]
fn pext(value: u64, mask: u64) -> u64 {
    pext_const(value, mask)
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
