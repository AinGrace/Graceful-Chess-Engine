use core::fmt;
use std::fmt::Display;

use crate::color::Color;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Castlings(u8);

impl Castlings {
    pub fn new() -> Self {
        let mut rights = 0;
        rights |= 1 << 0;
        rights |= 1 << 1;
        rights |= 1 << 2;
        rights |= 1 << 3;

        Self(rights)
    }

    pub fn new_empty() -> Self {
        Self(0)
    }

    pub fn from_str(raw: &str) -> Option<Self> {
        if raw.len() > 4 {
            return None;
        }

        let mut rights = 0;
        for chr in raw.chars() {
            match chr {
                'K' => rights |= 1 << 0,
                'Q' => rights |= 1 << 1,
                'k' => rights |= 1 << 2,
                'q' => rights |= 1 << 3,
                _illegal => return None,
            }
        }

        Some(Self(rights))
    }

    pub fn w_short(&self) -> bool {
        self.0 & 1 << 0 != 0
    }

    pub fn w_long(&self) -> bool {
        self.0 & 1 << 1 != 0
    }

    pub fn b_short(&self) -> bool {
        self.0 & 1 << 2 != 0
    }

    pub fn b_long(&self) -> bool {
        self.0 & 1 << 3 != 0
    }

    pub fn short(&self, side: Color) -> bool {
        match side {
            Color::White => self.w_short(),
            Color::Black => self.b_short(),
        }
    }

    pub fn long(&self, side: Color) -> bool {
        match side {
            Color::White => self.w_long(),
            Color::Black => self.b_long(),
        }
    }

    pub fn remove_w_short(&mut self) {
        self.0 &= !(1 << 0);
    }

    pub fn remove_w_long(&mut self) {
        self.0 &= !(1 << 1);
    }

    pub fn remove_b_short(&mut self) {
        self.0 &= !(1 << 2);
    }

    pub fn remove_b_long(&mut self) {
        self.0 &= !(1 << 3);
    }

    pub fn remove_short(&mut self, side: Color) {
        match side {
            Color::White => self.remove_w_short(),
            Color::Black => self.remove_b_short(),
        }
    }

    pub fn remove_long(&mut self, side: Color) {
        match side {
            Color::White => self.remove_w_long(),
            Color::Black => self.remove_b_long(),
        }
    }

    pub fn remove_all(&mut self) {
        self.0 = 0;
    }

    pub fn remove_all_of(&mut self, side: Color) {
        match side {
            Color::White => {
                self.remove_w_short();
                self.remove_w_long();
            }
            Color::Black => {
                self.remove_b_short();
                self.remove_b_long();
            }
        }
    }

    pub fn remove_white(&mut self) {
        self.remove_w_short();
        self.remove_w_long();
    }

    pub fn remove_black(&mut self) {
        self.remove_b_short();
        self.remove_b_long();
    }

    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

impl Default for Castlings {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Castlings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}

impl Display for Castlings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.w_short() && !self.w_long() && !self.b_short() && !self.b_long() {
            return write!(f, "-");
        }
        if self.w_short() {
            write!(f, "K")?;
        }

        if self.w_long() {
            write!(f, "Q")?;
        }

        if self.b_short() {
            write!(f, "k")?;
        }

