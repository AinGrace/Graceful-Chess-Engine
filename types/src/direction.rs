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
        self.offset().signum() == 1 
    }
}

#[cfg(test)]
mod direction_tests {
    use super::*;

    const ALL_DIRS: [Direction; 8] = [
        Direction::North,
        Direction::NorthWest,
        Direction::NorthEast,
        Direction::West,
        Direction::East,
        Direction::South,
        Direction::SouthWest,
        Direction::SouthEast,
    ];

    #[test]
    fn discriminants_are_correct() {
        assert_eq!(Direction::North as i8, 8);
        assert_eq!(Direction::NorthWest as i8, 7);
        assert_eq!(Direction::NorthEast as i8, 9);
        assert_eq!(Direction::West as i8, -1);
        assert_eq!(Direction::East as i8, 1);
        assert_eq!(Direction::South as i8, -8);
        assert_eq!(Direction::SouthWest as i8, -9);
        assert_eq!(Direction::SouthEast as i8, -7);
    }

    #[test]
    fn offset_matches_discriminant() {
        assert_eq!(Direction::North.offset(), 8);
        assert_eq!(Direction::NorthWest.offset(), 7);
        assert_eq!(Direction::NorthEast.offset(), 9);
        assert_eq!(Direction::West.offset(), -1);
        assert_eq!(Direction::East.offset(), 1);
        assert_eq!(Direction::South.offset(), -8);
        assert_eq!(Direction::SouthWest.offset(), -9);
        assert_eq!(Direction::SouthEast.offset(), -7);
    }

    #[test]
    fn offset_returns_i32() {
        // Ensure the cast to i32 doesn't corrupt the value for any direction.
        for dir in ALL_DIRS {
            let o = dir.offset();
            assert!(o >= i8::MIN as i32 && o <= i8::MAX as i32);
        }
    }

    #[test]
    fn offset_values_are_unique() {
        let offsets: Vec<i32> = ALL_DIRS.iter().map(|d| d.offset()).collect();
        let unique: std::collections::HashSet<i32> = offsets.iter().copied().collect();
        assert_eq!(
            offsets.len(),
            unique.len(),
            "all offsets should be distinct"
        );
    }

    #[test]
    fn opposite_directions_sum_to_zero() {
        let pairs = [
            (Direction::North, Direction::South),
            (Direction::NorthWest, Direction::SouthEast),
            (Direction::NorthEast, Direction::SouthWest),
            (Direction::West, Direction::East),
        ];
        for (a, b) in pairs {
            assert_eq!(a.offset() + b.offset(), 0, "{a:?} + {b:?} should be 0");
        }
    }

    #[test]
    fn invert_each_direction() {
        assert_eq!(Direction::North.invert(), Direction::South);
        assert_eq!(Direction::NorthWest.invert(), Direction::SouthEast);
        assert_eq!(Direction::NorthEast.invert(), Direction::SouthWest);
        assert_eq!(Direction::West.invert(), Direction::East);
        assert_eq!(Direction::East.invert(), Direction::West);
        assert_eq!(Direction::South.invert(), Direction::North);
        assert_eq!(Direction::SouthWest.invert(), Direction::NorthEast);
        assert_eq!(Direction::SouthEast.invert(), Direction::NorthWest);
    }

    #[test]
    fn invert_is_involution() {
        // Inverting twice should return the original direction.
        for dir in ALL_DIRS {
            assert_eq!(
                dir.invert().invert(),
                dir,
                "double-invert of {dir:?} should be identity"
            );
        }
    }

    #[test]
    fn invert_offsets_sum_to_zero() {
        for dir in ALL_DIRS {
            assert_eq!(
                dir.offset() + dir.invert().offset(),
                0,
                "offset({dir:?}) + offset(invert({dir:?})) should be 0"
            );
        }
    }

    #[test]
    fn invert_produces_distinct_direction() {
        // No direction should be its own inverse.
        for dir in ALL_DIRS {
            assert_ne!(dir, dir.invert(), "{dir:?} should not invert to itself");
        }
    }

    #[test]
    fn invert_covers_all_eight_directions() {
        // The set of inverted directions should equal the set of all directions.
        let inverted: std::collections::HashSet<i32> =
            ALL_DIRS.iter().map(|d| d.invert().offset()).collect();
        let original: std::collections::HashSet<i32> =
            ALL_DIRS.iter().map(|d| d.offset()).collect();
        assert_eq!(inverted, original);
    }

    #[test]
    fn is_anti_positive_offset_directions() {
        // Directions with positive offsets: North(8), NorthWest(7), NorthEast(9), East(1)
        assert!(Direction::North.is_anti());
        assert!(Direction::NorthWest.is_anti());
        assert!(Direction::NorthEast.is_anti());
        assert!(Direction::East.is_anti());
    }

    #[test]
    fn is_anti_negative_offset_directions() {
        // Directions with negative offsets should return false.
        assert!(!Direction::South.is_anti());
        assert!(!Direction::SouthWest.is_anti());
        assert!(!Direction::SouthEast.is_anti());
        assert!(!Direction::West.is_anti());
    }

    #[test]
    fn is_anti_matches_positive_offset_signum() {
        for dir in ALL_DIRS {
            let expected = dir.offset().signum() == 1;
            assert_eq!(dir.is_anti(), expected, "is_anti mismatch for {dir:?}");
        }
    }

    #[test]
    fn is_anti_and_invert_are_opposite() {
        // Inverting a direction should always flip is_anti.
        for dir in ALL_DIRS {
            assert_ne!(
                dir.is_anti(),
                dir.invert().is_anti(),
                "is_anti({dir:?}) and is_anti(invert({dir:?})) should differ"
            );
        }
    }

    #[test]
    fn exactly_four_anti_directions() {
        let anti_count = ALL_DIRS.iter().filter(|d| d.is_anti()).count();
        assert_eq!(anti_count, 4);
    }

    #[test]
    fn exactly_four_non_anti_directions() {
        let non_anti_count = ALL_DIRS.iter().filter(|d| !d.is_anti()).count();
        assert_eq!(non_anti_count, 4);
    }
}
