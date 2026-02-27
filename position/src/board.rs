use std::{fmt, ops::Not};

use types::{
    bitboard::{Bitboard, ToBitboard},
    by_color::ByColor,
    by_role::ByRole,
    color::Color,
    file::File,
    piece::Piece,
    rank::Rank,
    role::Role,
    square::Square,
};

#[derive(Clone)]
pub struct Board {
    occupied: Bitboard,
    by_role: ByRole,
    by_color: ByColor,
}

impl Board {
    /// create a board initialized with default chess position
    pub fn new() -> Self {
        let by_role = ByRole::new(
            0x00ff_0000_0000_ff00.to_bb(),
            0x4200_0000_0000_0042.to_bb(),
            0x2400_0000_0000_0024.to_bb(),
            0x8100_0000_0000_0081.to_bb(),
            0x0800_0000_0000_0008.to_bb(),
            0x1000_0000_0000_0010.to_bb(),
        );

        let by_color = ByColor::new(0x0000_0000_0000_ffff.to_bb(), 0xffff_0000_0000_0000.to_bb());

        let occupied = 0xffff_0000_0000_ffff.to_bb();

        Self {
            occupied,
            by_role,
            by_color,
        }
    }

    pub fn new_empty() -> Self {
        let by_role = ByRole::new(
            Bitboard::new_empty(),
            Bitboard::new_empty(),
            Bitboard::new_empty(),
            Bitboard::new_empty(),
            Bitboard::new_empty(),
            Bitboard::new_empty(),
        );

        let by_color = ByColor::new(Bitboard::new_empty(), Bitboard::new_empty());

        let occupied = Bitboard::new_empty();

        Self {
            occupied,
            by_role,
            by_color,
        }
    }

    #[inline(always)]
    pub fn occupied(&self) -> Bitboard {
        self.occupied
    }

    #[inline(always)]
    pub fn whites(&self) -> Bitboard {
        self.by_color.whites()
    }

    #[inline(always)]
    pub fn blacks(&self) -> Bitboard {
        self.by_color.blacks()
    }

    #[inline(always)]
    pub fn by_color(&self, color: Color) -> Bitboard {
        self.by_color.get(color)
    }

    #[inline(always)]
    pub fn pawns(&self, color: Color) -> Bitboard {
        let pawns = self.by_role.pawns();
        let color_mask = self.by_color.get(color);

        pawns & color_mask
    }

    #[inline(always)]
    pub fn knights(&self, color: Color) -> Bitboard {
        let knights = self.by_role.knights();
        let color_mask = self.by_color.get(color);

        knights & color_mask
    }

    #[inline(always)]
    pub fn bishops(&self, color: Color) -> Bitboard {
        let bishops = self.by_role.bishops();
        let color_mask = self.by_color.get(color);

        bishops & color_mask
    }

    #[inline(always)]
    pub fn rooks(&self, color: Color) -> Bitboard {
        let rooks = self.by_role.rooks();
        let color_mask = self.by_color.get(color);

        rooks & color_mask
    }

    #[inline(always)]
    pub fn queens(&self, color: Color) -> Bitboard {
        let queens = self.by_role.queens();
        let color_mask = self.by_color.get(color);

        queens & color_mask
    }

    /// returns the UNIQUE king of specified side, panics otherwise
    #[inline(always)]
    pub fn the_king(&self, color: Color) -> Square {
        self.king(color)
            .first_square()
            .expect("The king always exists and is always unique")
    }

    #[inline(always)]
    pub fn king(&self, color: Color) -> Bitboard {
        let kings = self.by_role.kings();
        let color_mask = self.by_color.get(color);

        kings & color_mask
    }

    #[inline(always)]
    pub fn peek(&self, square: Square) -> Option<Piece> {
        let role = self.by_role.peek_role(square)?;
        let color = self.by_color.peek_color(square)?;

        Some(Piece::of(role, color))
    }

    #[inline(always)]
    pub fn peek_checked(&self, square: Square) -> Piece {
        let role = self.by_role.peek_role_checked(square);
        let color = self.by_color.peek_color_checked(square);

        Piece::of(role, color)
    }

    #[inline(always)]
    pub fn peek_role(&self, square: Square) -> Option<Role> {
        self.by_role.peek_role(square)
    }

    #[inline(always)]
    pub fn peek_role_checked(&self, square: Square) -> Role {
        self.by_role.peek_role_checked(square)
    }

    #[inline(always)]
    pub fn peek_color(&self, square: Square) -> Option<Color> {
        self.by_color.peek_color(square)
    }

    #[inline(always)]
    pub fn peek_color_checked(&self, square: Square) -> Color {
        self.by_color.peek_color_checked(square)
    }

