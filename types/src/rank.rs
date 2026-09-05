use std::mem::transmute;

#[rustfmt::skip]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Rank {
    First   = 0,
    Second  = 1,
    Third   = 2,
    Fourth  = 3,
    Fifth   = 4,
    Sixth   = 5,
    Seventh = 6,
    Eighth  = 7,
}

impl Rank {
    pub const fn to_u32(self) -> u32 {
        self as u32
    }

    pub const fn to_i32(self) -> i32 {
        self as i32
    }

    pub fn from_char(chr: char) -> Option<Self> {
        match chr {
            '1' => Some(Rank::First),
            '2' => Some(Rank::Second),
            '3' => Some(Rank::Third),
            '4' => Some(Rank::Fourth),
            '5' => Some(Rank::Fifth),
            '6' => Some(Rank::Sixth),
            '7' => Some(Rank::Seventh),
            '8' => Some(Rank::Eighth),
            _rest => None,
        }
    }

    pub fn char(self) -> char {
        match self {
            Rank::First => '1',
            Rank::Second => '2',
            Rank::Third => '3',
            Rank::Fourth => '4',
            Rank::Fifth => '5',
            Rank::Sixth => '6',
            Rank::Seventh => '7',
            Rank::Eighth => '8',
        }
    }

    pub const fn from_u32(index: u32) -> Option<Self> {
        if index > 7 {
            None
        } else {
            Some(Self::from_u32_checked(index))
        }
    }

    /// Will panic if index is >= 8
    pub const fn from_u32_checked(index: u32) -> Self {
        assert!(index < 8);

        // SAFETY: index is always at valid range
        unsafe { transmute(index as u8) }
    }
}
#[cfg(test)]
mod rank_tests {
    use super::*;

    #[test]
    fn discriminants_are_correct() {
        assert_eq!(Rank::First as u8, 0);
        assert_eq!(Rank::Second as u8, 1);
        assert_eq!(Rank::Third as u8, 2);
        assert_eq!(Rank::Fourth as u8, 3);
        assert_eq!(Rank::Fifth as u8, 4);
        assert_eq!(Rank::Sixth as u8, 5);
        assert_eq!(Rank::Seventh as u8, 6);
        assert_eq!(Rank::Eighth as u8, 7);
    }

    #[test]
    fn to_u32_matches_discriminant() {
        assert_eq!(Rank::First.to_u32(), 0);
        assert_eq!(Rank::Second.to_u32(), 1);
        assert_eq!(Rank::Third.to_u32(), 2);
        assert_eq!(Rank::Fourth.to_u32(), 3);
        assert_eq!(Rank::Fifth.to_u32(), 4);
        assert_eq!(Rank::Sixth.to_u32(), 5);
        assert_eq!(Rank::Seventh.to_u32(), 6);
        assert_eq!(Rank::Eighth.to_u32(), 7);
    }

    #[test]
    fn to_u32_round_trips_from_u32_checked() {
        for i in 0u32..8 {
            let r = Rank::from_u32_checked(i);
            assert_eq!(r.to_u32(), i);
        }
    }

    #[test]
    fn from_u32_zero_is_first() {
        assert_eq!(Rank::from_u32(0), Some(Rank::First));
    }

    #[test]
    fn from_u32_seven_is_eighth() {
        assert_eq!(Rank::from_u32(7), Some(Rank::Eighth));
    }

    #[test]
    fn from_u32_all_valid_indices() {
        let expected = [
            Rank::First,
            Rank::Second,
            Rank::Third,
            Rank::Fourth,
            Rank::Fifth,
            Rank::Sixth,
            Rank::Seventh,
            Rank::Eighth,
        ];
        for (i, &exp) in expected.iter().enumerate() {
            assert_eq!(
                Rank::from_u32(i as u32),
                Some(exp),
                "index {i} should map to {exp:?}"
            );
        }
    }

    #[test]
    fn from_u32_round_trips_to_u32() {
        for i in 0u32..8 {
            let r = Rank::from_u32(i).unwrap();
            assert_eq!(r.to_u32(), i);
        }
    }

    #[test]
    fn from_u32_8_is_none() {
        assert_eq!(Rank::from_u32(8), None);
    }

    #[test]
    fn from_u32_large_values_are_none() {
        assert_eq!(Rank::from_u32(9), None);
        assert_eq!(Rank::from_u32(255), None);
        assert_eq!(Rank::from_u32(u32::MAX), None);
    }

    #[test]
    fn from_u32_checked_boundaries() {
        assert_eq!(Rank::from_u32_checked(0), Rank::First);
        assert_eq!(Rank::from_u32_checked(7), Rank::Eighth);
    }

    #[test]
    fn from_u32_checked_all_valid() {
        for i in 0u32..8 {
            let r = Rank::from_u32_checked(i);
            assert_eq!(r.to_u32(), i);
        }
    }

    #[test]
    fn from_u32_checked_matches_from_u32() {
        for i in 0u32..8 {
            assert_eq!(Rank::from_u32_checked(i), Rank::from_u32(i).unwrap());
        }
    }

    #[test]
    #[should_panic]
    fn from_u32_checked_8_panics() {
        let _ = Rank::from_u32_checked(8);
    }

