use core::fmt;
use std::fmt::Display;

use crate::color::Color;

#[derive(Clone)]
pub struct CastlingRights(u8);

impl CastlingRights {
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
}

impl Default for CastlingRights {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for CastlingRights {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}

impl Display for CastlingRights {
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
mod tests {
    use super::*;

    #[test]
    fn castling_rights_removal_test() {
        let mut rights = CastlingRights::new();
        dbg!(&rights);

        rights.remove_w_short();
        dbg!(&rights);

        rights.remove_w_short();
        dbg!(&rights);

        rights.remove_white();
        dbg!(&rights);
    }
}
