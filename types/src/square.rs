use std::{fmt::Display, mem::transmute};

use crate::{file::File, rank::Rank};

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Square {
    A8 = 56, B8 = 57, C8 = 58, D8 = 59, E8 = 60, F8 = 61, G8 = 62, H8 = 63,
    A7 = 48, B7 = 49, C7 = 50, D7 = 51, E7 = 52, F7 = 53, G7 = 54, H7 = 55,
    A6 = 40, B6 = 41, C6 = 42, D6 = 43, E6 = 44, F6 = 45, G6 = 46, H6 = 47,
    A5 = 32, B5 = 33, C5 = 34, D5 = 35, E5 = 36, F5 = 37, G5 = 38, H5 = 39,
    A4 = 24, B4 = 25, C4 = 26, D4 = 27, E4 = 28, F4 = 29, G4 = 30, H4 = 31,
    A3 = 16, B3 = 17, C3 = 18, D3 = 19, E3 = 20, F3 = 21, G3 = 22, H3 = 23,
    A2 =  8, B2 =  9, C2 = 10, D2 = 11, E2 = 12, F2 = 13, G2 = 14, H2 = 15,
    A1 =  0, B1 =  1, C1 =  2, D1 =  3, E1 =  4, F1 =  5, G1 =  6, H1 =  7,
}

impl Square {
    pub fn of(file: File, rank: Rank) -> Self {
        Self::from_u32_checked((rank.to_u32() * 8) + file.to_u32())
    }

    pub const fn from_u32(index: u32) -> Option<Self> {
        if index >= 64 {
            None
        } else {
            Some(Self::from_u32_checked(index))
        }
    }

    /// Will panic if index is >= 64
    #[rustfmt::skip]
    #[inline(always)]
    pub const fn from_u32_checked(index: u32) -> Self {
        assert!(index < 64);

        // SAFETY: index is always at valid range
        unsafe { transmute(index as u8) }
    }

    #[inline(always)]
    pub fn offset(self, by: i32) -> Option<Self> {
        let idx = self as i32 + by;

        if (0..64).contains(&idx) {
            // SAFETY: index is always at valid range
            Some(unsafe { transmute::<u8, Self>(idx as u8) })
        } else {
            None
        }
    }

    pub fn offset_checked(self, by: i32) -> Self {
        self.offset(by).unwrap()
    }

    #[inline(always)]
    pub const fn as_u32(self) -> u32 {
        self as u32
    }

    #[inline(always)]
    pub const fn as_usize(self) -> usize {
        self as usize
    }

    #[inline(always)]
    pub const fn as_mask(self) -> u64 {
        1 << self.as_u32()
    }

    #[inline(always)]
    pub const fn file(self) -> File {
        File::from_u32_checked(self.as_u32() & 7)
    }

    #[inline(always)]
    pub const fn rank(self) -> Rank {
        Rank::from_u32_checked(self.as_u32() >> 3)
    }

    #[inline(always)]
    pub const fn abs_diff(left: Square, right: Square) -> u32 {
        (left as i32 - right as i32).unsigned_abs()
    }

    #[inline(always)]
    pub const fn is_dark_square(self) -> bool {
        self.as_u32().is_multiple_of(2)
    }

    #[inline(always)]
    pub const fn mirror_vertical(self) -> Self {
        Self::from_u32_checked(self.as_u32() ^ 56)
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.file().char(), self.rank().char())
    }
}

#[cfg(test)]
mod square_tests {
    use super::*;

    #[test]
    fn discriminants_rank_1() {
        assert_eq!(Square::A1 as u8, 0);
        assert_eq!(Square::B1 as u8, 1);
        assert_eq!(Square::C1 as u8, 2);
        assert_eq!(Square::D1 as u8, 3);
        assert_eq!(Square::E1 as u8, 4);
        assert_eq!(Square::F1 as u8, 5);
        assert_eq!(Square::G1 as u8, 6);
        assert_eq!(Square::H1 as u8, 7);
    }

    #[test]
    fn discriminants_rank_2() {
        assert_eq!(Square::A2 as u8, 8);
        assert_eq!(Square::B2 as u8, 9);
        assert_eq!(Square::H2 as u8, 15);
    }

    #[test]
    fn discriminants_rank_8() {
        assert_eq!(Square::A8 as u8, 56);
        assert_eq!(Square::B8 as u8, 57);
        assert_eq!(Square::H8 as u8, 63);
    }

