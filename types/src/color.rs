use std::ops::Not;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn from_char(chr: char) -> Option<Self> {
        match chr {
            'w' | 'W' => Some(Self::White),
            'b' | 'B' => Some(Self::Black),
            _unknown => None,
        }
    }

    pub fn char(self) -> char {
        match self {
            Color::White => 'w',
            Color::Black => 'b',
        }
    }
}

impl Not for Color {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[cfg(test)]
mod color_tests {
    use super::*;

    const ALL_COLORS: [Color; 2] = [Color::White, Color::Black];

    #[test]
    fn new_lowercase_w_is_white() {
        assert_eq!(Color::from_char('w'), Some(Color::White));
    }

    #[test]
    fn new_uppercase_w_is_white() {
        assert_eq!(Color::from_char('W'), Some(Color::White));
    }

    #[test]
    fn new_lowercase_b_is_black() {
        assert_eq!(Color::from_char('b'), Some(Color::Black));
    }

    #[test]
    fn new_uppercase_b_is_black() {
        assert_eq!(Color::from_char('B'), Some(Color::Black));
    }

    #[test]
    fn new_upper_and_lower_produce_same_variant() {
        assert_eq!(Color::from_char('w'), Color::from_char('W'));
        assert_eq!(Color::from_char('b'), Color::from_char('B'));
    }

    #[test]
    fn new_other_letters_are_none() {
        for c in [
            'a', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
            's', 't', 'u', 'v', 'x', 'y', 'z', 'A', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K',
            'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'X', 'Y', 'Z',
        ] {
            assert_eq!(Color::from_char(c), None, "'{c}' should return None");
        }
    }

    #[test]
    fn new_digits_are_none() {
        for c in '0'..='9' {
            assert_eq!(Color::from_char(c), None, "digit '{c}' should return None");
        }
    }

    #[test]
    fn new_special_chars_are_none() {
        for c in [' ', '\n', '\t', '-', '_', '!', '/', '.', ',', '#'] {
            assert_eq!(Color::from_char(c), None, "'{c}' should return None");
        }
    }

    #[test]
    fn char_white_is_lowercase_w() {
        assert_eq!(Color::White.char(), 'w');
    }

    #[test]
    fn char_black_is_lowercase_b() {
        assert_eq!(Color::Black.char(), 'b');
    }

    #[test]
    fn char_is_always_lowercase() {
        for color in ALL_COLORS {
            let c = color.char();
            assert!(
                c.is_lowercase(),
                "char() for {color:?} returned '{c}', expected lowercase"
            );
        }
    }

    #[test]
    fn char_values_are_distinct() {
        assert_ne!(Color::White.char(), Color::Black.char());
    }

    #[test]
    fn char_then_new_round_trips() {
        for color in ALL_COLORS {
            let c = color.char();
            assert_eq!(
                Color::from_char(c),
                Some(color),
                "round-trip failed for {color:?}"
            );
        }
    }

    #[test]
    fn new_lowercase_then_char_is_identity() {
        for c in ['w', 'b'] {
            let color = Color::from_char(c).unwrap();
            assert_eq!(color.char(), c);
        }
    }

    #[test]
    fn new_uppercase_then_char_is_lowercase() {
        assert_eq!(Color::from_char('W').unwrap().char(), 'w');
        assert_eq!(Color::from_char('B').unwrap().char(), 'b');
    }

    #[test]
    fn not_white_is_black() {
        assert_eq!(!Color::White, Color::Black);
    }

    #[test]
    fn not_black_is_white() {
        assert_eq!(!Color::Black, Color::White);
    }

    #[test]
    fn not_is_involution() {
        for color in ALL_COLORS {
            assert_eq!(!!color, color, "double-not of {color:?} should be identity");
        }
    }

    #[test]
    fn not_produces_distinct_color() {
        for color in ALL_COLORS {
            assert_ne!(!color, color, "{color:?} should not be its own inverse");
        }
    }

    #[test]
    fn not_covers_both_colors() {
        let inverted: std::collections::HashSet<String> =
            ALL_COLORS.iter().map(|c| format!("{:?}", !*c)).collect();
        let original: std::collections::HashSet<String> =
            ALL_COLORS.iter().map(|c| format!("{c:?}")).collect();
        assert_eq!(inverted, original);
    }
}
