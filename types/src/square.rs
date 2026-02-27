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
    pub const fn of(file: File, rank: Rank) -> Self {
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
    pub const fn from_u32_checked(index: u32) -> Self {
        assert!(index < 64);

        // SAFETY: index is always at valid range
        unsafe { transmute(index as u8) }
    }

    pub fn offset(self, by: i32) -> Option<Self> {
        let idx = self as i32 + by;

        if idx >= 0 && idx < 64 {
            // SAFETY: index is always at valid range
            Some(unsafe { transmute(idx as u8) })
        } else {
            None
        }
    }

    pub fn offset_checked(self, by: i32) -> Self {
        self.offset(by).unwrap()
    }

    pub const fn as_u32(self) -> u32 {
        self as u32
    }

    pub const fn as_usize(self) -> usize {
        self as usize
    }

    pub const fn as_mask(self) -> u64 {
        1 << self.as_u32()
    }

    pub const fn file(self) -> File {
        File::new_checked(self.as_u32() & 7)
    }

    pub const fn rank(self) -> Rank {
        Rank::new_checked(self.as_u32() >> 3)
    }
}

impl Square {
    /// An iterator over all values of Square enum starting from A1, B1, C1...
    pub fn values() -> impl Iterator<Item = Self> {
        (0..64).map(Self::from_u32_checked)
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.file().char(), self.rank().char())
    }
}
