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
    pub static RANK_6: Bitboard = Bitboard::from_u64(0x0000FF0000000000);
    pub static RANK_5: Bitboard = Bitboard::from_u64(0x000000FF00000000);
    pub static RANK_4: Bitboard = Bitboard::from_u64(0x00000000FF000000);
    pub static RANK_3: Bitboard = Bitboard::from_u64(0x0000000000FF0000);
    pub static RANK_2: Bitboard = Bitboard::from_u64(0x000000000000FF00);
    pub static RANK_1: Bitboard = Bitboard::from_u64(0x00000000000000FF);

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
    #[track_caller]
    pub const fn first_square_checked(self) -> Square {
        let steps = self.0.trailing_zeros();
        Square::from_u32_checked(steps)
    }

    #[inline(always)]
    pub const unsafe fn first_square_unchecked(self) -> Square {
        let steps = self.0.trailing_zeros();
        unsafe { Square::from_u32_unchecked(steps) }
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
    // TODO: think about double north or south for pawn double move
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

impl Iterator for Bitboard {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        let bb = &mut self.0;
        if *bb == 0 {
            return None;
        }
        let index = bb.trailing_zeros();
        *bb &= *bb - 1;
        Some(Square::from_u32_checked(index))
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
mod bitboard_tests {
    use super::*;
    use crate::direction::Direction;
    use crate::square::Square;

    #[test]
    fn new_empty_is_zero() {
        let bb = Bitboard::new_empty();
        assert_eq!(bb.as_u64(), 0);
    }

    #[test]
    fn from_u64_round_trips() {
        assert_eq!(Bitboard::from_u64(0).as_u64(), 0);
        assert_eq!(Bitboard::from_u64(u64::MAX).as_u64(), u64::MAX);
        assert_eq!(Bitboard::from_u64(0xDEAD_BEEF).as_u64(), 0xDEAD_BEEF);
    }

    #[test]
    fn from_square_sets_exactly_one_bit() {
        for i in 0u32..64 {
            let sq = Square::from_u32_checked(i);
            let bb = Bitboard::from_square(sq);
            assert_eq!(
                bb.as_u64(),
                1u64 << i,
                "from_square({sq:?}) should set bit {i}"
            );
        }
    }

    #[test]
    fn from_square_trait_impl_matches_from_square() {
        for i in 0u32..64 {
            let sq = Square::from_u32_checked(i);
            let via_from: Bitboard = sq.into();
            let via_fn = Bitboard::from_square(sq);
            assert_eq!(via_from, via_fn);
        }
    }

    #[test]
    fn from_u64_trait_impl_matches_from_u64() {
        let raw = 0x0102_0408_1020_4080u64;
        let via_from: Bitboard = raw.into();
        let via_fn = Bitboard::from_u64(raw);
        assert_eq!(via_from, via_fn);
    }

    #[test]
    fn default_is_empty() {
        let bb = Bitboard::default();
        assert_eq!(bb, Bitboard::new_empty());
    }

    #[test]
    fn to_bb_from_u64() {
        let raw = 0xFF00u64;
        assert_eq!(raw.to_bb(), Bitboard::from_u64(raw));
    }

    #[test]
    fn to_bb_from_square() {
        for i in 0u32..64 {
            let sq = Square::from_u32_checked(i);
            assert_eq!(sq.to_bb(), Bitboard::from_square(sq));
        }
    }

    #[test]
    fn empty_on_zero_board() {
        assert!(Bitboard::new_empty().empty());
    }

    #[test]
    fn empty_false_when_any_bit_set() {
        assert!(!Square::A1.to_bb().empty());
        assert!(!Square::H8.to_bb().empty());
        assert!(!Bitboard::from_u64(u64::MAX).empty());
    }

    #[test]
    fn present_on_non_zero_board() {
        assert!(Square::A1.to_bb().present());
        assert!(Bitboard::from_u64(u64::MAX).present());
    }

    #[test]
    fn present_false_on_empty() {
        assert!(!Bitboard::new_empty().present());
    }

    #[test]
    fn popcnt_empty_is_zero() {
        assert_eq!(Bitboard::new_empty().popcnt(), 0);
    }

    #[test]
    fn popcnt_full_board_is_64() {
        assert_eq!(Bitboard::from_u64(u64::MAX).popcnt(), 64);
    }

    #[test]
    fn popcnt_single_square_is_one() {
        for i in 0u32..64 {
            let bb = Bitboard::from_square(Square::from_u32_checked(i));
            assert_eq!(bb.popcnt(), 1, "single-square board should have popcnt 1");
        }
    }

    #[test]
    fn popcnt_multiple_squares() {
        let bb = Bitboard::new_empty()
            .set_square(Square::A1)
            .set_square(Square::B2)
            .set_square(Square::C3)
            .set_square(Square::D4);
        assert_eq!(bb.popcnt(), 4);
    }

    #[test]
    fn set_square_and_is_square_set_all_64() {
        for i in 0u32..64 {
            let sq = Square::from_u32_checked(i);
            let bb = Bitboard::new_empty().set_square(sq);
            assert!(bb.is_square_set(sq), "{sq:?} should be set");
        }
    }

    #[test]
    fn set_square_does_not_affect_others() {
        let target = Square::E4;
        let bb = Bitboard::new_empty().set_square(target);
        for i in 0u32..64 {
            let sq = Square::from_u32_checked(i);
            if sq == target {
                assert!(bb.is_square_set(sq));
            } else {
                assert!(!bb.is_square_set(sq), "{sq:?} should not be set");
            }
        }
    }

    #[test]
    fn set_square_is_idempotent() {
        let bb = Bitboard::new_empty()
            .set_square(Square::D4)
            .set_square(Square::D4);
        assert_eq!(bb.popcnt(), 1);
    }

    #[test]
    fn is_square_set_false_on_empty() {
        let bb = Bitboard::new_empty();
        for i in 0u32..64 {
            assert!(!bb.is_square_set(Square::from_u32_checked(i)));
        }
    }

    #[test]
    fn clear_square_removes_set_bit() {
        for i in 0u32..64 {
            let sq = Square::from_u32_checked(i);
            let bb = Bitboard::new_empty().set_square(sq).clear_square(sq);
            assert!(!bb.is_square_set(sq));
            assert!(bb.empty());
        }
    }

    #[test]
    fn clear_square_on_unset_bit_is_noop() {
        let bb = Bitboard::new_empty().set_square(Square::E4);
        let bb2 = bb.clear_square(Square::D5); // D5 not set
        assert_eq!(bb, bb2);
    }

    #[test]
    fn clear_square_does_not_affect_others() {
        let bb = Bitboard::new_empty()
            .set_square(Square::A1)
            .set_square(Square::H8)
            .clear_square(Square::A1);

        assert!(!bb.is_square_set(Square::A1));
        assert!(bb.is_square_set(Square::H8));
    }

    #[test]
    fn clear_all_squares_produces_empty() {
        let squares = [Square::A1, Square::B2, Square::C3, Square::D4];
        let mut bb = squares
            .iter()
            .fold(Bitboard::new_empty(), |b, &s| b.set_square(s));
        for sq in squares {
            bb = bb.clear_square(sq);
        }
        assert!(bb.empty());
    }

    #[test]
    fn first_square_empty_is_none() {
        assert!(Bitboard::new_empty().first_square().is_none());
    }

    #[test]
    fn first_square_single_bit_all_64() {
        for i in 0u32..64 {
            let sq = Square::from_u32_checked(i);
            let bb = sq.to_bb();
            assert_eq!(bb.first_square(), Some(sq));
        }
    }

    #[test]
    fn first_square_is_lowest_index() {
        let bb = Bitboard::new_empty()
            .set_square(Square::E5)
            .set_square(Square::C3)
            .set_square(Square::A1);

        assert_eq!(bb.first_square(), Some(Square::A1));
    }

    #[test]
    fn first_square_full_board_is_a1() {
        assert_eq!(
            Bitboard::from_u64(u64::MAX).first_square(),
            Some(Square::A1)
        );
    }

    #[test]
    fn first_square_checked_matches_first_square() {
        for i in 0u32..64 {
            let sq = Square::from_u32_checked(i);
            let bb = sq.to_bb();
            assert_eq!(bb.first_square_checked(), bb.first_square().unwrap());
        }
    }

    #[test]
    fn last_square_empty_is_none() {
        assert!(Bitboard::new_empty().last_square().is_none());
    }

    #[test]
    fn last_square_single_bit_all_64() {
        for i in 0u32..64 {
            let sq = Square::from_u32_checked(i);
            let bb = sq.to_bb();
            assert_eq!(bb.last_square(), Some(sq));
        }
    }

    #[test]
    fn last_square_is_highest_index() {
        let bb = Bitboard::new_empty()
            .set_square(Square::A1)
            .set_square(Square::C3)
            .set_square(Square::H8);
        assert_eq!(bb.last_square(), Some(Square::H8));
    }

    #[test]
    fn last_square_full_board_is_h8() {
        assert_eq!(Bitboard::from_u64(u64::MAX).last_square(), Some(Square::H8));
    }

    #[test]
    fn first_and_last_agree_on_single_square() {
        for i in 0u32..64 {
            let sq = Square::from_u32_checked(i);
            let bb = sq.to_bb();
            assert_eq!(bb.first_square(), bb.last_square());
        }
    }

    #[test]
    fn last_square_checked_matches_last_square() {
        for i in 0u32..64 {
            let sq = Square::from_u32_checked(i);
            let bb = sq.to_bb();
            assert_eq!(bb.last_square_checked(), bb.last_square().unwrap());
        }
    }

    #[test]
    fn only_first_square_empty_is_none() {
        assert!(Bitboard::new_empty().only_first_square().is_none());
    }

    #[test]
    fn only_first_square_single_bit_is_some() {
        for i in 0u32..64 {
            let sq = Square::from_u32_checked(i);
            assert_eq!(sq.to_bb().only_first_square(), Some(sq));
        }
    }

    #[test]
    fn only_first_square_two_bits_is_none() {
        let bb = Bitboard::new_empty()
            .set_square(Square::A1)
            .set_square(Square::H8);
        assert!(bb.only_first_square().is_none());
    }

    #[test]
    fn only_first_square_full_board_is_none() {
        assert!(Bitboard::from_u64(u64::MAX).only_first_square().is_none());
    }

    #[test]
    fn for_each_empty_never_calls_closure() {
        let mut count = 0;
        Bitboard::new_empty().for_each(|_| count += 1);
        assert_eq!(count, 0);
    }

    #[test]
    fn for_each_visits_every_set_square() {
        let squares = [Square::A1, Square::C3, Square::E5, Square::H8];
        let bb = squares
            .iter()
            .fold(Bitboard::new_empty(), |b, &s| b.set_square(s));
        let mut visited = Vec::new();
        bb.for_each(|sq| visited.push(sq));
        assert_eq!(visited.len(), 4);
        for sq in squares {
            assert!(visited.contains(&sq), "{sq:?} should have been visited");
        }
    }

    #[test]
    fn for_each_visits_in_ascending_index_order() {
        let bb = Bitboard::new_empty()
            .set_square(Square::H8)
            .set_square(Square::A1)
            .set_square(Square::D4);
        let mut visited = Vec::new();
        bb.for_each(|sq| visited.push(sq.as_u32()));
        assert!(
            visited.windows(2).all(|w| w[0] < w[1]),
            "should be ascending"
        );
    }

    #[test]
    fn for_each_full_board_visits_64_squares() {
        let mut count = 0;
        Bitboard::from_u64(u64::MAX).for_each(|_| count += 1);
        assert_eq!(count, 64);
    }

    #[test]
    fn into_iter_owned_yields_same_as_for_each() {
        let squares = [Square::B2, Square::D4, Square::F6, Square::H8];
        let bb = squares
            .iter()
            .fold(Bitboard::new_empty(), |b, &s| b.set_square(s));

        let mut from_iter: Vec<Square> = bb.into_iter().collect();
        let mut from_for_each = Vec::new();
        bb.for_each(|sq| from_for_each.push(sq));

        from_iter.sort_by_key(|s| s.as_u32());
        from_for_each.sort_by_key(|s| s.as_u32());
        assert_eq!(from_iter, from_for_each);
    }

    #[test]
    fn into_iter_ref_yields_same_as_owned() {
        let bb = Bitboard::new_empty()
            .set_square(Square::A1)
            .set_square(Square::H8);

        let owned: Vec<Square> = bb.into_iter().collect();
        let by_ref: Vec<Square> = (&bb).into_iter().collect();
        assert_eq!(owned, by_ref);
    }

    #[test]
    fn into_iter_empty_produces_no_items() {
        let count = Bitboard::new_empty().into_iter().count();
        assert_eq!(count, 0);
    }

    #[test]
    fn into_iter_single_square() {
        let sq = Square::E4;
        let items: Vec<Square> = sq.to_bb().into_iter().collect();
        assert_eq!(items, vec![sq]);
    }

    #[test]
    fn intersects_overlapping_boards() {
        let a = Bitboard::new_empty()
            .set_square(Square::E4)
            .set_square(Square::D4);
        let b = Bitboard::new_empty()
            .set_square(Square::E4)
            .set_square(Square::F4);
        assert!(a.intersects(b));
    }

    #[test]
    fn intersects_disjoint_boards() {
        let a = Square::A1.to_bb();
        let b = Square::H8.to_bb();
        assert!(!a.intersects(b));
    }

    #[test]
    fn intersects_empty_board_is_false() {
        let bb = Bitboard::from_u64(u64::MAX);
        assert!(!bb.intersects(Bitboard::new_empty()));
        assert!(!Bitboard::new_empty().intersects(bb));
    }

    #[test]
    fn bitand_operator() {
        let a = Bitboard::from_u64(0xFF);
        let b = Bitboard::from_u64(0x0F);
        assert_eq!((a & b).as_u64(), 0x0F);
    }

    #[test]
    fn bitor_operator() {
        let a = Bitboard::from_u64(0xF0);
        let b = Bitboard::from_u64(0x0F);
        assert_eq!((a | b).as_u64(), 0xFF);
    }

    #[test]
    fn bitxor_operator() {
        let a = Bitboard::from_u64(0xFF);
        let b = Bitboard::from_u64(0x0F);
        assert_eq!((a ^ b).as_u64(), 0xF0);
    }

    #[test]
    fn not_operator() {
        assert_eq!((!Bitboard::new_empty()).as_u64(), u64::MAX);
        assert_eq!((!Bitboard::from_u64(u64::MAX)).as_u64(), 0);
    }

    #[test]
    fn bitand_matches_intersect_const() {
        let a = Bitboard::from_u64(0xABCD);
        let b = Bitboard::from_u64(0x1234);
        assert_eq!((a & b), a.intersect_const(b));
    }

    #[test]
    fn bitor_matches_union_const() {
        let a = Bitboard::from_u64(0xABCD);
        let b = Bitboard::from_u64(0x1234);
        assert_eq!((a | b), a.union_const(b));
    }

    #[test]
    fn not_matches_not_const() {
        let a = Bitboard::from_u64(0xDEAD_BEEF);
        assert_eq!(!a, a.not_const());
    }

    #[test]
    fn union_const_identity_with_empty() {
        let bb = Square::E4.to_bb();
        assert_eq!(bb.union_const(Bitboard::new_empty()), bb);
    }

    #[test]
    fn intersect_const_with_empty_is_empty() {
        let bb = Bitboard::from_u64(u64::MAX);
        assert!(bb.intersect_const(Bitboard::new_empty()).empty());
    }

    #[test]
    fn not_const_double_negation_is_identity() {
        let bb = Bitboard::from_u64(0x1234_5678_9ABC_DEF0);
        assert_eq!(bb.not_const().not_const(), bb);
    }

    #[test]
    fn shift_north_from_interior() {
        assert!(
            Square::C4
                .to_bb()
                .shift_dir(Direction::North)
                .is_square_set(Square::C5)
        );
    }

    #[test]
    fn shift_south_from_interior() {
        assert!(
            Square::C4
                .to_bb()
                .shift_dir(Direction::South)
                .is_square_set(Square::C3)
        );
    }

    #[test]
    fn shift_east_from_interior() {
        assert!(
            Square::C4
                .to_bb()
                .shift_dir(Direction::East)
                .is_square_set(Square::D4)
        );
    }

    #[test]
    fn shift_west_from_interior() {
        assert!(
            Square::C4
                .to_bb()
                .shift_dir(Direction::West)
                .is_square_set(Square::B4)
        );
    }

    #[test]
    fn shift_northeast_from_interior() {
        assert!(
            Square::C4
                .to_bb()
                .shift_dir(Direction::NorthEast)
                .is_square_set(Square::D5)
        );
    }

    #[test]
    fn shift_northwest_from_interior() {
        assert!(
            Square::C4
                .to_bb()
                .shift_dir(Direction::NorthWest)
                .is_square_set(Square::B5)
        );
    }

    #[test]
    fn shift_southeast_from_interior() {
        assert!(
            Square::C4
                .to_bb()
                .shift_dir(Direction::SouthEast)
                .is_square_set(Square::D3)
        );
    }

    #[test]
    fn shift_southwest_from_interior() {
        assert!(
            Square::C4
                .to_bb()
                .shift_dir(Direction::SouthWest)
                .is_square_set(Square::B3)
        );
    }

    #[test]
    fn shift_interior_produces_single_square() {
        let dirs = [
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
            Direction::NorthEast,
            Direction::NorthWest,
            Direction::SouthEast,
            Direction::SouthWest,
        ];
        for dir in dirs {
            let result = Square::D4.to_bb().shift_dir(dir);
            assert_eq!(
                result.popcnt(),
                1,
                "{dir:?} from D4 should produce exactly one square"
            );
        }
    }

    #[test]
    fn shift_west_from_a_file_is_empty() {
        for rank in 0u32..8 {
            let sq = Square::from_u32_checked(rank * 8); // A-file
            assert!(
                sq.to_bb().shift_dir(Direction::West).empty(),
                "West from {sq:?} should be empty"
            );
        }
    }

    #[test]
    fn shift_northwest_from_a_file_is_empty() {
        for rank in 0u32..7 {
            let sq = Square::from_u32_checked(rank * 8);
            assert!(sq.to_bb().shift_dir(Direction::NorthWest).empty());
        }
    }

    #[test]
    fn shift_southwest_from_a_file_is_empty() {
        for rank in 1u32..8 {
            let sq = Square::from_u32_checked(rank * 8);
            assert!(sq.to_bb().shift_dir(Direction::SouthWest).empty());
        }
    }

    #[test]
    fn shift_east_from_h_file_is_empty() {
        for rank in 0u32..8 {
            let sq = Square::from_u32_checked(rank * 8 + 7); // H-file
            assert!(
                sq.to_bb().shift_dir(Direction::East).empty(),
                "East from {sq:?} should be empty"
            );
        }
    }

    #[test]
    fn shift_northeast_from_h_file_is_empty() {
        for rank in 0u32..7 {
            let sq = Square::from_u32_checked(rank * 8 + 7);
            assert!(sq.to_bb().shift_dir(Direction::NorthEast).empty());
        }
    }

    #[test]
    fn shift_southeast_from_h_file_is_empty() {
        for rank in 1u32..8 {
            let sq = Square::from_u32_checked(rank * 8 + 7);
            assert!(sq.to_bb().shift_dir(Direction::SouthEast).empty());
        }
    }

    #[test]
    fn shift_north_from_rank_8_is_empty() {
        for file in 0u32..8 {
            let sq = Square::from_u32_checked(56 + file); // rank 8
            assert!(sq.to_bb().shift_dir(Direction::North).empty());
        }
    }

    #[test]
    fn shift_northeast_from_rank_8_is_empty() {
        for file in 0u32..7 {
            let sq = Square::from_u32_checked(56 + file);
            assert!(sq.to_bb().shift_dir(Direction::NorthEast).empty());
        }
    }

    #[test]
    fn shift_northwest_from_rank_8_is_empty() {
        for file in 1u32..8 {
            let sq = Square::from_u32_checked(56 + file);
            assert!(sq.to_bb().shift_dir(Direction::NorthWest).empty());
        }
    }

    #[test]
    fn shift_south_from_rank_1_is_empty() {
        for file in 0u32..8 {
            let sq = Square::from_u32_checked(file); // rank 1
            assert!(sq.to_bb().shift_dir(Direction::South).empty());
        }
    }

    #[test]
    fn shift_southeast_from_rank_1_is_empty() {
        for file in 0u32..7 {
            let sq = Square::from_u32_checked(file);
            assert!(sq.to_bb().shift_dir(Direction::SouthEast).empty());
        }
    }

    #[test]
    fn shift_southwest_from_rank_1_is_empty() {
        for file in 1u32..8 {
            let sq = Square::from_u32_checked(file);
            assert!(sq.to_bb().shift_dir(Direction::SouthWest).empty());
        }
    }

    #[test]
    fn shift_from_a1_corner() {
        let bb = Square::A1.to_bb();
        assert!(bb.shift_dir(Direction::South).empty());
        assert!(bb.shift_dir(Direction::West).empty());
        assert!(bb.shift_dir(Direction::SouthWest).empty());
        assert!(bb.shift_dir(Direction::SouthEast).empty());
        assert!(bb.shift_dir(Direction::NorthWest).empty());
        assert_eq!(
            bb.shift_dir(Direction::North).first_square(),
            Some(Square::A2)
        );
        assert_eq!(
            bb.shift_dir(Direction::East).first_square(),
            Some(Square::B1)
        );
        assert_eq!(
            bb.shift_dir(Direction::NorthEast).first_square(),
            Some(Square::B2)
        );
    }

    #[test]
    fn shift_from_h1_corner() {
        let bb = Square::H1.to_bb();
        assert!(bb.shift_dir(Direction::South).empty());
        assert!(bb.shift_dir(Direction::East).empty());
        assert!(bb.shift_dir(Direction::SouthEast).empty());
        assert!(bb.shift_dir(Direction::SouthWest).empty());
        assert!(bb.shift_dir(Direction::NorthEast).empty());
        assert_eq!(
            bb.shift_dir(Direction::North).first_square(),
            Some(Square::H2)
        );
        assert_eq!(
            bb.shift_dir(Direction::West).first_square(),
            Some(Square::G1)
        );
        assert_eq!(
            bb.shift_dir(Direction::NorthWest).first_square(),
            Some(Square::G2)
        );
    }

    #[test]
    fn shift_from_a8_corner() {
        let bb = Square::A8.to_bb();
        assert!(bb.shift_dir(Direction::North).empty());
        assert!(bb.shift_dir(Direction::West).empty());
        assert!(bb.shift_dir(Direction::NorthWest).empty());
        assert!(bb.shift_dir(Direction::NorthEast).empty());
        assert!(bb.shift_dir(Direction::SouthWest).empty());
        assert_eq!(
            bb.shift_dir(Direction::South).first_square(),
            Some(Square::A7)
        );
        assert_eq!(
            bb.shift_dir(Direction::East).first_square(),
            Some(Square::B8)
        );
        assert_eq!(
            bb.shift_dir(Direction::SouthEast).first_square(),
            Some(Square::B7)
        );
    }

    #[test]
    fn shift_from_h8_corner() {
        let bb = Square::H8.to_bb();
        assert!(bb.shift_dir(Direction::North).empty());
        assert!(bb.shift_dir(Direction::East).empty());
        assert!(bb.shift_dir(Direction::NorthEast).empty());
        assert!(bb.shift_dir(Direction::NorthWest).empty());
        assert!(bb.shift_dir(Direction::SouthEast).empty());
        assert_eq!(
            bb.shift_dir(Direction::South).first_square(),
            Some(Square::H7)
        );
        assert_eq!(
            bb.shift_dir(Direction::West).first_square(),
            Some(Square::G8)
        );
        assert_eq!(
            bb.shift_dir(Direction::SouthWest).first_square(),
            Some(Square::G7)
        );
    }

    #[test]
    fn shift_north_preserves_popcnt_on_interior_rank() {
        // All of rank 4 shifted north should give all of rank 5 (no overflow)
        let rank4: Bitboard = (0u32..8)
            .map(|f| Square::from_u32_checked(24 + f))
            .fold(Bitboard::new_empty(), |b, s| b.set_square(s));
        let shifted = rank4.shift_dir(Direction::North);
        assert_eq!(shifted.popcnt(), 8);
    }

    #[test]
    fn shift_east_preserves_popcnt_on_interior_file() {
        // All of B-file shifted east should give C-file (no overflow)
        let b_file: Bitboard = (0u32..8)
            .map(|r| Square::from_u32_checked(r * 8 + 1))
            .fold(Bitboard::new_empty(), |b, s| b.set_square(s));
        let shifted = b_file.shift_dir(Direction::East);
        assert_eq!(shifted.popcnt(), 8);
    }

    #[test]
    fn shift_dir_repeat_zero_is_identity() {
        let bb = Square::D4.to_bb();
        assert_eq!(bb.shift_dir_repeat(Direction::North, 0), bb);
    }

    #[test]
    fn shift_dir_repeat_one_matches_single_shift() {
        let bb = Square::D4.to_bb();
        assert_eq!(
            bb.shift_dir_repeat(Direction::North, 1),
            bb.shift_dir(Direction::North)
        );
    }

    #[test]
    fn shift_dir_repeat_north_four_ranks() {
        // D4 shifted north 4 should reach D8
        let result = Square::D4.to_bb().shift_dir_repeat(Direction::North, 4);
        assert_eq!(result.first_square(), Some(Square::D8));
    }

    #[test]
    fn shift_dir_repeat_past_edge_is_empty() {
        // A1 shifted south 1 already empty, repeat 8 should still be empty
        let result = Square::A1.to_bb().shift_dir_repeat(Direction::South, 8);
        assert!(result.empty());
    }

    #[test]
    fn shift_dir_repeat_matches_chained_shifts() {
        let bb = Square::E4.to_bb();
        let chained = bb
            .shift_dir(Direction::North)
            .shift_dir(Direction::North)
            .shift_dir(Direction::North);
        let repeated = bb.shift_dir_repeat(Direction::North, 3);
        assert_eq!(chained, repeated);
    }

    #[test]
    fn equality_same_board() {
        let bb = Bitboard::from_u64(0x1234);
        assert_eq!(bb, bb);
    }

    #[test]
    fn inequality_different_boards() {
        assert_ne!(Square::A1.to_bb(), Square::H8.to_bb());
    }

    #[test]
    fn empty_boards_are_equal() {
        assert_eq!(Bitboard::new_empty(), Bitboard::from_u64(0));
    }
}
