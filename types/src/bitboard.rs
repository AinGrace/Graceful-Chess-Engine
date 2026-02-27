use core::fmt;
use std::ops::{BitAnd, BitOr, BitXor, Not};

use crate::{
    bitboard::masks::{NOT_FILE_A, NOT_FILE_H},
    direction::Direction,
    square::Square,
};

#[rustfmt::skip]
pub mod masks {
    use crate::bitboard::Bitboard;

    pub static NOT_FILE_A:  u64  = 0xfefe_fefe_fefe_fefe;
    pub static NOT_FILE_B:  u64  = 0xfdfd_fdfd_fdfd_fdfd;
    pub static NOT_FILE_AB: u64  = NOT_FILE_A & NOT_FILE_B;

    pub static NOT_FILE_G:  u64  = 0xbfbf_bfbf_bfbf_bfbf;
    pub static NOT_FILE_H:  u64  = 0x7f7f_7f7f_7f7f_7f7f;
    pub static NOT_FILE_GH: u64  = NOT_FILE_G & NOT_FILE_H;

    pub static RANK_8: Bitboard = Bitboard::from_u64(0xFF00000000000000);
    pub static RANK_7: Bitboard = Bitboard::from_u64(0x00FF000000000000);
    pub static RANK_1: Bitboard = Bitboard::from_u64(0x00000000000000FF);
    pub static RANK_2: Bitboard = Bitboard::from_u64(0x000000000000FF00);

    pub static OUTER_LAYER: Bitboard = Bitboard::from_u64(RANK_8.as_u64() | RANK_1.as_u64() | !NOT_FILE_A | !NOT_FILE_H);
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Bitboard(u64);

impl Bitboard {
    #[inline(always)]
    pub const fn new_empty() -> Self {
        Self(0)
    }

    #[inline(always)]
    pub const fn from_u64(num: u64) -> Self {
        Self(num)
    }

    #[inline(always)]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    #[inline(always)]
    pub const fn union_const(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[inline(always)]
    pub const fn intersect_const(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    #[inline(always)]
    pub const fn not_const(self) -> Self {
        Self(!self.0)
    }

    #[inline(always)]
    pub fn from_square(square: Square) -> Self {
        Self(1 << square.as_u32())
    }

    #[inline(always)]
    pub fn is_square_set(self, square: Square) -> bool {
        let mask = 1 << square.as_u32();
        self.0 & mask != 0
    }

    #[inline(always)]
    pub const fn set_square(self, square: Square) -> Self {
        Self(self.0 | 1 << square.as_u32())
    }

    #[inline(always)]
    pub const fn clear_square(self, square: Square) -> Self {
        let mask = 1u64 << square.as_u32();
        Self(self.0 & !mask)
    }

    #[inline(always)]
    pub const fn first_square(self) -> Option<Square> {
        let steps = self.0.trailing_zeros();
        if steps == 64 {
            None
        } else {
            Some(Square::from_u32_checked(steps))
        }
    }

    #[inline(always)]
    pub const fn first_square_checked(self) -> Square {
        let steps = self.0.trailing_zeros();
        Square::from_u32_checked(steps)
    }

    #[inline(always)]
    pub const fn only_first_square(self) -> Option<Square> {
        if self.popcnt() == 1 {
            Some(self.first_square_checked())
        } else {
            None
        }
    }

    #[inline(always)]
    pub const fn last_square(self) -> Option<Square> {
        let steps = self.0.leading_zeros();

        if steps == 64 {
            None
        } else {
            Some(Square::from_u32_checked(63 - steps))
        }
    }

    #[inline(always)]
    pub const fn last_square_checked(self) -> Square {
        let steps = self.0.leading_zeros();
        Square::from_u32_checked(63 - steps)
    }

    #[inline(always)]
    pub fn for_each<F>(self, mut f: F)
    where
        F: FnMut(Square),
    {
        let mut bb = self.0;
        while bb != 0 {
            let sq = bb.trailing_zeros();
            bb &= bb - 1;
            f(Square::from_u32_checked(sq))
        }
    }

    #[inline(always)]
    pub const fn empty(&self) -> bool {
        self.0 == 0
    }

    #[inline(always)]
    pub const fn present(&self) -> bool {
        self.0 != 0
    }

    #[inline(always)]
    pub const fn popcnt(self) -> u32 {
        self.0.count_ones()
    }

    #[inline(always)]
    pub fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    #[rustfmt::skip]
    #[inline(always)]
    pub const fn shift_dir(self, dir: Direction) -> Self {
        match dir {
            Direction::North     => Self(self.0 << 8),
            Direction::South     => Self(self.0 >> 8),
            Direction::NorthEast => Self((self.0 << 9) & NOT_FILE_A),
            Direction::SouthWest => Self((self.0 >> 9) & NOT_FILE_H),
            Direction::NorthWest => Self((self.0 << 7) & NOT_FILE_H),
            Direction::SouthEast => Self((self.0 >> 7) & NOT_FILE_A),
            Direction::East      => Self((self.0 << 1) & NOT_FILE_A),
            Direction::West      => Self((self.0 >> 1) & NOT_FILE_H),
        }
    }

    #[inline(always)]
    pub const fn shift_dir_repeat(self, dir: Direction, repeat: u32) -> Self {
        let mut pivot = self;
        let mut i = 0;
        while i < repeat {
            pivot = pivot.shift_dir(dir);
            i += 1;
        }
        pivot
    }
}

impl Default for Bitboard {
    fn default() -> Self {
        Self::new_empty()
    }
}

impl fmt::Debug for Bitboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{")?;

        write!(f, "    ")?;
        for x in 0..64 {
            if self.0.swap_bytes() & (1u64 << x) == (1u64 << x) {
                write!(f, "X ")?;
            } else {
                write!(f, ". ")?;
            }
            if x % 8 == 7 && x != 63 {
                writeln!(f)?;
                write!(f, "    ")?;
            }
        }

        write!(f, "\n}}")?;

        Ok(())
    }
}

impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitXor for Bitboard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl IntoIterator for Bitboard {
    type Item = Square;

    type IntoIter = BitboardIter;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter { inner: self }
    }
}

impl IntoIterator for &Bitboard {
    type Item = Square;