    #[test]
    fn discriminants_spot_check() {
        assert_eq!(Square::E4 as u8, 28);
        assert_eq!(Square::D5 as u8, 35);
        assert_eq!(Square::G6 as u8, 46);
        assert_eq!(Square::C7 as u8, 50);
    }

    #[test]
    fn from_u32_zero_is_a1() {
        assert_eq!(Square::from_u32(0), Some(Square::A1));
    }

    #[test]
    fn from_u32_63_is_h8() {
        assert_eq!(Square::from_u32(63), Some(Square::H8));
    }

    #[test]
    fn from_u32_all_64_squares() {
        for i in 0u32..64 {
            assert!(Square::from_u32(i).is_some(), "index {i} should be Some");
        }
    }

    #[test]
    fn from_u32_round_trips_as_u32() {
        for i in 0u32..64 {
            let sq = Square::from_u32(i).unwrap();
            assert_eq!(sq.as_u32(), i);
        }
    }

    #[test]
    fn from_u32_64_is_none() {
        assert_eq!(Square::from_u32(64), None);
    }

    #[test]
    fn from_u32_large_values_are_none() {
        assert_eq!(Square::from_u32(100), None);
        assert_eq!(Square::from_u32(u32::MAX), None);
        assert_eq!(Square::from_u32(255), None);
    }

    #[test]
    fn from_u32_checked_valid_boundaries() {
        assert_eq!(Square::from_u32_checked(0), Square::A1);
        assert_eq!(Square::from_u32_checked(63), Square::H8);
    }

    #[test]
    fn from_u32_checked_all_valid() {
        for i in 0u32..64 {
            let sq = Square::from_u32_checked(i);
            assert_eq!(sq.as_u32(), i);
        }
    }

    #[test]
    #[should_panic]
    fn from_u32_checked_64_panics() {
        let _ = Square::from_u32_checked(64);
    }

    #[test]
    #[should_panic]
    fn from_u32_checked_large_panics() {
        let _ = Square::from_u32_checked(200);
    }

    #[test]
    fn as_u32_matches_discriminant() {
        assert_eq!(Square::A1.as_u32(), 0);
        assert_eq!(Square::H1.as_u32(), 7);
        assert_eq!(Square::A8.as_u32(), 56);
        assert_eq!(Square::H8.as_u32(), 63);
        assert_eq!(Square::E4.as_u32(), 28);
    }

    #[test]
    fn as_usize_matches_as_u32() {
        for i in 0u32..64 {
            let sq = Square::from_u32_checked(i);
            assert_eq!(sq.as_usize(), i as usize);
        }
    }

    #[test]
    fn mask_a1_is_bit_0() {
        assert_eq!(Square::A1.as_mask(), 1u64);
    }

    #[test]
    fn mask_b1_is_bit_1() {
        assert_eq!(Square::B1.as_mask(), 0b10u64);
    }

    #[test]
    fn mask_h8_is_bit_63() {
        assert_eq!(Square::H8.as_mask(), 1u64 << 63);
    }

    #[test]
    fn mask_e4() {
        assert_eq!(Square::E4.as_mask(), 1u64 << 28);
    }

    #[test]
    fn masks_are_unique_powers_of_two() {
        let mut seen = 0u64;
        for i in 0u32..64 {
            let mask = Square::from_u32_checked(i).as_mask();
            assert_eq!(mask.count_ones(), 1, "mask should be a single bit");
            assert_eq!(seen & mask, 0, "mask should not overlap with previous");
            seen |= mask;
        }
        assert_eq!(seen, u64::MAX);
    }

    #[test]
    fn file_of_a_squares_is_file_a() {
        for rank_idx in 0u32..8 {
            let sq = Square::from_u32_checked(rank_idx * 8); // A-file
            assert_eq!(
                sq.file(),
                File::from_u32_checked(0),
                "expected file A for {sq:?}"
            );
        }
    }

    #[test]
    fn file_of_h_squares_is_file_h() {
        for rank_idx in 0u32..8 {
            let sq = Square::from_u32_checked(rank_idx * 8 + 7); // H-file
            assert_eq!(
                sq.file(),
                File::from_u32_checked(7),
                "expected file H for {sq:?}"
            );
        }
    }

