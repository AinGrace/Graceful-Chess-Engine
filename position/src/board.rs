use std::{
    cmp::{max, max_by},
    fmt,
    ops::Not,
};

use types::{
    bitboard::{Bitboard, ToBitboard},
    by_color::ByColor,
    by_role::ByRole,
    color::Color,
    file::File,
    piece::Piece::{self, WKing},
    rank::Rank,
    role::Role,
    square::Square,
};

#[derive(Clone, PartialEq, Eq)]
pub struct Board {
    occupied: Bitboard,
    by_role: ByRole<Bitboard>,
    by_color: ByColor<Bitboard>,
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
        *self.by_color.whites()
    }

    #[inline(always)]
    pub fn blacks(&self) -> Bitboard {
        *self.by_color.blacks()
    }

    #[inline(always)]
    pub fn by_color(&self, color: Color) -> Bitboard {
        *self.by_color.get(color)
    }

    #[inline(always)]
    pub fn pawns(&self, color: Color) -> Bitboard {
        let pawns = *self.by_role.pawns();
        let color_mask = *self.by_color.get(color);

        pawns & color_mask
    }

    #[inline(always)]
    pub fn knights(&self, color: Color) -> Bitboard {
        let knights = *self.by_role.knights();
        let color_mask = *self.by_color.get(color);

        knights & color_mask
    }

    #[inline(always)]
    pub fn bishops(&self, color: Color) -> Bitboard {
        let bishops = *self.by_role.bishops();
        let color_mask = *self.by_color.get(color);

        bishops & color_mask
    }

    #[inline(always)]
    pub fn rooks(&self, color: Color) -> Bitboard {
        let rooks = *self.by_role.rooks();
        let color_mask = *self.by_color.get(color);

        rooks & color_mask
    }

    #[inline(always)]
    pub fn queens(&self, color: Color) -> Bitboard {
        let queens = *self.by_role.queens();
        let color_mask = *self.by_color.get(color);

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
        let kings = *self.by_role.kings();
        let color_mask = self.by_color.get(color);

        kings & *color_mask
    }

    #[inline(always)]
    pub fn non_king_pieces_of(&self, color: Color) -> Bitboard {
        let pawns = self.pawns(color);
        let knights = self.knights(color);
        let bishops = self.bishops(color);
        let rooks = self.rooks(color);
        let queens = self.queens(color);

        pawns | knights | bishops | rooks | queens
    }

    #[inline(always)]
    pub fn is_endgame(&self) -> bool {
        self.occupied().popcnt() <= 8
    }

    #[inline(always)]
    pub fn dist_between_kings(&self) -> i16 {
        // Chebyshev distance
        let w_king = self.the_king(Color::White);
        let b_king = self.the_king(Color::Black);

        let x = w_king.file().to_i32() - b_king.file().to_i32();
        let y = w_king.rank().to_i32() - b_king.file().to_i32();

        max(x, y) as i16
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
    //TODO: consider pinned pieces and checks
    pub fn lva_to(&self, sqr: Square, us: Color) -> Option<Square> {
        let occupied = self.occupied();

        let pawn = self.pawns(us) & lookup::pawn_attacks(!us, sqr).to_bb();

        if let Some(pawn) = pawn.first_square() {
            return Some(pawn);
        }

        let knight = self.knights(us) & lookup::knight_attacks(sqr).to_bb();

        if let Some(knight) = knight.first_square() {
            return Some(knight);
        }

        let bishop = self.bishops(us) & lookup::bishop_attacks(sqr, occupied.as_u64()).to_bb();

        if let Some(bishop) = bishop.first_square() {
            return Some(bishop);
        }

        let rook = self.rooks(us) & lookup::rook_attacks(sqr, occupied.as_u64()).to_bb();

        if let Some(rook) = rook.first_square() {
            return Some(rook);
        }

        let queen = self.queens(us) & lookup::queen_attacks(sqr, occupied.as_u64()).to_bb();

        if let Some(queen) = queen.first_square() {
            return Some(queen);
        }

        let king = self.king(us) & lookup::king_attacks(sqr).to_bb();

        if let Some(king) = king.first_square() {
            return Some(king);
        }

        None
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
        if let Some(piece) = self.peek(square) {
            let role_bb = self.by_role.get_mut(piece.role());
            let color_bb = self.by_color.get_mut(piece.color());

            *role_bb = role_bb.clear_square(square);
            *color_bb = color_bb.clear_square(square);

            self.occupied = self.occupied.clear_square(square);
        }
    }

    #[inline(always)]
    pub fn discard_piece_at_checked(&mut self, square: Square) {
        let piece = self.peek_checked(square);

        let role_bb = self.by_role.get_mut(piece.role());
        let color_bb = self.by_color.get_mut(piece.color());

        *role_bb = role_bb.clear_square(square);
        *color_bb = color_bb.clear_square(square);

        self.occupied = self.occupied.clear_square(square);
    }

    #[must_use]
    #[inline(always)]
    pub fn take_piece_at(&mut self, square: Square) -> Option<Piece> {
        let piece = self.peek(square)?;

        let role_bb = self.by_role.get_mut(piece.role());
        let color_bb = self.by_color.get_mut(piece.color());

        *role_bb = role_bb.clear_square(square);
        *color_bb = color_bb.clear_square(square);

        self.occupied = self.occupied.clear_square(square);

        Some(piece)
    }

    #[must_use]
    #[inline(always)]
    pub fn take_piece_at_checked(&mut self, square: Square) -> Piece {
        let piece = self.peek_checked(square);

        let role_bb = self.by_role.get_mut(piece.role());
        let color_bb = self.by_color.get_mut(piece.color());

        *role_bb = role_bb.clear_square(square);
        *color_bb = color_bb.clear_square(square);

        self.occupied = self.occupied.clear_square(square);

        piece
    }

    #[must_use]
    #[inline(always)]
    pub fn replace_piece_at(&mut self, piece: Piece, square: Square) -> Option<Piece> {
        let old_piece = self.take_piece_at(square);
        self.set_piece_at(piece, square);

        old_piece
    }

    #[must_use]
    #[inline(always)]
    pub fn move_piece_to(&mut self, from: Square, to: Square) -> Option<Piece> {
        let moving_piece = self.take_piece_at(from)?;
        self.replace_piece_at(moving_piece, to)
    }

    pub fn is_insufficient_material(&self) -> bool {
        if (*self.by_role.pawns() | *self.by_role.rooks() | *self.by_role.queens()).present() {
            return false;
        }

        let w_sole_king = self.by_color(Color::White).popcnt() == 1;
        let b_sole_king = self.by_color(Color::Black).popcnt() == 1;

        if w_sole_king && b_sole_king {
            return true;
        }

        let sole_bishop = |color: Color| self.bishops(color).popcnt() == 1;

        let sole_bishop_or_knight = |color: Color| {
            let sole_bishop = sole_bishop(color);
            let sole_knight = self.knights(color).popcnt() == 1;

            sole_knight || sole_bishop
        };

        if (w_sole_king && sole_bishop_or_knight(Color::Black))
            || (b_sole_king && sole_bishop_or_knight(Color::White))
        {
            return true;
        }

        if sole_bishop(Color::White) && sole_bishop(Color::Black) {
            let w_square_is_dark = self
                .bishops(Color::White)
                .first_square_checked()
                .is_dark_square();

            let b_square_is_dark = self
                .bishops(Color::Black)
                .first_square_checked()
                .is_dark_square();

            if w_square_is_dark == b_square_is_dark {
                return true;
            }
        }

        false
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
        for rank in (0..8).rev().map(Rank::from_u32_checked) {
            write!(f, "    ")?;
            for file in (0..8).map(File::from_u32_checked) {
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
mod board_tests {
    use super::*;

    const ALL_PIECES: [Piece; 12] = [
        Piece::WPawn,
        Piece::WKnight,
        Piece::WBishop,
        Piece::WRook,
        Piece::WQueen,
        Piece::WKing,
        Piece::BPawn,
        Piece::BKnight,
        Piece::BBishop,
        Piece::BRook,
        Piece::BQueen,
        Piece::BKing,
    ];

    #[test]
    fn new_empty_has_no_occupied_squares() {
        let board = Board::new_empty();
        assert!(board.occupied().empty());
    }

    #[test]
    fn new_empty_has_no_pieces_anywhere() {
        let board = Board::new_empty();
        for i in 0u32..64 {
            let sq = Square::from_u32_checked(i);
            assert_eq!(board.peek(sq), None, "{sq:?} should be empty");
        }
    }

    #[test]
    fn new_empty_no_white_or_black_pieces() {
        let board = Board::new_empty();
        assert!(board.whites().empty());
        assert!(board.blacks().empty());
    }

    #[test]
    fn default_equals_new() {
        let default_board = Board::default();
        let new_board = Board::new();
        assert_eq!(
            default_board.occupied().as_u64(),
            new_board.occupied().as_u64()
        );
    }

    #[test]
    fn new_occupied_is_correct() {
        let board = Board::new();
        // Ranks 1, 2, 7, 8 should be occupied
        assert_eq!(board.occupied().as_u64(), 0xffff_0000_0000_ffff);
    }

    #[test]
    fn new_has_32_occupied_squares() {
        assert_eq!(Board::new().occupied().popcnt(), 32);
    }

    #[test]
    fn new_has_16_white_and_16_black_pieces() {
        let board = Board::new();
        assert_eq!(board.whites().popcnt(), 16);
        assert_eq!(board.blacks().popcnt(), 16);
    }

    #[test]
    fn new_white_pawns_on_rank_2() {
        let board = Board::new();
        let pawns = board.pawns(Color::White);
        assert_eq!(pawns.popcnt(), 8);
        for file in 0u32..8 {
            assert!(
                pawns.is_square_set(Square::from_u32_checked(8 + file)),
                "white pawn missing on file {file}"
            );
        }
    }

    #[test]
    fn new_black_pawns_on_rank_7() {
        let board = Board::new();
        let pawns = board.pawns(Color::Black);
        assert_eq!(pawns.popcnt(), 8);
        for file in 0u32..8 {
            assert!(
                pawns.is_square_set(Square::from_u32_checked(48 + file)),
                "black pawn missing on file {file}"
            );
        }
    }

    #[test]
    fn new_white_rooks_on_a1_and_h1() {
        let board = Board::new();
        let rooks = board.rooks(Color::White);
        assert_eq!(rooks.popcnt(), 2);
        assert!(rooks.is_square_set(Square::A1));
        assert!(rooks.is_square_set(Square::H1));
    }

    #[test]
    fn new_black_rooks_on_a8_and_h8() {
        let board = Board::new();
        let rooks = board.rooks(Color::Black);
        assert_eq!(rooks.popcnt(), 2);
        assert!(rooks.is_square_set(Square::A8));
        assert!(rooks.is_square_set(Square::H8));
    }

    #[test]
    fn new_white_knights_on_b1_and_g1() {
        let board = Board::new();
        let knights = board.knights(Color::White);
        assert_eq!(knights.popcnt(), 2);
        assert!(knights.is_square_set(Square::B1));
        assert!(knights.is_square_set(Square::G1));
    }

    #[test]
    fn new_black_knights_on_b8_and_g8() {
        let board = Board::new();
        let knights = board.knights(Color::Black);
        assert_eq!(knights.popcnt(), 2);
        assert!(knights.is_square_set(Square::B8));
        assert!(knights.is_square_set(Square::G8));
    }

    #[test]
    fn new_white_bishops_on_c1_and_f1() {
        let board = Board::new();
        let bishops = board.bishops(Color::White);
        assert_eq!(bishops.popcnt(), 2);
        assert!(bishops.is_square_set(Square::C1));
        assert!(bishops.is_square_set(Square::F1));
    }

    #[test]
    fn new_black_bishops_on_c8_and_f8() {
        let board = Board::new();
        let bishops = board.bishops(Color::Black);
        assert_eq!(bishops.popcnt(), 2);
        assert!(bishops.is_square_set(Square::C8));
        assert!(bishops.is_square_set(Square::F8));
    }

    #[test]
    fn new_white_queen_on_d1() {
        let board = Board::new();
        let queens = board.queens(Color::White);
        assert_eq!(queens.popcnt(), 1);
        assert!(queens.is_square_set(Square::D1));
    }

    #[test]
    fn new_black_queen_on_d8() {
        let board = Board::new();
        let queens = board.queens(Color::Black);
        assert_eq!(queens.popcnt(), 1);
        assert!(queens.is_square_set(Square::D8));
    }

    #[test]
    fn new_white_king_on_e1() {
        let board = Board::new();
        assert_eq!(board.the_king(Color::White), Square::E1);
    }

    #[test]
    fn new_black_king_on_e8() {
        let board = Board::new();
        assert_eq!(board.the_king(Color::Black), Square::E8);
    }

    #[test]
    fn new_ranks_3_through_6_are_empty() {
        let board = Board::new();
        for i in 16u32..48 {
            let sq = Square::from_u32_checked(i);
            assert_eq!(
                board.peek(sq),
                None,
                "{sq:?} should be empty in starting position"
            );
        }
    }

    #[test]
    fn new_starting_pieces_peek_correctly() {
        let board = Board::new();
        // White back rank
        assert_eq!(board.peek(Square::A1), Some(Piece::WRook));
        assert_eq!(board.peek(Square::B1), Some(Piece::WKnight));
        assert_eq!(board.peek(Square::C1), Some(Piece::WBishop));
        assert_eq!(board.peek(Square::D1), Some(Piece::WQueen));
        assert_eq!(board.peek(Square::E1), Some(Piece::WKing));
        assert_eq!(board.peek(Square::F1), Some(Piece::WBishop));
        assert_eq!(board.peek(Square::G1), Some(Piece::WKnight));
        assert_eq!(board.peek(Square::H1), Some(Piece::WRook));
        // Black back rank
        assert_eq!(board.peek(Square::A8), Some(Piece::BRook));
        assert_eq!(board.peek(Square::B8), Some(Piece::BKnight));
        assert_eq!(board.peek(Square::C8), Some(Piece::BBishop));
        assert_eq!(board.peek(Square::D8), Some(Piece::BQueen));
        assert_eq!(board.peek(Square::E8), Some(Piece::BKing));
        assert_eq!(board.peek(Square::F8), Some(Piece::BBishop));
        assert_eq!(board.peek(Square::G8), Some(Piece::BKnight));
        assert_eq!(board.peek(Square::H8), Some(Piece::BRook));
    }

    #[test]
    fn occupied_equals_whites_union_blacks() {
        let board = Board::new();
        let combined = board.whites() | board.blacks();
        assert_eq!(board.occupied().as_u64(), combined.as_u64());
    }

    #[test]
    fn whites_and_blacks_are_disjoint() {
        let board = Board::new();
        assert!(!board.whites().intersects(board.blacks()));
    }

    #[test]
    fn by_color_matches_whites_and_blacks() {
        let board = Board::new();
        assert_eq!(
            board.by_color(Color::White).as_u64(),
            board.whites().as_u64()
        );
        assert_eq!(
            board.by_color(Color::Black).as_u64(),
            board.blacks().as_u64()
        );
    }

    #[test]
    fn occupied_updates_after_set_piece() {
        let mut board = Board::new_empty();
        assert!(board.occupied().empty());
        board.set_piece_at(Piece::WKing, Square::E4);
        assert!(board.occupied().is_square_set(Square::E4));
        assert_eq!(board.occupied().popcnt(), 1);
    }

    #[test]
    fn occupied_updates_after_discard() {
        let mut board = Board::new_empty();
        board.set_piece_at(Piece::WKing, Square::E1);
        board.discard_piece_at(Square::E1);
        assert!(board.occupied().empty());
    }

    #[test]
    fn set_piece_at_all_pieces_all_squares() {
        for piece in ALL_PIECES {
            let mut board = Board::new_empty();
            let sq = Square::E4;
            board.set_piece_at(piece, sq);
            assert_eq!(board.peek(sq), Some(piece));
            assert!(board.occupied().is_square_set(sq));
        }
    }

    #[test]
    fn set_piece_at_updates_role_bitboard() {
        let mut board = Board::new_empty();
        board.set_piece_at(Piece::WRook, Square::A1);
        assert!(board.rooks(Color::White).is_square_set(Square::A1));
    }

    #[test]
    fn set_piece_at_updates_color_bitboard() {
        let mut board = Board::new_empty();
        board.set_piece_at(Piece::BKnight, Square::G8);
        assert!(board.blacks().is_square_set(Square::G8));
        assert!(!board.whites().is_square_set(Square::G8));
    }

    #[test]
    fn set_piece_at_does_not_affect_other_squares() {
        let mut board = Board::new_empty();
        board.set_piece_at(Piece::WPawn, Square::E4);
        for i in 0u32..64 {
            let sq = Square::from_u32_checked(i);
            if sq == Square::E4 {
                continue;
            }
            assert_eq!(board.peek(sq), None, "{sq:?} should remain empty");
        }
    }

    #[test]
    fn set_piece_at_multiple_pieces() {
        let mut board = Board::new_empty();
        let placements = [
            (Piece::WKing, Square::E1),
            (Piece::BKing, Square::E8),
            (Piece::WQueen, Square::D1),
            (Piece::BQueen, Square::D8),
        ];
        for (piece, sq) in placements {
            board.set_piece_at(piece, sq);
        }
        for (piece, sq) in placements {
            assert_eq!(board.peek(sq), Some(piece));
        }
        assert_eq!(board.occupied().popcnt(), 4);
    }

    #[test]
    fn set_piece_at_is_idempotent_for_same_piece() {
        let mut board = Board::new_empty();
        board.set_piece_at(Piece::WKing, Square::E1);
        board.set_piece_at(Piece::WKing, Square::E1);
        assert_eq!(board.peek(Square::E1), Some(Piece::WKing));
        assert_eq!(board.occupied().popcnt(), 1);
    }

    #[test]
    fn peek_returns_none_on_empty_board() {
        let board = Board::new_empty();
        for i in 0u32..64 {
            assert_eq!(board.peek(Square::from_u32_checked(i)), None);
        }
    }

    #[test]
    fn peek_returns_correct_piece_after_set() {
        let mut board = Board::new_empty();
        for (i, &piece) in ALL_PIECES.iter().enumerate() {
            let sq = Square::from_u32_checked(i as u32);
            board.set_piece_at(piece, sq);
            assert_eq!(board.peek(sq), Some(piece));
        }
    }

    #[test]
    fn peek_role_returns_correct_role() {
        let mut board = Board::new_empty();
        let pairs = [
            (Piece::WPawn, Square::A2, Role::Pawn),
            (Piece::BKnight, Square::B8, Role::Knight),
            (Piece::WQueen, Square::D1, Role::Queen),
        ];
        for (piece, sq, role) in pairs {
            board.set_piece_at(piece, sq);
            assert_eq!(board.peek_role(sq), Some(role));
        }
    }

    #[test]
    fn peek_role_returns_none_on_empty_square() {
        let board = Board::new_empty();
        assert_eq!(board.peek_role(Square::E4), None);
    }

    #[test]
    fn peek_color_returns_correct_color() {
        let mut board = Board::new_empty();
        board.set_piece_at(Piece::WKing, Square::E1);
        board.set_piece_at(Piece::BKing, Square::E8);
        assert_eq!(board.peek_color(Square::E1), Some(Color::White));
        assert_eq!(board.peek_color(Square::E8), Some(Color::Black));
    }

    #[test]
    fn peek_color_returns_none_on_empty_square() {
        let board = Board::new_empty();
        assert_eq!(board.peek_color(Square::D4), None);
    }

    #[test]
    fn peek_checked_returns_piece_on_occupied_square() {
        let mut board = Board::new_empty();
        board.set_piece_at(Piece::BRook, Square::H8);
        assert_eq!(board.peek_checked(Square::H8), Piece::BRook);
    }

    #[test]
    fn peek_role_checked_returns_role_on_occupied_square() {
        let mut board = Board::new_empty();
        board.set_piece_at(Piece::WBishop, Square::C1);
        assert_eq!(board.peek_role_checked(Square::C1), Role::Bishop);
    }

    #[test]
    fn discard_piece_at_removes_piece() {
        let mut board = Board::new_empty();
        board.set_piece_at(Piece::WPawn, Square::E4);
        board.discard_piece_at(Square::E4);
        assert_eq!(board.peek(Square::E4), None);
        assert!(!board.occupied().is_square_set(Square::E4));
    }

    #[test]
    fn discard_piece_at_clears_role_and_color_bitboards() {
        let mut board = Board::new_empty();
        board.set_piece_at(Piece::BRook, Square::A8);
        board.discard_piece_at(Square::A8);
        assert!(board.rooks(Color::Black).empty());
        assert!(board.blacks().empty());
        assert!(board.occupied().empty());
    }

    #[test]
    fn discard_piece_at_on_empty_square_is_noop() {
        let mut board = Board::new_empty();
        board.set_piece_at(Piece::WKing, Square::E1);
        board.discard_piece_at(Square::H8); // H8 is empty
        assert_eq!(board.peek(Square::E1), Some(Piece::WKing));
        assert_eq!(board.occupied().popcnt(), 1);
    }

    #[test]
    fn discard_piece_at_does_not_affect_other_squares() {
        let mut board = Board::new_empty();
        board.set_piece_at(Piece::WKing, Square::E1);
        board.set_piece_at(Piece::BKing, Square::E8);
        board.discard_piece_at(Square::E1);
        assert_eq!(board.peek(Square::E8), Some(Piece::BKing));
        assert_eq!(board.occupied().popcnt(), 1);
    }

    #[test]
    fn discard_piece_at_checked_removes_piece() {
        let mut board = Board::new_empty();
        board.set_piece_at(Piece::WQueen, Square::D1);
        board.discard_piece_at_checked(Square::D1);
        assert_eq!(board.peek(Square::D1), None);
        assert!(board.occupied().empty());
    }

    #[test]
    fn discard_piece_at_checked_clears_bitboards() {
        let mut board = Board::new_empty();
        board.set_piece_at(Piece::BQueen, Square::D8);
        board.discard_piece_at_checked(Square::D8);
        assert!(board.queens(Color::Black).empty());
        assert!(board.blacks().empty());
    }

    #[test]
    fn remove_piece_at_returns_the_piece() {
        let mut board = Board::new_empty();
        board.set_piece_at(Piece::WKnight, Square::G1);
        let removed = board.take_piece_at(Square::G1);
        assert_eq!(removed, Some(Piece::WKnight));
    }

    #[test]
    fn remove_piece_at_clears_square() {
        let mut board = Board::new_empty();
        board.set_piece_at(Piece::WKnight, Square::G1);
        let _ = board.take_piece_at(Square::G1);
        assert_eq!(board.peek(Square::G1), None);
        assert!(board.occupied().empty());
    }

    #[test]
    fn remove_piece_at_on_empty_returns_none() {
        let mut board = Board::new_empty();
        assert_eq!(board.take_piece_at(Square::E4), None);
    }

    #[test]
    fn remove_piece_at_does_not_affect_others() {
        let mut board = Board::new_empty();
        board.set_piece_at(Piece::WKing, Square::E1);
        board.set_piece_at(Piece::BKing, Square::E8);
        let _ = board.take_piece_at(Square::E1);
        assert_eq!(board.peek(Square::E8), Some(Piece::BKing));
    }

    #[test]
    fn remove_piece_at_clears_role_and_color_bitboards() {
        let mut board = Board::new_empty();
        board.set_piece_at(Piece::BBishop, Square::F8);
        let _ = board.take_piece_at(Square::F8);
        assert!(board.bishops(Color::Black).empty());
        assert!(board.blacks().empty());
    }

    #[test]
    fn replace_piece_at_replaces_with_new_piece() {
        let mut board = Board::new_empty();
        board.set_piece_at(Piece::WPawn, Square::E7);
        board.set_piece_at(Piece::WQueen, Square::E7);
        assert_eq!(board.peek(Square::E7), Some(Piece::WQueen));
    }

    #[test]
    fn replace_piece_at_clears_old_piece_bitboards() {
        let mut board = Board::new_empty();
        board.set_piece_at(Piece::WPawn, Square::E7);
        board.set_piece_at(Piece::WQueen, Square::E7);
        // Old pawn board must be clear
        assert!(!board.pawns(Color::White).is_square_set(Square::E7));
        // New queen board must be set
        assert!(board.queens(Color::White).is_square_set(Square::E7));
        assert_eq!(board.occupied().popcnt(), 1);
    }

    #[test]
    fn replace_piece_at_with_different_color() {
        let mut board = Board::new_empty();
        board.set_piece_at(Piece::WRook, Square::A1);
        board.set_piece_at(Piece::BRook, Square::A1);
        assert_eq!(board.peek(Square::A1), Some(Piece::BRook));
        assert!(board.rooks(Color::White).empty());
        assert!(board.rooks(Color::Black).is_square_set(Square::A1));
        assert!(board.whites().empty());
        assert!(board.blacks().is_square_set(Square::A1));
    }

    #[test]
    fn replace_piece_at_checked_replaces_piece() {
        let mut board = Board::new_empty();
        board.set_piece_at(Piece::BPawn, Square::A2);
        board.set_piece_at(Piece::BQueen, Square::A2);
        assert_eq!(board.peek(Square::A2), Some(Piece::BQueen));
        assert!(!board.pawns(Color::Black).is_square_set(Square::A2));
        assert!(board.queens(Color::Black).is_square_set(Square::A2));
    }

    #[test]
    fn role_accessors_return_only_matching_role_and_color() {
        let mut board = Board::new_empty();
        board.set_piece_at(Piece::WPawn, Square::E4);
        board.set_piece_at(Piece::BPawn, Square::E5);
        board.set_piece_at(Piece::WKnight, Square::D4);

        // White pawns
        let wp = board.pawns(Color::White);
        assert!(wp.is_square_set(Square::E4));
        assert!(!wp.is_square_set(Square::E5));
        assert!(!wp.is_square_set(Square::D4));

        // Black pawns
        let bp = board.pawns(Color::Black);
        assert!(bp.is_square_set(Square::E5));
        assert!(!bp.is_square_set(Square::E4));

        // White knights
        let wn = board.knights(Color::White);
        assert!(wn.is_square_set(Square::D4));
        assert!(!wn.is_square_set(Square::E4));
    }

    #[test]
    fn role_accessors_empty_when_role_absent() {
        let mut board = Board::new_empty();
        board.set_piece_at(Piece::WKing, Square::E1);
        assert!(board.pawns(Color::White).empty());
        assert!(board.knights(Color::White).empty());
        assert!(board.bishops(Color::White).empty());
        assert!(board.rooks(Color::White).empty());
        assert!(board.queens(Color::White).empty());
        assert!(board.pawns(Color::Black).empty());
    }

    #[test]
    fn role_accessors_intersection_with_color_is_correct() {
        let board = Board::new();
        // In the starting position: white pawns ∩ black squares should be empty
        assert!(!board.pawns(Color::White).intersects(board.blacks()));
        assert!(!board.pawns(Color::Black).intersects(board.whites()));
        assert!(!board.rooks(Color::White).intersects(board.blacks()));
        assert!(!board.rooks(Color::Black).intersects(board.whites()));
    }

    #[test]
    fn the_king_starting_position() {
        let board = Board::new();
        assert_eq!(board.the_king(Color::White), Square::E1);
        assert_eq!(board.the_king(Color::Black), Square::E8);
    }

    #[test]
    fn the_king_after_move() {
        let mut board = Board::new_empty();
        board.set_piece_at(Piece::WKing, Square::G1);
        assert_eq!(board.the_king(Color::White), Square::G1);
    }

    #[test]
    fn king_bitboard_has_exactly_one_square() {
        let board = Board::new();
        assert_eq!(board.king(Color::White).popcnt(), 1);
        assert_eq!(board.king(Color::Black).popcnt(), 1);
    }

    #[test]
    #[should_panic]
    fn the_king_panics_when_no_king_present() {
        let board = Board::new_empty();
        let _ = board.the_king(Color::White);
    }

    #[test]
    fn occupied_is_union_of_all_role_bitboards() {
        let board = Board::new();
        let all_roles = board.pawns(Color::White)
            | board.knights(Color::White)
            | board.bishops(Color::White)
            | board.rooks(Color::White)
            | board.queens(Color::White)
            | board.king(Color::White)
            | board.pawns(Color::Black)
            | board.knights(Color::Black)
            | board.bishops(Color::Black)
            | board.rooks(Color::Black)
            | board.queens(Color::Black)
            | board.king(Color::Black);
        assert_eq!(board.occupied().as_u64(), all_roles.as_u64());
    }

    #[test]
    fn role_bitboards_are_pairwise_disjoint() {
        let board = Board::new();
        let roles_white = [
            board.pawns(Color::White),
            board.knights(Color::White),
            board.bishops(Color::White),
            board.rooks(Color::White),
            board.queens(Color::White),
            board.king(Color::White),
            board.pawns(Color::Black),
            board.knights(Color::Black),
            board.bishops(Color::Black),
            board.rooks(Color::Black),
            board.queens(Color::Black),
            board.king(Color::Black),
        ];
        for i in 0..roles_white.len() {
            for j in (i + 1)..roles_white.len() {
                assert!(
                    !roles_white[i].intersects(roles_white[j]),
                    "role bitboards [{i}] and [{j}] overlap"
                );
            }
        }
    }

    #[test]
    fn occupied_never_has_phantom_bits_after_operations() {
        let mut board = Board::new_empty();
        board.set_piece_at(Piece::WKing, Square::E1);
        board.set_piece_at(Piece::BKing, Square::E8);
        board.discard_piece_at(Square::E1);
        board.discard_piece_at(Square::E8);
        assert_eq!(
            board.occupied().as_u64(),
            0,
            "occupied should be zero after all discards"
        );
    }

    #[test]
    fn occupied_consistent_after_replace() {
        let mut board = Board::new_empty();
        board.set_piece_at(Piece::WPawn, Square::E7);
        board.set_piece_at(Piece::WQueen, Square::E7);
        assert_eq!(board.occupied().popcnt(), 1);
        assert!(board.occupied().is_square_set(Square::E7));
    }

    #[test]
    fn debug_starts_and_ends_with_braces() {
        let s = format!("{:?}", Board::new());
        assert!(s.starts_with('{'));
        assert!(s.ends_with('}'));
    }

    #[test]
    fn debug_empty_board_has_only_dots() {
        let s = format!("{:?}", Board::new_empty());
        assert!(!s.contains(|c: char| c.is_alphabetic()));
    }

    #[test]
    fn debug_starting_position_contains_expected_chars() {
        let s = format!("{:?}", Board::new());
        // White pieces uppercase, black pieces lowercase
        for c in ['R', 'N', 'B', 'Q', 'K', 'P'] {
            assert!(s.contains(c), "debug should contain '{c}'");
        }
        for c in ['r', 'n', 'b', 'q', 'k', 'p'] {
            assert!(s.contains(c), "debug should contain '{c}'");
        }
    }

    #[test]
    fn full_round_trip_set_peek_discard_all_pieces_all_squares() {
        for piece in ALL_PIECES {
            for i in 0u32..64 {
                let sq = Square::from_u32_checked(i);
                let mut board = Board::new_empty();
                board.set_piece_at(piece, sq);
                assert_eq!(board.peek(sq), Some(piece));
                assert!(board.occupied().is_square_set(sq));

                board.discard_piece_at(sq);
                assert_eq!(board.peek(sq), None);
                assert!(!board.occupied().is_square_set(sq));
            }
        }
    }

    #[test]
    fn pawn_promotion_sequence() {
        let mut board = Board::new_empty();
        board.set_piece_at(Piece::WPawn, Square::E7);
        assert_eq!(board.peek(Square::E7), Some(Piece::WPawn));

        board.set_piece_at(Piece::WQueen, Square::E7);
        assert_eq!(board.peek(Square::E7), Some(Piece::WQueen));
        assert!(!board.pawns(Color::White).is_square_set(Square::E7));
        assert!(board.queens(Color::White).is_square_set(Square::E7));
        assert_eq!(board.occupied().popcnt(), 1);
    }

    #[test]
    fn capture_sequence() {
        let mut board = Board::new_empty();
        board.set_piece_at(Piece::WRook, Square::A1);
        board.set_piece_at(Piece::BPawn, Square::A8);
        assert_eq!(board.occupied().popcnt(), 2);

        // White rook captures black pawn
        let _ = board.take_piece_at(Square::A8); // remove captured piece
        let _ = board.take_piece_at(Square::A1); // move rook
        board.set_piece_at(Piece::WRook, Square::A8);

        assert_eq!(board.peek(Square::A8), Some(Piece::WRook));
        assert_eq!(board.peek(Square::A1), None);
        assert_eq!(board.occupied().popcnt(), 1);
        assert!(board.pawns(Color::Black).empty());
    }

    #[test]
    fn place_all_12_pieces_simultaneously() {
        let squares = [
            Square::A1,
            Square::B1,
            Square::C1,
            Square::D1,
            Square::E1,
            Square::F1,
            Square::A8,
            Square::B8,
            Square::C8,
            Square::D8,
            Square::E8,
            Square::F8,
        ];
        let mut board = Board::new_empty();
        for (piece, sq) in ALL_PIECES.iter().zip(squares.iter()) {
            board.set_piece_at(*piece, *sq);
        }
        assert_eq!(board.occupied().popcnt(), 12);
        for (piece, sq) in ALL_PIECES.iter().zip(squares.iter()) {
            assert_eq!(board.peek(*sq), Some(*piece));
        }
    }
}
