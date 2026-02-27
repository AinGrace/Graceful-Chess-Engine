#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i8)]
pub enum Direction {
    North = 8,
    NorthWest = 7,
    NorthEast = 9,
    West = -1,
    East = 1,
    South = -8,
    SouthWest = -9,
    SouthEast = -7,
}

#[rustfmt::skip]
impl Direction {
    #[inline(always)]
    pub const fn offset(self) -> i32 {
        self as i32
    }

    #[inline(always)]
    pub fn invert(self) -> Self {
        match self {
            Self::North     => Self::South,
            Self::NorthWest => Self::SouthEast,
            Self::NorthEast => Self::SouthWest,
            Self::West      => Self::East,
            Self::East      => Self::West,
            Self::South     => Self::North,
            Self::SouthWest => Self::NorthEast,
            Self::SouthEast => Self::NorthWest,
        }
    }

    pub const fn is_anti(&self) -> bool {
        if self.offset().signum() == 1 {
            true
        } else {
            false
        }
    }
}