    #[test]
    fn file_cycles_0_to_7_across_ranks() {
        for i in 0u32..64 {
            let sq = Square::from_u32_checked(i);
            let expected_file = i % 8;
            assert_eq!(sq.file(), File::from_u32_checked(expected_file));
        }
    }

    #[test]
    fn file_spot_checks() {
        assert_eq!(Square::E4.file(), File::E);
        assert_eq!(Square::D5.file(), File::D);
        assert_eq!(Square::G7.file(), File::G);
    }

    #[test]
    fn rank_of_first_rank_squares() {
        for file_idx in 0u32..8 {
            let sq = Square::from_u32_checked(file_idx); // rank 1
            assert_eq!(
                sq.rank(),
                Rank::from_u32_checked(0),
                "expected rank 1 for {sq:?}"
            );
        }
    }

    #[test]
    fn rank_of_eighth_rank_squares() {
        for file_idx in 0u32..8 {
            let sq = Square::from_u32_checked(56 + file_idx); // rank 8
            assert_eq!(
                sq.rank(),
                Rank::from_u32_checked(7),
                "expected rank 8 for {sq:?}"
            );
        }
    }

    #[test]
    fn rank_is_index_divided_by_8() {
        for i in 0u32..64 {
            let sq = Square::from_u32_checked(i);
            let expected_rank = i / 8;
            assert_eq!(sq.rank(), Rank::from_u32_checked(expected_rank));
        }
    }

    #[test]
    fn rank_spot_checks() {
        assert_eq!(Square::E4.rank(), Rank::Fourth);
        assert_eq!(Square::D5.rank(), Rank::Fifth);
        assert_eq!(Square::A8.rank(), Rank::Eighth);
    }

    #[test]
    fn of_a1_is_a1() {
        let sq = Square::of(File::A, Rank::First);
        assert_eq!(sq, Square::A1);
    }

    #[test]
    fn of_h8_is_h8() {
        let sq = Square::of(File::H, Rank::Eighth);
        assert_eq!(sq, Square::H8);
    }

    #[test]
    fn of_round_trips_file_and_rank() {
        for file_idx in 0u32..8 {
            for rank_idx in 0u32..8 {
                let file = File::from_u32_checked(file_idx);
                let rank = Rank::from_u32_checked(rank_idx);
                let sq = Square::of(file, rank);
                assert_eq!(sq.file(), file);
                assert_eq!(sq.rank(), rank);
            }
        }
    }

    #[test]
    fn of_matches_from_u32_checked() {
        for file_idx in 0u32..8 {
            for rank_idx in 0u32..8 {
                let sq_of = Square::of(
                    File::from_u32_checked(file_idx),
                    Rank::from_u32_checked(rank_idx),
                );
                let sq_idx = Square::from_u32_checked(rank_idx * 8 + file_idx);
                assert_eq!(sq_of, sq_idx);
            }
        }
    }

    #[test]
    fn offset_zero_is_identity() {
        for i in 0u32..64 {
            let sq = Square::from_u32_checked(i);
            assert_eq!(sq.offset(0), Some(sq));
        }
    }

    #[test]
    fn offset_positive_in_range() {
        assert_eq!(Square::A1.offset(1), Some(Square::B1));
        assert_eq!(Square::A1.offset(8), Some(Square::A2));
        assert_eq!(Square::A1.offset(63), Some(Square::H8));
    }

    #[test]
    fn offset_negative_in_range() {
        assert_eq!(Square::H8.offset(-1), Some(Square::G8));
        assert_eq!(Square::H8.offset(-8), Some(Square::H7));
        assert_eq!(Square::H8.offset(-63), Some(Square::A1));
    }

    #[test]
    fn offset_out_of_range_returns_none() {
        assert_eq!(Square::A1.offset(-1), None);
        assert_eq!(Square::A1.offset(-100), None);
        assert_eq!(Square::H8.offset(1), None);
        assert_eq!(Square::H8.offset(100), None);
    }

    #[test]
    fn offset_boundary_exact() {
        assert_eq!(Square::A1.offset(63), Some(Square::H8));
        assert_eq!(Square::H8.offset(-63), Some(Square::A1));

        assert_eq!(Square::A1.offset(-1), None);
        assert_eq!(Square::H8.offset(1), None);
    }

