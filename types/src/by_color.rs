use std::hint::unreachable_unchecked;

use crate::{bitboard::Bitboard, color::Color, square::Square};

// TODO: consider converthing this into array
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByColor<T> {
    white: T,
    black: T,
}

impl<T> ByColor<T> {
    pub fn new(white: T, black: T) -> Self {
        Self { white, black }
    }

    pub const fn get(&self, color: Color) -> &T {
        match color {
            Color::White => &self.white,
            Color::Black => &self.black,
        }
    }

    pub fn get_mut(&mut self, color: Color) -> &mut T {
        match color {
            Color::White => &mut self.white,
            Color::Black => &mut self.black,
        }
    }

    pub fn whites(&self) -> &T {
        &self.white
    }

    pub fn whites_mut(&mut self) -> &mut T {
        &mut self.white
    }

    pub fn blacks(&self) -> &T {
        &self.black
    }

    pub fn blacks_mut(&mut self) -> &mut T {
        &mut self.black
    }
}

/// Specialization methods if ByColor contains Bitboard
impl ByColor<Bitboard> {
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
        } else if self.blacks().is_square_set(square) {
            Color::Black
        } else {
            panic!("peek_color_checked on unset square")
        }
    }

    /// caller should guarantee that square corresponds to a bitboard with existing piece
    pub unsafe fn peek_color_unchecked(&self, square: Square) -> Color {
        if self.whites().is_square_set(square) {
            Color::White
        } else if self.blacks().is_square_set(square) {
            Color::Black
        } else {
            unsafe { unreachable_unchecked() }
        }
    }
}

#[cfg(test)]
mod by_color_tests {

    use crate::{
        bitboard::{Bitboard, ToBitboard},
        color::Color,
        square::Square,
    };

    type ByColor = super::ByColor<Bitboard>;

    fn fixture() -> ByColor {
        ByColor::new(Square::E1.to_bb(), Square::E8.to_bb())
    }

    fn empty() -> ByColor {
        ByColor::new(Bitboard::new_empty(), Bitboard::new_empty())
    }

    #[test]
    fn new_stores_white_and_black_independently() {
        let bc = fixture();
        assert_eq!(bc.whites().as_u64(), Square::E1.to_bb().as_u64());
        assert_eq!(bc.blacks().as_u64(), Square::E8.to_bb().as_u64());
    }

    #[test]
    fn new_empty_both_fields_zero() {
        let bc = empty();
        assert!(bc.whites().empty());
        assert!(bc.blacks().empty());
    }

    #[test]
    fn get_white_returns_white_bitboard() {
        let bc = fixture();
        assert_eq!(bc.get(Color::White).as_u64(), bc.whites().as_u64());
    }

    #[test]
    fn get_black_returns_black_bitboard() {
        let bc = fixture();
        assert_eq!(bc.get(Color::Black).as_u64(), bc.blacks().as_u64());
    }

    #[test]
    fn get_does_not_alias_colors() {
        let bc = fixture();
        // White square must not appear in black board and vice versa
        assert!(!bc.get(Color::Black).is_square_set(Square::E1));
        assert!(!bc.get(Color::White).is_square_set(Square::E8));
    }

    #[test]
    fn get_on_empty_is_empty() {
        let bc = empty();
        assert!(bc.get(Color::White).empty());
        assert!(bc.get(Color::Black).empty());
    }

    #[test]
    fn get_mut_white_modifies_only_white() {
        let mut bc = empty();
        *bc.get_mut(Color::White) = Square::H1.to_bb();
        assert!(bc.get(Color::White).is_square_set(Square::H1));
        assert!(bc.get(Color::Black).empty());
    }

    #[test]
    fn get_mut_black_modifies_only_black() {
        let mut bc = empty();
        *bc.get_mut(Color::Black) = Square::H8.to_bb();
        assert!(bc.get(Color::Black).is_square_set(Square::H8));
        assert!(bc.get(Color::White).empty());
    }

    #[test]
    fn get_mut_reflects_change_via_get() {
        let mut bc = empty();
        let new_bb = Square::D4.to_bb();
        *bc.get_mut(Color::White) = new_bb;
        assert_eq!(bc.get(Color::White).as_u64(), new_bb.as_u64());

        *bc.get_mut(Color::Black) = new_bb;
        assert_eq!(bc.get(Color::Black).as_u64(), new_bb.as_u64());
    }

    #[test]
    fn get_mut_set_and_clear_square() {
        let mut bc = empty();
        let bb = bc.get_mut(Color::White);
        *bb = bb.set_square(Square::C3);
        assert!(bc.get(Color::White).is_square_set(Square::C3));

        let bb = bc.get_mut(Color::White);
        *bb = bb.clear_square(Square::C3);
        assert!(bc.get(Color::White).empty());
    }

    #[test]
    fn whites_matches_get_white() {
        let bc = fixture();
        assert_eq!(bc.whites().as_u64(), bc.get(Color::White).as_u64());
    }

    #[test]
    fn blacks_matches_get_black() {
        let bc = fixture();
        assert_eq!(bc.blacks().as_u64(), bc.get(Color::Black).as_u64());
    }