        if self.b_long() {
            write!(f, "q")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod castlings_tests {
    use super::*;
    use crate::color::Color;

    // =========================================================================
    // Construction
    // =========================================================================

    #[test]
    fn new_grants_all_four_rights() {
        let c = Castlings::new();
        assert!(c.w_short());
        assert!(c.w_long());
        assert!(c.b_short());
        assert!(c.b_long());
    }

    #[test]
    fn new_as_usize_is_0b1111() {
        assert_eq!(Castlings::new().as_usize(), 0b1111);
    }

    #[test]
    fn new_empty_grants_no_rights() {
        let c = Castlings::new_empty();
        assert!(!c.w_short());
        assert!(!c.w_long());
        assert!(!c.b_short());
        assert!(!c.b_long());
    }

    #[test]
    fn new_empty_as_usize_is_zero() {
        assert_eq!(Castlings::new_empty().as_usize(), 0);
    }

    #[test]
    fn default_equals_new() {
        let d = Castlings::default();
        assert!(d.w_short());
        assert!(d.w_long());
        assert!(d.b_short());
        assert!(d.b_long());
        assert_eq!(d.as_usize(), Castlings::new().as_usize());
    }

    #[test]
    fn from_str_full_rights() {
        let c = Castlings::from_str("KQkq").unwrap();
        assert!(c.w_short());
        assert!(c.w_long());
        assert!(c.b_short());
        assert!(c.b_long());
    }

    #[test]
    fn from_str_full_rights_matches_new() {
        assert_eq!(
            Castlings::from_str("KQkq").unwrap().as_usize(),
            Castlings::new().as_usize()
        );
    }

    #[test]
    fn from_str_empty_string_grants_no_rights() {
        let c = Castlings::from_str("").unwrap();
        assert!(!c.w_short());
        assert!(!c.w_long());
        assert!(!c.b_short());
        assert!(!c.b_long());
    }

    #[test]
    fn from_str_single_chars() {
        let c = Castlings::from_str("K").unwrap();
        assert!(c.w_short());
        assert!(!c.w_long());
        assert!(!c.b_short());
        assert!(!c.b_long());

        let c = Castlings::from_str("Q").unwrap();
        assert!(!c.w_short());
        assert!(c.w_long());
        assert!(!c.b_short());
        assert!(!c.b_long());

        let c = Castlings::from_str("k").unwrap();
        assert!(!c.w_short());
        assert!(!c.w_long());
        assert!(c.b_short());
        assert!(!c.b_long());

        let c = Castlings::from_str("q").unwrap();
        assert!(!c.w_short());
        assert!(!c.w_long());
        assert!(!c.b_short());
        assert!(c.b_long());
    }

    #[test]
    fn from_str_white_only() {
        let c = Castlings::from_str("KQ").unwrap();
        assert!(c.w_short());
        assert!(c.w_long());
        assert!(!c.b_short());
        assert!(!c.b_long());
    }

    #[test]
    fn from_str_black_only() {
        let c = Castlings::from_str("kq").unwrap();
        assert!(!c.w_short());
        assert!(!c.w_long());
        assert!(c.b_short());
        assert!(c.b_long());
    }

    #[test]
    fn from_str_short_only() {
        let c = Castlings::from_str("Kk").unwrap();
        assert!(c.w_short());
        assert!(!c.w_long());
        assert!(c.b_short());
        assert!(!c.b_long());
    }

    #[test]
    fn from_str_long_only() {
        let c = Castlings::from_str("Qq").unwrap();
        assert!(!c.w_short());
        assert!(c.w_long());
        assert!(!c.b_short());
        assert!(c.b_long());
    }

    #[test]
    fn from_str_duplicate_chars_are_idempotent() {
        let c = Castlings::from_str("KK").unwrap();
        assert_eq!(c.as_usize(), Castlings::from_str("K").unwrap().as_usize());
    }

    #[test]
    fn from_str_too_long_is_none() {
        assert!(Castlings::from_str("KQkqK").is_none());
        assert!(Castlings::from_str("KQkqq").is_none());
        assert!(Castlings::from_str("KKKKK").is_none());
    }

    #[test]
    fn from_str_illegal_char_is_none() {
        assert!(Castlings::from_str("X").is_none());
        assert!(Castlings::from_str("KQx").is_none());
        assert!(Castlings::from_str("-").is_none());
        assert!(Castlings::from_str("1").is_none());
        assert!(Castlings::from_str(" ").is_none());
    }

    #[test]
    fn short_delegates_to_w_short_and_b_short() {
        let c = Castlings::new();
        assert_eq!(c.short(Color::White), c.w_short());
        assert_eq!(c.short(Color::Black), c.b_short());
    }

    #[test]
    fn long_delegates_to_w_long_and_b_long() {
        let c = Castlings::new();
        assert_eq!(c.long(Color::White), c.w_long());
        assert_eq!(c.long(Color::Black), c.b_long());
    }

    #[test]
    fn short_and_long_after_partial_removal() {
        let mut c = Castlings::new();
        c.remove_w_short();
        assert!(!c.short(Color::White));
        assert!(c.short(Color::Black));

        c.remove_b_long();
        assert!(!c.long(Color::Black));
        assert!(c.long(Color::White));
    }

    #[test]
    fn remove_w_short_only_clears_w_short() {
        let mut c = Castlings::new();
        c.remove_w_short();
        assert!(!c.w_short());
        assert!(c.w_long());
        assert!(c.b_short());
        assert!(c.b_long());
    }

    #[test]
    fn remove_w_long_only_clears_w_long() {
        let mut c = Castlings::new();
        c.remove_w_long();
        assert!(c.w_short());
        assert!(!c.w_long());
        assert!(c.b_short());
        assert!(c.b_long());
    }

    #[test]
    fn remove_b_short_only_clears_b_short() {
        let mut c = Castlings::new();
        c.remove_b_short();
        assert!(c.w_short());
        assert!(c.w_long());
        assert!(!c.b_short());
        assert!(c.b_long());
    }

    #[test]
    fn remove_b_long_only_clears_b_long() {
        let mut c = Castlings::new();
        c.remove_b_long();
        assert!(c.w_short());
        assert!(c.w_long());
        assert!(c.b_short());
        assert!(!c.b_long());
    }

    #[test]
    fn remove_is_idempotent() {
        let mut c = Castlings::new();
        c.remove_w_short();
        c.remove_w_short(); // second call should be a no-op
        assert!(!c.w_short());
        assert_eq!(c.as_usize(), Castlings::from_str("Qkq").unwrap().as_usize());
    }

    #[test]
    fn remove_short_white_clears_w_short() {
        let mut c = Castlings::new();
        c.remove_short(Color::White);
        assert!(!c.w_short());
        assert!(c.w_long());
        assert!(c.b_short());
        assert!(c.b_long());
    }

    #[test]
    fn remove_short_black_clears_b_short() {
        let mut c = Castlings::new();
        c.remove_short(Color::Black);
        assert!(c.w_short());
        assert!(c.w_long());
        assert!(!c.b_short());
        assert!(c.b_long());
    }

    #[test]
    fn remove_long_white_clears_w_long() {
        let mut c = Castlings::new();
        c.remove_long(Color::White);
        assert!(c.w_short());
        assert!(!c.w_long());
        assert!(c.b_short());
        assert!(c.b_long());
    }

    #[test]
    fn remove_long_black_clears_b_long() {
        let mut c = Castlings::new();
        c.remove_long(Color::Black);
        assert!(c.w_short());
        assert!(c.w_long());
        assert!(c.b_short());
        assert!(!c.b_long());
    }

    #[test]
    fn remove_white_clears_both_white_rights() {
        let mut c = Castlings::new();
        c.remove_white();
        assert!(!c.w_short());
        assert!(!c.w_long());
        assert!(c.b_short());
        assert!(c.b_long());
    }

    #[test]
    fn remove_black_clears_both_black_rights() {
        let mut c = Castlings::new();
        c.remove_black();
        assert!(c.w_short());
        assert!(c.w_long());
        assert!(!c.b_short());
        assert!(!c.b_long());
    }

    #[test]
    fn remove_all_of_white_matches_remove_white() {
        let mut c1 = Castlings::new();
        let mut c2 = Castlings::new();
        c1.remove_all_of(Color::White);
        c2.remove_white();
        assert_eq!(c1.as_usize(), c2.as_usize());
    }

    #[test]
    fn remove_all_of_black_matches_remove_black() {
        let mut c1 = Castlings::new();
        let mut c2 = Castlings::new();
        c1.remove_all_of(Color::Black);
        c2.remove_black();
        assert_eq!(c1.as_usize(), c2.as_usize());
    }

    #[test]
    fn remove_white_then_black_equals_remove_all() {
        let mut c1 = Castlings::new();
        c1.remove_white();
        c1.remove_black();

        let mut c2 = Castlings::new();
        c2.remove_all();

        assert_eq!(c1.as_usize(), c2.as_usize());
    }

    #[test]
    fn remove_all_clears_all_rights() {
        let mut c = Castlings::new();
        c.remove_all();
        assert!(!c.w_short());
        assert!(!c.w_long());
        assert!(!c.b_short());
        assert!(!c.b_long());
        assert_eq!(c.as_usize(), 0);
    }

    #[test]
    fn remove_all_on_empty_is_noop() {
        let mut c = Castlings::new_empty();
        c.remove_all();
        assert_eq!(c.as_usize(), 0);
    }

    #[test]
    fn as_usize_bit_layout() {
        // bit 0 = w_short, bit 1 = w_long, bit 2 = b_short, bit 3 = b_long
        assert_eq!(Castlings::from_str("K").unwrap().as_usize(), 0b0001);
        assert_eq!(Castlings::from_str("Q").unwrap().as_usize(), 0b0010);
        assert_eq!(Castlings::from_str("k").unwrap().as_usize(), 0b0100);
        assert_eq!(Castlings::from_str("q").unwrap().as_usize(), 0b1000);
        assert_eq!(Castlings::from_str("KQ").unwrap().as_usize(), 0b0011);
        assert_eq!(Castlings::from_str("kq").unwrap().as_usize(), 0b1100);
        assert_eq!(Castlings::from_str("KQkq").unwrap().as_usize(), 0b1111);
    }

    // =========================================================================
    // Display / Debug
    // =========================================================================

    #[test]
    fn display_full_rights_is_kqkq() {
        assert_eq!(Castlings::new().to_string(), "KQkq");
    }

    #[test]
    fn display_empty_rights_is_dash() {
        assert_eq!(Castlings::new_empty().to_string(), "-");
    }

    #[test]
    fn display_white_only() {
        let c = Castlings::from_str("KQ").unwrap();
        assert_eq!(c.to_string(), "KQ");
    }

    #[test]
    fn display_black_only() {
        let c = Castlings::from_str("kq").unwrap();
        assert_eq!(c.to_string(), "kq");
    }

    #[test]
    fn display_single_rights() {
        assert_eq!(Castlings::from_str("K").unwrap().to_string(), "K");
        assert_eq!(Castlings::from_str("Q").unwrap().to_string(), "Q");
        assert_eq!(Castlings::from_str("k").unwrap().to_string(), "k");
        assert_eq!(Castlings::from_str("q").unwrap().to_string(), "q");
    }

    #[test]
    fn display_order_is_always_kqkq() {
        // Regardless of parse order, display is always K then Q then k then q
        let c = Castlings::from_str("qkQK").unwrap();
        assert_eq!(c.to_string(), "KQkq");
    }
}