    #[test]
    fn offset_checked_valid() {
        assert_eq!(Square::E4.offset_checked(1), Square::F4);
        assert_eq!(Square::E4.offset_checked(-1), Square::D4);
        assert_eq!(Square::E4.offset_checked(8), Square::E5);
    }

    #[test]
    #[should_panic]
    fn offset_checked_panics_on_overflow() {
        let _ = Square::H8.offset_checked(1);
    }

    #[test]
    #[should_panic]
    fn offset_checked_panics_on_underflow() {
        let _ = Square::A1.offset_checked(-1);
    }

    #[test]
    fn abs_diff_same_square_is_zero() {
        assert_eq!(Square::abs_diff(Square::E4, Square::E4), 0);
        assert_eq!(Square::abs_diff(Square::A1, Square::A1), 0);
        assert_eq!(Square::abs_diff(Square::H8, Square::H8), 0);
    }

    #[test]
    fn abs_diff_is_symmetric() {
        let pairs = [
            (Square::A1, Square::H8),
            (Square::E4, Square::D5),
            (Square::B2, Square::G7),
        ];
        for (a, b) in pairs {
            assert_eq!(Square::abs_diff(a, b), Square::abs_diff(b, a));
        }
    }

    #[test]
    fn abs_diff_adjacent_squares() {
        assert_eq!(Square::abs_diff(Square::A1, Square::B1), 1);
        assert_eq!(Square::abs_diff(Square::A1, Square::A2), 8);
    }

    #[test]
    fn abs_diff_a1_to_h8() {
        assert_eq!(Square::abs_diff(Square::A1, Square::H8), 63);
    }

    #[test]
    fn abs_diff_matches_index_difference() {
        let pairs = [
            (Square::C3, Square::F6),
            (Square::A1, Square::H8),
            (Square::D4, Square::D5),
            (Square::B2, Square::B2),
        ];
        for (a, b) in pairs {
            let expected = (a.as_u32() as i32 - b.as_u32() as i32).unsigned_abs();
            assert_eq!(Square::abs_diff(a, b), expected);
        }
    }

    #[test]
    fn all_64_squares_correct_index() {
        let table: &[(Square, u32)] = &[
            (Square::A1, 0),
            (Square::B1, 1),
            (Square::C1, 2),
            (Square::D1, 3),
            (Square::E1, 4),
            (Square::F1, 5),
            (Square::G1, 6),
            (Square::H1, 7),
            (Square::A2, 8),
            (Square::B2, 9),
            (Square::C2, 10),
            (Square::D2, 11),
            (Square::E2, 12),
            (Square::F2, 13),
            (Square::G2, 14),
            (Square::H2, 15),
            (Square::A3, 16),
            (Square::B3, 17),
            (Square::C3, 18),
            (Square::D3, 19),
            (Square::E3, 20),
            (Square::F3, 21),
            (Square::G3, 22),
            (Square::H3, 23),
            (Square::A4, 24),
            (Square::B4, 25),
            (Square::C4, 26),
            (Square::D4, 27),
            (Square::E4, 28),
            (Square::F4, 29),
            (Square::G4, 30),
            (Square::H4, 31),
            (Square::A5, 32),
            (Square::B5, 33),
            (Square::C5, 34),
            (Square::D5, 35),
            (Square::E5, 36),
            (Square::F5, 37),
            (Square::G5, 38),
            (Square::H5, 39),
            (Square::A6, 40),
            (Square::B6, 41),
            (Square::C6, 42),
            (Square::D6, 43),
            (Square::E6, 44),
            (Square::F6, 45),
            (Square::G6, 46),
            (Square::H6, 47),
            (Square::A7, 48),
            (Square::B7, 49),
            (Square::C7, 50),
            (Square::D7, 51),
            (Square::E7, 52),
            (Square::F7, 53),
            (Square::G7, 54),
            (Square::H7, 55),
            (Square::A8, 56),
            (Square::B8, 57),
            (Square::C8, 58),
            (Square::D8, 59),
            (Square::E8, 60),
            (Square::F8, 61),
            (Square::G8, 62),
            (Square::H8, 63),
        ];
        for &(sq, expected_idx) in table {
            assert_eq!(
                sq.as_u32(),
                expected_idx,
                "{sq:?} should have index {expected_idx}"
            );
            assert_eq!(Square::from_u32(expected_idx), Some(sq));
        }
    }
}
