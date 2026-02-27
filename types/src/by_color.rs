use crate::{bitboard::Bitboard, color::Color, square::Square};

#[derive(Clone, Copy)]
pub struct ByColor {
    white: Bitboard,
    black: Bitboard,
}

impl ByColor {
    pub fn new(white: Bitboard, black: Bitboard) -> Self {
        Self { white, black }
    }

    pub const fn get(&self, color: Color) -> Bitboard {
        match color {
            Color::White => self.white,
            Color::Black => self.black,
        }
    }

    pub fn get_mut(&mut self, color: Color) -> &mut Bitboard {
        match color {
            Color::White => &mut self.white,
            Color::Black => &mut self.black,
        }
    }

    pub fn whites(&self) -> Bitboard {
        self.white
    }

    pub fn whites_mut(&mut self) -> &mut Bitboard {
        &mut self.white
    }

    pub fn blacks(&self) -> Bitboard {
        self.black
    }

    pub fn blacks_mut(&mut self) -> &mut Bitboard {
        &mut self.black
    }

    pub fn peek_color(&self, square: Square) -> Option<Color> {
        if self.white.is_square_set(square) {
            return Some(Color::White);
        }

        if self.black.is_square_set(square) {
            return Some(Color::Black);
        }

        None
    }

    pub fn peek_color_checked(&self, square: Square) -> Color {
        if self.whites().is_square_set(square) {
            Color::White
        } else {
            Color::Black
        }
    }
}
