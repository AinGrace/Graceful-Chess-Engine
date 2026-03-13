use std::mem::transmute;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum File {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    F = 5,
    G = 6,
    H = 7,
}

impl File {
    pub const fn to_usize(self) -> usize {
        self as usize
    }

    pub const fn to_u32(self) -> u32 {
        self as u32
    }

    pub fn from_char(chr: char) -> Option<Self> {
        match chr {
            'A' | 'a' => Some(Self::A),
            'B' | 'b' => Some(Self::B),
            'C' | 'c' => Some(Self::C),
            'D' | 'd' => Some(Self::D),
            'E' | 'e' => Some(Self::E),
            'F' | 'f' => Some(Self::F),
            'G' | 'g' => Some(Self::G),
            'H' | 'h' => Some(Self::H),
            _ => None,
        }
    }

    pub fn char(self) -> char {
        match self {
            File::A => 'a',
            File::B => 'b',
            File::C => 'c',
            File::D => 'd',
            File::E => 'e',
            File::F => 'f',
            File::G => 'g',
            File::H => 'h',
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
mod file_tests {
    use super::*;

    #[test]
    fn discriminants_are_correct() {
        assert_eq!(File::A as u8, 0);
        assert_eq!(File::B as u8, 1);
        assert_eq!(File::C as u8, 2);
        assert_eq!(File::D as u8, 3);
        assert_eq!(File::E as u8, 4);
        assert_eq!(File::F as u8, 5);
        assert_eq!(File::G as u8, 6);
        assert_eq!(File::H as u8, 7);
    }

    #[test]
    fn to_u32_matches_discriminant() {
        assert_eq!(File::A.to_u32(), 0);
        assert_eq!(File::B.to_u32(), 1);
        assert_eq!(File::C.to_u32(), 2);
        assert_eq!(File::D.to_u32(), 3);
        assert_eq!(File::E.to_u32(), 4);
        assert_eq!(File::F.to_u32(), 5);
        assert_eq!(File::G.to_u32(), 6);
        assert_eq!(File::H.to_u32(), 7);
    }

    #[test]
    fn to_usize_matches_discriminant() {
        assert_eq!(File::A.to_usize(), 0);
        assert_eq!(File::B.to_usize(), 1);
        assert_eq!(File::C.to_usize(), 2);
        assert_eq!(File::D.to_usize(), 3);
        assert_eq!(File::E.to_usize(), 4);
        assert_eq!(File::F.to_usize(), 5);
        assert_eq!(File::G.to_usize(), 6);
        assert_eq!(File::H.to_usize(), 7);
    }

    #[test]
    fn to_usize_matches_to_u32() {
        let all = [
            File::A,
            File::B,
            File::C,
            File::D,
            File::E,
            File::F,
            File::G,
            File::H,
        ];
        for f in all {
            assert_eq!(f.to_usize(), f.to_u32() as usize);
        }
    }

    #[test]
    fn from_u32_zero_is_file_a() {
        assert_eq!(File::from_u32(0), Some(File::A));
    }

    #[test]
    fn from_u32_seven_is_file_h() {
        assert_eq!(File::from_u32(7), Some(File::H));
    }

    #[test]
    fn from_u32_all_valid_indices() {
        let expected = [
            File::A,
            File::B,
            File::C,
            File::D,
            File::E,
            File::F,
            File::G,
            File::H,
        ];
        for (i, &exp) in expected.iter().enumerate() {
            assert_eq!(
                File::from_u32(i as u32),
                Some(exp),
                "index {i} should be Some({exp:?})"
            );
        }
    }

    #[test]
    fn from_u32_round_trips_to_u32() {
        for i in 0u32..8 {
            let f = File::from_u32(i).unwrap();
            assert_eq!(f.to_u32(), i);
        }
    }

    #[test]
    fn from_u32_8_is_none() {
        assert_eq!(File::from_u32(8), None);
    }

    #[test]
    fn from_32_large_values_are_none() {
        assert_eq!(File::from_u32(9), None);
        assert_eq!(File::from_u32(255), None);
        assert_eq!(File::from_u32(u32::MAX), None);
    }

    #[test]
    fn from_u32_checked_boundaries() {
        assert_eq!(File::from_u32_checked(0), File::A);
        assert_eq!(File::from_u32_checked(7), File::H);
    }

    #[test]
    fn from_u32_checked_all_valid() {
        for i in 0u32..8 {
            let f = File::from_u32_checked(i);
            assert_eq!(f.to_u32(), i);
        }
    }

    #[test]
    fn from_u32_checked_matches_new() {
        for i in 0u32..8 {
            assert_eq!(File::from_u32_checked(i), File::from_u32(i).unwrap());
        }
    }

    #[test]
    #[should_panic]
    fn from_u32_checked_8_panics() {
        let _ = File::from_u32_checked(8);
    }

    #[test]
    #[should_panic]
    fn from_u32_checked_large_panics() {
        let _ = File::from_u32_checked(100);
    }

    #[test]
    fn from_char_uppercase_all() {
        assert_eq!(File::from_char('A'), Some(File::A));
        assert_eq!(File::from_char('B'), Some(File::B));
        assert_eq!(File::from_char('C'), Some(File::C));
        assert_eq!(File::from_char('D'), Some(File::D));
        assert_eq!(File::from_char('E'), Some(File::E));
        assert_eq!(File::from_char('F'), Some(File::F));
        assert_eq!(File::from_char('G'), Some(File::G));
        assert_eq!(File::from_char('H'), Some(File::H));
    }

    #[test]
    fn from_char_lowercase_all() {
        assert_eq!(File::from_char('a'), Some(File::A));
        assert_eq!(File::from_char('b'), Some(File::B));
        assert_eq!(File::from_char('c'), Some(File::C));
        assert_eq!(File::from_char('d'), Some(File::D));
        assert_eq!(File::from_char('e'), Some(File::E));
        assert_eq!(File::from_char('f'), Some(File::F));
        assert_eq!(File::from_char('g'), Some(File::G));
        assert_eq!(File::from_char('h'), Some(File::H));
    }

    #[test]
    fn from_char_invalid_letters() {
        // Letters just outside the valid range
        assert_eq!(File::from_char('I'), None);
        assert_eq!(File::from_char('i'), None);
        assert_eq!(File::from_char('Z'), None);
        assert_eq!(File::from_char('z'), None);
    }

    #[test]
    fn from_char_digits_are_none() {
        for c in '0'..='9' {
            assert_eq!(File::from_char(c), None, "digit '{c}' should return None");
        }
    }

    #[test]
    fn from_char_special_chars_are_none() {
        for c in [' ', '\n', '\t', '-', '_', '!', '@', '#'] {
            assert_eq!(File::from_char(c), None, "char '{c}' should return None");
        }
    }

    #[test]
    fn char_returns_lowercase() {
        assert_eq!(File::A.char(), 'a');
        assert_eq!(File::B.char(), 'b');
        assert_eq!(File::C.char(), 'c');
        assert_eq!(File::D.char(), 'd');
        assert_eq!(File::E.char(), 'e');
        assert_eq!(File::F.char(), 'f');
        assert_eq!(File::G.char(), 'g');
        assert_eq!(File::H.char(), 'h');
    }

    #[test]
    fn char_is_always_lowercase() {
        let all = [
            File::A,
            File::B,
            File::C,
            File::D,
            File::E,
            File::F,
            File::G,
            File::H,
        ];
        for f in all {
            let c = f.char();
            assert!(
                c.is_lowercase(),
                "char() for {f:?} returned '{c}', expected lowercase"
            );
        }
    }

    #[test]
    fn char_then_from_char_round_trips() {
        let all = [
            File::A,
            File::B,
            File::C,
            File::D,
            File::E,
            File::F,
            File::G,
            File::H,
        ];
        for f in all {
            let c = f.char();
            assert_eq!(File::from_char(c), Some(f), "round-trip failed for {f:?}");
        }
    }

    #[test]
    fn from_char_lowercase_then_char_is_identity() {
        for c in 'a'..='h' {
            let f = File::from_char(c).unwrap();
            assert_eq!(f.char(), c);
        }
    }

    #[test]
    fn from_char_uppercase_then_char_is_lowercase() {
        for (upper, lower) in ('A'..='H').zip('a'..='h') {
            let f = File::from_char(upper).unwrap();
            assert_eq!(f.char(), lower);
        }
    }

    #[test]
    fn ordering_a_less_than_h() {
        assert!(File::A < File::H);
    }

    #[test]
    fn ordering_is_sequential() {
        let all = [
            File::A,
            File::B,
            File::C,
            File::D,
            File::E,
            File::F,
            File::G,
            File::H,
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
            File::A,
            File::B,
            File::C,
            File::D,
            File::E,
            File::F,
            File::G,
            File::H,
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
    fn min_is_a_max_is_h() {
        let all = [
            File::A,
            File::B,
            File::C,
            File::D,
            File::E,
            File::F,
            File::G,
            File::H,
        ];
        assert_eq!(all.iter().copied().min(), Some(File::A));
        assert_eq!(all.iter().copied().max(), Some(File::H));
    }
}