    #[test]
    #[should_panic]
    fn from_u32_checked_large_panics() {
        let _ = Rank::from_u32_checked(100);
    }

    #[test]
    fn from_char_valid_digits() {
        assert_eq!(Rank::from_char('1'), Some(Rank::First));
        assert_eq!(Rank::from_char('2'), Some(Rank::Second));
        assert_eq!(Rank::from_char('3'), Some(Rank::Third));
        assert_eq!(Rank::from_char('4'), Some(Rank::Fourth));
        assert_eq!(Rank::from_char('5'), Some(Rank::Fifth));
        assert_eq!(Rank::from_char('6'), Some(Rank::Sixth));
        assert_eq!(Rank::from_char('7'), Some(Rank::Seventh));
        assert_eq!(Rank::from_char('8'), Some(Rank::Eighth));
    }

    #[test]
    fn from_char_zero_is_none() {
        assert_eq!(Rank::from_char('0'), None);
    }

    #[test]
    fn from_char_nine_is_none() {
        assert_eq!(Rank::from_char('9'), None);
    }

    #[test]
    fn from_char_letters_are_none() {
        for c in ['a', 'b', 'h', 'A', 'Z', 'r'] {
            assert_eq!(Rank::from_char(c), None, "'{c}' should return None");
        }
    }

    #[test]
    fn from_char_special_chars_are_none() {
        for c in [' ', '\n', '\t', '-', '_', '!', '/'] {
            assert_eq!(Rank::from_char(c), None, "'{c}' should return None");
        }
    }

    #[test]
    fn char_returns_correct_digit() {
        assert_eq!(Rank::First.char(), '1');
        assert_eq!(Rank::Second.char(), '2');
        assert_eq!(Rank::Third.char(), '3');
        assert_eq!(Rank::Fourth.char(), '4');
        assert_eq!(Rank::Fifth.char(), '5');
        assert_eq!(Rank::Sixth.char(), '6');
        assert_eq!(Rank::Seventh.char(), '7');
        assert_eq!(Rank::Eighth.char(), '8');
    }

    #[test]
    fn char_is_always_ascii_digit() {
        let all = [
            Rank::First,
            Rank::Second,
            Rank::Third,
            Rank::Fourth,
            Rank::Fifth,
            Rank::Sixth,
            Rank::Seventh,
            Rank::Eighth,
        ];
        for r in all {
            let c = r.char();
            assert!(
                c.is_ascii_digit(),
                "char() for {r:?} returned '{c}', expected digit"
            );
        }
    }

    #[test]
    fn char_digit_value_equals_one_plus_index() {
        for i in 0u32..8 {
            let r = Rank::from_u32_checked(i);
            let digit = r.char().to_digit(10).unwrap();
            assert_eq!(
                digit,
                i + 1,
                "char digit for rank index {i} should be {}",
                i + 1
            );
        }
    }

    #[test]
    fn char_then_from_char_round_trips() {
        let all = [
            Rank::First,
            Rank::Second,
            Rank::Third,
            Rank::Fourth,
            Rank::Fifth,
            Rank::Sixth,
            Rank::Seventh,
            Rank::Eighth,
        ];
        for r in all {
            let c = r.char();
            assert_eq!(Rank::from_char(c), Some(r), "round-trip failed for {r:?}");
        }
    }

    #[test]
    fn from_char_then_char_is_identity() {
        for c in '1'..='8' {
            let r = Rank::from_char(c).unwrap();
            assert_eq!(r.char(), c, "round-trip failed for '{c}'");
        }
    }

    #[test]
    fn ordering_first_less_than_eighth() {
        assert!(Rank::First < Rank::Eighth);
    }

    #[test]
    fn ordering_is_sequential() {
        let all = [
            Rank::First,
            Rank::Second,
            Rank::Third,
            Rank::Fourth,
            Rank::Fifth,
            Rank::Sixth,
            Rank::Seventh,
            Rank::Eighth,
        ];
        for window in all.windows(2) {
            assert!(
                window[0] < window[1],
                "{:?} should be less than {:?}",
                window[0],
                window[1]
            );
        }
    }

    #[test]
    fn ordering_matches_index() {
        let all = [
            Rank::First,
            Rank::Second,
            Rank::Third,
            Rank::Fourth,
            Rank::Fifth,
            Rank::Sixth,
            Rank::Seventh,
            Rank::Eighth,
        ];
        for i in 0..all.len() {
            for j in 0..all.len() {
                assert_eq!(
                    all[i].cmp(&all[j]),
                    i.cmp(&j),
                    "ordering mismatch between {:?} and {:?}",
                    all[i],
                    all[j]
                );
            }
        }
    }

    #[test]
    fn min_is_first_max_is_eighth() {
        let all = [
            Rank::First,
            Rank::Second,
            Rank::Third,
            Rank::Fourth,
            Rank::Fifth,
            Rank::Sixth,
            Rank::Seventh,
            Rank::Eighth,
        ];
        assert_eq!(all.iter().copied().min(), Some(Rank::First));
        assert_eq!(all.iter().copied().max(), Some(Rank::Eighth));
    }
}