    type IntoIter = BitboardIter;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter { inner: *self }
    }
}

pub struct BitboardIter {
    inner: Bitboard,
}

impl Iterator for BitboardIter {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        let bb = self.inner.0;

        if bb == 0 {
            return None;
        }

        let sq = bb.trailing_zeros();
        self.inner.0 &= bb - 1;

        Some(Square::from_u32_checked(sq))
    }
}

impl From<u64> for Bitboard {
    #[inline(always)]
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<Square> for Bitboard {
    #[inline(always)]
    fn from(value: Square) -> Self {
        Self::from_square(value)
    }
}

pub trait ToBitboard {
    fn to_bb(self) -> Bitboard;
}

impl ToBitboard for u64 {
    #[inline(always)]
    fn to_bb(self) -> Bitboard {
        Bitboard(self)
    }
}

impl ToBitboard for Square {
    #[inline(always)]
    fn to_bb(self) -> Bitboard {
        Bitboard::from_square(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_square_should_occupy_valid_index_on_bb() {
        let bb = Square::A8.to_bb();

        assert!(bb.is_square_set(Square::A8));
        assert!(bb.first_square().is_some());
        assert!(bb.last_square().is_some());
        assert!(bb.last_square() == bb.first_square());
        assert!(bb.popcnt() == 1);
    }

    #[test]
    fn clear_square_should_unset_valid_square() {
        let mut bb = Square::A8.to_bb();

        bb = bb.clear_square(Square::A8);

        assert!(bb.is_square_set(Square::A8).not());
        assert!(bb.first_square().is_none());
        assert!(bb.last_square().is_none());
        assert!(bb.popcnt() == 0);
    }

    #[test]
    fn first_square_should_point_to_valid_square() {
        let bb = Square::H8.to_bb();
        assert_eq!(bb.first_square().unwrap(), Square::H8);

        let bb = Square::A8.to_bb();
        assert_eq!(bb.first_square().unwrap(), Square::A8);

        let bb = Square::A1.to_bb();
        assert_eq!(bb.first_square().unwrap(), Square::A1);

        let bb = Square::H1.to_bb();
        assert_eq!(bb.first_square().unwrap(), Square::H1);

        let bb = Square::E5.to_bb();
        assert_eq!(bb.first_square().unwrap(), Square::E5);

        let bb = Square::D4.to_bb();
        assert_eq!(bb.first_square().unwrap(), Square::D4);

        let bb = Bitboard::new_empty()
            .set_square(Square::A8)
            .set_square(Square::H8)
            .set_square(Square::A1)
            .set_square(Square::H1);

        assert_eq!(bb.first_square().unwrap(), Square::A1);

        let bb = Bitboard::new_empty()
            .set_square(Square::D6)
            .set_square(Square::D5)
            .set_square(Square::D4)
            .set_square(Square::D3);

        assert_eq!(bb.first_square().unwrap(), Square::D3);

        let bb = Bitboard::new_empty()
            .set_square(Square::D6)
            .set_square(Square::C5)
            .set_square(Square::B4)
            .set_square(Square::A3);

        assert_eq!(bb.first_square().unwrap(), Square::A3);
    }

    #[test]
    fn last_square_should_point_to_valid_square() {
        let bb = Square::H8.to_bb();
        assert_eq!(bb.last_square().unwrap(), Square::H8);

        let bb = Square::A8.to_bb();
        assert_eq!(bb.last_square().unwrap(), Square::A8);

        let bb = Square::A1.to_bb();
        assert_eq!(bb.last_square().unwrap(), Square::A1);

        let bb = Square::H1.to_bb();
        assert_eq!(bb.last_square().unwrap(), Square::H1);

        let bb = Square::E5.to_bb();
        assert_eq!(bb.last_square().unwrap(), Square::E5);

        let bb = Square::D4.to_bb();
        assert_eq!(bb.last_square().unwrap(), Square::D4);

        let bb = Bitboard::new_empty()
            .set_square(Square::A8)
            .set_square(Square::H8)
            .set_square(Square::A1)
            .set_square(Square::H1);

        assert_eq!(bb.last_square().unwrap(), Square::H8);

        let bb = Bitboard::new_empty()
            .set_square(Square::D6)
            .set_square(Square::D5)
            .set_square(Square::D4)
            .set_square(Square::D3);

        assert_eq!(bb.last_square().unwrap(), Square::D6);

        let bb = Bitboard::new_empty()
            .set_square(Square::D6)
            .set_square(Square::C5)
            .set_square(Square::B4)
            .set_square(Square::A3);

        assert_eq!(bb.last_square().unwrap(), Square::D6);
    }

    #[test]
    fn shift_dir_should_shift_by_valid_amount_and_respect_wrapping() {
        let bb = Square::C4.to_bb();

        let shifted_bb = bb.shift_dir(Direction::North);
        assert!(shifted_bb.is_square_set(Square::C5));

        let shifted_bb = bb.shift_dir(Direction::NorthEast);
        assert!(shifted_bb.is_square_set(Square::D5));

        let shifted_bb = bb.shift_dir(Direction::NorthWest);
        assert!(shifted_bb.is_square_set(Square::B5));

        let shifted_bb = bb.shift_dir(Direction::East);
        assert!(shifted_bb.is_square_set(Square::D4));

        let shifted_bb = bb.shift_dir(Direction::West);
        assert!(shifted_bb.is_square_set(Square::B4));

        let shifted_bb = bb.shift_dir(Direction::South);
        assert!(shifted_bb.is_square_set(Square::C3));

        let shifted_bb = bb.shift_dir(Direction::SouthEast);
        assert!(shifted_bb.is_square_set(Square::D3));

        let shifted_bb = bb.shift_dir(Direction::SouthWest);
        assert!(shifted_bb.is_square_set(Square::B3));

        let bb = Square::A8.to_bb();

        let shifted_bb = bb.shift_dir(Direction::North);
        assert!(shifted_bb.empty());

        let shifted_bb = bb.shift_dir(Direction::South);
        assert!(shifted_bb.is_square_set(Square::A7));

        let shifted_bb = bb.shift_dir(Direction::NorthEast);
        assert!(shifted_bb.empty());

        let shifted_bb = bb.shift_dir(Direction::NorthWest);
        assert!(shifted_bb.empty());

        let shifted_bb = bb.shift_dir(Direction::East);
        assert!(shifted_bb.is_square_set(Square::B8));

        let shifted_bb = bb.shift_dir(Direction::West);
        assert!(shifted_bb.empty());

        let shifted_bb = bb.shift_dir(Direction::SouthEast);
        assert!(shifted_bb.is_square_set(Square::B7));

        let shifted_bb = bb.shift_dir(Direction::SouthWest);
        assert!(shifted_bb.empty());
    }
}