    #[test]
    fn whites_mut_modifies_only_white_field() {
        let mut bc = empty();
        *bc.whites_mut() = Square::A1.to_bb();
        assert!(bc.whites().is_square_set(Square::A1));
        assert!(bc.blacks().empty());
    }

    #[test]
    fn blacks_mut_modifies_only_black_field() {
        let mut bc = empty();
        *bc.blacks_mut() = Square::A8.to_bb();
        assert!(bc.blacks().is_square_set(Square::A8));
        assert!(bc.whites().empty());
    }

    #[test]
    fn whites_mut_matches_get_mut_white() {
        let mut bc1 = empty();
        let mut bc2 = empty();
        *bc1.whites_mut() = Square::F6.to_bb();
        *bc2.get_mut(Color::White) = Square::F6.to_bb();
        assert_eq!(bc1.whites().as_u64(), bc2.whites().as_u64());
    }

    #[test]
    fn blacks_mut_matches_get_mut_black() {
        let mut bc1 = empty();
        let mut bc2 = empty();
        *bc1.blacks_mut() = Square::F3.to_bb();
        *bc2.get_mut(Color::Black) = Square::F3.to_bb();
        assert_eq!(bc1.blacks().as_u64(), bc2.blacks().as_u64());
    }

    #[test]
    fn peek_color_white_square_returns_white() {
        let bc = fixture();
        assert_eq!(bc.peek_color(Square::E1), Some(Color::White));
    }

    #[test]
    fn peek_color_black_square_returns_black() {
        let bc = fixture();
        assert_eq!(bc.peek_color(Square::E8), Some(Color::Black));
    }

    #[test]
    fn peek_color_empty_square_returns_none() {
        let bc = fixture();
        // D4 is not set in either board in the fixture
        assert_eq!(bc.peek_color(Square::D4), None);
    }

    #[test]
    fn peek_color_all_empty_returns_none_everywhere() {
        let bc = empty();
        for i in 0u32..64 {
            let sq = Square::from_u32_checked(i);
            assert_eq!(
                bc.peek_color(sq),
                None,
                "empty board should return None for {sq:?}"
            );
        }
    }

    #[test]
    fn peek_color_full_white_board() {
        let bc = ByColor::new(Bitboard::from_u64(u64::MAX), Bitboard::new_empty());
        for i in 0u32..64 {
            let sq = Square::from_u32_checked(i);
            assert_eq!(bc.peek_color(sq), Some(Color::White));
        }
    }

    #[test]
    fn peek_color_full_black_board() {
        let bc = ByColor::new(Bitboard::new_empty(), Bitboard::from_u64(u64::MAX));
        for i in 0u32..64 {
            let sq = Square::from_u32_checked(i);
            assert_eq!(bc.peek_color(sq), Some(Color::Black));
        }
    }

    #[test]
    fn peek_color_white_takes_priority_over_black_on_overlap() {
        // When a square is set in both boards, white is checked first and wins.
        let sq = Square::G5;
        let bc = ByColor::new(sq.to_bb(), sq.to_bb());
        assert_eq!(bc.peek_color(sq), Some(Color::White));
    }

    #[test]
    fn peek_color_many_distinct_squares() {
        let white_squares = [Square::A1, Square::C3, Square::E5, Square::G7];
        let black_squares = [Square::B2, Square::D4, Square::F6, Square::H8];

        let white_bb = white_squares
            .iter()
            .fold(Bitboard::new_empty(), |b, &s| b.set_square(s));

        let black_bb = black_squares
            .iter()
            .fold(Bitboard::new_empty(), |b, &s| b.set_square(s));

        let bc = ByColor::new(white_bb, black_bb);

        for sq in white_squares {
            assert_eq!(
                bc.peek_color(sq),
                Some(Color::White),
                "{sq:?} should be White"
            );
        }
        for sq in black_squares {
            assert_eq!(
                bc.peek_color(sq),
                Some(Color::Black),
                "{sq:?} should be Black"
            );
        }
    }

    #[test]
    fn peek_color_checked_white_square_returns_white() {
        let bc = fixture();
        assert_eq!(bc.peek_color_checked(Square::E1), Color::White);
    }

    #[test]
    fn peek_color_checked_black_square_returns_black() {
        let bc = fixture();
        assert_eq!(bc.peek_color_checked(Square::E8), Color::Black);
    }

    #[test]
    fn peek_color_checked_matches_peek_color_on_occupied_squares() {
        let bc = fixture();
        for sq in [Square::E1, Square::E8] {
            assert_eq!(
                Some(bc.peek_color_checked(sq)),
                bc.peek_color(sq),
                "checked and unchecked should agree on {sq:?}"
            );
        }
    }

    #[test]
    fn peek_color_checked_full_white_board_all_white() {
        let bc = ByColor::new(Bitboard::from_u64(u64::MAX), Bitboard::new_empty());
        for i in 0u32..64 {
            assert_eq!(
                bc.peek_color_checked(Square::from_u32_checked(i)),
                Color::White
            );
        }
    }

    #[test]
    fn peek_color_checked_full_black_board_all_black() {
        let bc = ByColor::new(Bitboard::new_empty(), Bitboard::from_u64(u64::MAX));
        for i in 0u32..64 {
            assert_eq!(
                bc.peek_color_checked(Square::from_u32_checked(i)),
                Color::Black
            );
        }
    }
}