    #[inline(always)]
    pub fn attacks_to(&self, sqr: Square, opponent: Color) -> Bitboard {
        let occupied = self.occupied();

        let pawn = self.pawns(opponent) & lookup::pawn_attacks(!opponent, sqr).to_bb();

        let knight = self.knights(opponent) & lookup::knight_attacks(sqr).to_bb();

        let bishop =
            self.bishops(opponent) & lookup::bishop_attacks(sqr, occupied.as_u64()).to_bb();

        let rook = self.rooks(opponent) & lookup::rook_attacks(sqr, occupied.as_u64()).to_bb();

        let queen = self.queens(opponent) & lookup::queen_attacks(sqr, occupied.as_u64()).to_bb();

        let king = self.king(opponent) & lookup::king_attacks(sqr).to_bb();

        pawn | knight | bishop | rook | queen | king
    }

    #[inline(always)]
    pub fn sliding_attacks_to(&self, sqr: Square, opponent: Color) -> Bitboard {
        let occupied = self.occupied();

        let bishop =
            self.bishops(opponent) & lookup::bishop_attacks(sqr, occupied.as_u64()).to_bb();

        let rook = self.rooks(opponent) & lookup::rook_attacks(sqr, occupied.as_u64()).to_bb();

        let queen = self.queens(opponent) & lookup::queen_attacks(sqr, occupied.as_u64()).to_bb();

        bishop | rook | queen
    }

    /// determines if this square(empty or not) is attacked by the figures of the opposing color
    #[rustfmt::skip]
    #[inline(always)]
    pub fn is_square_attacked_by(&self, sqr: Square, opponent: Color) -> bool {
        let occupied = self.occupied();

        let sliders =
            (lookup::bishop_attacks(sqr, occupied.as_u64()).to_bb() & (self.bishops(opponent) | self.queens(opponent)))
            | (lookup::rook_attacks(sqr, occupied.as_u64()).to_bb() & (self.rooks(opponent) | self.queens(opponent)));

        if sliders.present() {
            return true;
        }

        let attackers = (lookup::pawn_attacks(!opponent, sqr).to_bb() & self.pawns(opponent))
            | (lookup::knight_attacks(sqr).to_bb() & self.knights(opponent))
            | (lookup::king_attacks(sqr).to_bb() & self.king(opponent));

        if attackers.present() {
            return true;
        }

        false
    }

    #[inline(always)]
    pub fn set_piece_at(&mut self, piece: Piece, square: Square) {
        let role_bb = self.by_role.get_mut(piece.role());
        let color_bb = self.by_color.get_mut(piece.color());

        *role_bb = role_bb.set_square(square);
        *color_bb = color_bb.set_square(square);

        self.occupied = self.occupied.set_square(square);
    }

    #[inline(always)]
    pub fn discard_piece_at(&mut self, square: Square) {
        let piece = self.peek_checked(square);
        let role_bb = self.by_role.get_mut(piece.role());
        let color_bb = self.by_color.get_mut(piece.color());

        *role_bb = role_bb.clear_square(square);
        *color_bb = color_bb.clear_square(square);

        self.occupied = self.occupied.clear_square(square);
    }

    #[must_use]
    #[inline(always)]
    pub fn remove_piece_at(&mut self, square: Square) -> Option<Piece> {
        let piece = self.peek(square)?;

        let role_bb = self.by_role.get_mut(piece.role());
        *role_bb = role_bb.clear_square(square);

        let color_bb = self.by_color.get_mut(piece.color());
        *color_bb = color_bb.clear_square(square);

        self.occupied = self.occupied.clear_square(square);

        Some(piece)
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{{")?;
        for rank in (0..8).rev().map(Rank::new_checked) {
            write!(f, "    ")?;
            for file in (0..8).map(File::new_checked) {
                let sqr = Square::of(file, rank);
                let maybe_piece = self.peek(sqr);

                match maybe_piece {
                    Some(piece) => write!(f, "{} ", piece.char())?,
                    None => write!(f, ". ")?,
                }
            }
            if matches!(rank, Rank::First).not() {
                writeln!(f)?;
            }
        }

        write!(f, "\n}}")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn peek_should_return_valid_piece_info_of_set_square() {
        let mut board = Board::new_empty();
        board.set_piece_at(Piece::WPawn, Square::C4);

        assert_eq!(board.peek(Square::C4).unwrap(), Piece::WPawn);
        assert_eq!(board.peek(Square::A1), None);
    }

    #[test]
    fn get_kings() {
        let board = Board::new();

        assert_eq!(board.the_king(Color::White), Square::E1);
        assert_eq!(board.the_king(Color::Black), Square::E8);
    }

    #[test]
    fn set_piece_at_should_set_valid_piece() {
        let mut board = Board::new();

        board.set_piece_at(Piece::WPawn, Square::C4);
        let piece = board.peek(Square::C4);
        assert_eq!(piece, Some(Piece::WPawn));
    }
}
