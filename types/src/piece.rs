use std::fmt::Debug;

use crate::{color::Color, role::Role};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Piece {
    WPawn = 0,
    WKnight = 1,
    WBishop = 2,
    WRook = 3,
    WQueen = 4,
    WKing = 5,

    BPawn = 6,
    BKnight = 7,
    BBishop = 8,
    BRook = 9,
    BQueen = 10,
    BKing = 11,
}

impl Piece {
    #[rustfmt::skip]
    #[inline(always)]
    pub fn of(role: Role, color: Color) -> Self {
        match (role, color) {
            (Role::Pawn,   Color::White) => Self::WPawn,
            (Role::Pawn,   Color::Black) => Self::BPawn,
            (Role::Knight, Color::White) => Self::WKnight,
            (Role::Knight, Color::Black) => Self::BKnight,
            (Role::Bishop, Color::White) => Self::WBishop,
            (Role::Bishop, Color::Black) => Self::BBishop,
            (Role::Rook,   Color::White) => Self::WRook,
            (Role::Rook,   Color::Black) => Self::BRook,
            (Role::Queen,  Color::White) => Self::WQueen,
            (Role::Queen,  Color::Black) => Self::BQueen,
            (Role::King,   Color::White) => Self::WKing,
            (Role::King,   Color::Black) => Self::BKing,
        }
    }

    pub fn from_char(chr: char) -> Option<Self> {
        match chr {
            'P' => Some(Self::WPawn),
            'N' => Some(Self::WKnight),
            'B' => Some(Self::WBishop),
            'R' => Some(Self::WRook),
            'Q' => Some(Self::WQueen),
            'K' => Some(Self::WKing),
            'p' => Some(Self::BPawn),
            'n' => Some(Self::BKnight),
            'b' => Some(Self::BBishop),
            'r' => Some(Self::BRook),
            'q' => Some(Self::BQueen),
            'k' => Some(Self::BKing),
            _unknown => None,
        }
    }

    #[rustfmt::skip]
    pub fn char(&self) -> char {
        match self {
            Piece::WPawn   => 'P',
            Piece::WKnight => 'N',
            Piece::WBishop => 'B',
            Piece::WRook   => 'R',
            Piece::WQueen  => 'Q',
            Piece::WKing   => 'K',
            Piece::BPawn   => 'p',
            Piece::BKnight => 'n',
            Piece::BBishop => 'b',
            Piece::BRook   => 'r',
            Piece::BQueen  => 'q',
            Piece::BKing   => 'k',
        }
    }

    #[rustfmt::skip]
    pub const fn role(&self) -> Role {
        match self {
            Piece::WPawn   | Piece::BPawn   => Role::Pawn,
            Piece::WKnight | Piece::BKnight => Role::Knight,
            Piece::WBishop | Piece::BBishop => Role::Bishop,
            Piece::WRook   | Piece::BRook   => Role::Rook,
            Piece::WQueen  | Piece::BQueen  => Role::Queen,
            Piece::WKing   | Piece::BKing   => Role::King,
        }
    }

    pub const fn color(&self) -> Color {
        match self {
            Piece::WPawn
            | Piece::WKnight
            | Piece::WBishop
            | Piece::WRook
            | Piece::WQueen
            | Piece::WKing => Color::White,

            Piece::BPawn
            | Piece::BKnight
            | Piece::BBishop
            | Piece::BRook
            | Piece::BQueen
            | Piece::BKing => Color::Black,
        }
    }

    pub const fn as_usize(self) -> usize {
        self as usize
    }
}

#[cfg(test)]
mod piece_tests {
    use super::*;
    use crate::{color::Color, role::Role};

    const WHITE_PIECES: [Piece; 6] = [
        Piece::WPawn,
        Piece::WKnight,
        Piece::WBishop,
        Piece::WRook,
        Piece::WQueen,
        Piece::WKing,
    ];
    const BLACK_PIECES: [Piece; 6] = [
        Piece::BPawn,
        Piece::BKnight,
        Piece::BBishop,
        Piece::BRook,
        Piece::BQueen,
        Piece::BKing,
    ];
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
    fn discriminants_are_correct() {
        assert_eq!(Piece::WPawn as u8, 0);
        assert_eq!(Piece::WKnight as u8, 1);
        assert_eq!(Piece::WBishop as u8, 2);
        assert_eq!(Piece::WRook as u8, 3);
        assert_eq!(Piece::WQueen as u8, 4);
        assert_eq!(Piece::WKing as u8, 5);
        assert_eq!(Piece::BPawn as u8, 6);
        assert_eq!(Piece::BKnight as u8, 7);
        assert_eq!(Piece::BBishop as u8, 8);
        assert_eq!(Piece::BRook as u8, 9);
        assert_eq!(Piece::BQueen as u8, 10);
        assert_eq!(Piece::BKing as u8, 11);
    }

    #[test]
    fn as_usize_matches_discriminant() {
        for piece in ALL_PIECES {
            assert_eq!(piece.as_usize(), piece as usize);
        }
    }

    #[test]
    fn as_usize_range_is_0_to_11() {
        for piece in ALL_PIECES {
            let idx = piece.as_usize();
            assert!(
                idx < 12,
                "as_usize() for {piece:?} should be < 12, got {idx}"
            );
        }
    }

    #[test]
    fn as_usize_values_are_unique() {
        let indices: std::collections::HashSet<usize> =
            ALL_PIECES.iter().map(|p| p.as_usize()).collect();
        assert_eq!(indices.len(), 12);
    }

    #[test]
    fn white_pieces_occupy_indices_0_to_5() {
        for piece in WHITE_PIECES {
            assert!(piece.as_usize() < 6, "{piece:?} should have index < 6");
        }
    }

    #[test]
    fn black_pieces_occupy_indices_6_to_11() {
        for piece in BLACK_PIECES {
            let idx = piece.as_usize();
            assert!(idx >= 6 && idx < 12, "{piece:?} should have index 6..12");
        }
    }

    #[test]
    fn of_white_pieces() {
        assert_eq!(Piece::of(Role::Pawn, Color::White), Piece::WPawn);
        assert_eq!(Piece::of(Role::Knight, Color::White), Piece::WKnight);
        assert_eq!(Piece::of(Role::Bishop, Color::White), Piece::WBishop);
        assert_eq!(Piece::of(Role::Rook, Color::White), Piece::WRook);
        assert_eq!(Piece::of(Role::Queen, Color::White), Piece::WQueen);
        assert_eq!(Piece::of(Role::King, Color::White), Piece::WKing);
    }

    #[test]
    fn of_black_pieces() {
        assert_eq!(Piece::of(Role::Pawn, Color::Black), Piece::BPawn);
        assert_eq!(Piece::of(Role::Knight, Color::Black), Piece::BKnight);
        assert_eq!(Piece::of(Role::Bishop, Color::Black), Piece::BBishop);
        assert_eq!(Piece::of(Role::Rook, Color::Black), Piece::BRook);
        assert_eq!(Piece::of(Role::Queen, Color::Black), Piece::BQueen);
        assert_eq!(Piece::of(Role::King, Color::Black), Piece::BKing);
    }

    #[test]
    fn of_round_trips_role_and_color() {
        let roles = [
            Role::Pawn,
            Role::Knight,
            Role::Bishop,
            Role::Rook,
            Role::Queen,
            Role::King,
        ];
        for role in roles {
            for color in [Color::White, Color::Black] {
                let piece = Piece::of(role, color);
                assert_eq!(piece.role(), role);
                assert_eq!(piece.color(), color);
            }
        }
    }

    #[test]
    fn from_char_uppercase_are_white() {
        assert_eq!(Piece::from_char('P'), Some(Piece::WPawn));
        assert_eq!(Piece::from_char('N'), Some(Piece::WKnight));
        assert_eq!(Piece::from_char('B'), Some(Piece::WBishop));
        assert_eq!(Piece::from_char('R'), Some(Piece::WRook));
        assert_eq!(Piece::from_char('Q'), Some(Piece::WQueen));
        assert_eq!(Piece::from_char('K'), Some(Piece::WKing));
    }

    #[test]
    fn from_char_lowercase_are_black() {
        assert_eq!(Piece::from_char('p'), Some(Piece::BPawn));
        assert_eq!(Piece::from_char('n'), Some(Piece::BKnight));
        assert_eq!(Piece::from_char('b'), Some(Piece::BBishop));
        assert_eq!(Piece::from_char('r'), Some(Piece::BRook));
        assert_eq!(Piece::from_char('q'), Some(Piece::BQueen));
        assert_eq!(Piece::from_char('k'), Some(Piece::BKing));
    }

    #[test]
    fn from_char_invalid_letters_are_none() {
        for c in [
            'a', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'm', 'o', 's', 't', 'A', 'C', 'D', 'E',
            'F', 'G', 'H', 'I', 'J', 'M', 'O', 'S', 'T',
        ] {
            assert_eq!(Piece::from_char(c), None, "'{c}' should return None");
        }
    }

    #[test]
    fn from_char_digits_are_none() {
        for c in '0'..='9' {
            assert_eq!(Piece::from_char(c), None, "digit '{c}' should return None");
        }
    }

    #[test]
    fn from_char_special_chars_are_none() {
        for c in [' ', '\n', '\t', '-', '_', '/', '.', ',', '!'] {
            assert_eq!(Piece::from_char(c), None, "'{c}' should return None");
        }
    }

    #[test]
    fn char_white_pieces_are_uppercase() {
        assert_eq!(Piece::WPawn.char(), 'P');
        assert_eq!(Piece::WKnight.char(), 'N');
        assert_eq!(Piece::WBishop.char(), 'B');
        assert_eq!(Piece::WRook.char(), 'R');
        assert_eq!(Piece::WQueen.char(), 'Q');
        assert_eq!(Piece::WKing.char(), 'K');
    }

    #[test]
    fn char_black_pieces_are_lowercase() {
        assert_eq!(Piece::BPawn.char(), 'p');
        assert_eq!(Piece::BKnight.char(), 'n');
        assert_eq!(Piece::BBishop.char(), 'b');
        assert_eq!(Piece::BRook.char(), 'r');
        assert_eq!(Piece::BQueen.char(), 'q');
        assert_eq!(Piece::BKing.char(), 'k');
    }

    #[test]
    fn char_white_pieces_are_all_uppercase() {
        for piece in WHITE_PIECES {
            assert!(
                piece.char().is_uppercase(),
                "{piece:?}.char() should be uppercase"
            );
        }
    }

    #[test]
    fn char_black_pieces_are_all_lowercase() {
        for piece in BLACK_PIECES {
            assert!(
                piece.char().is_lowercase(),
                "{piece:?}.char() should be lowercase"
            );
        }
    }

    #[test]
    fn char_values_are_unique() {
        let chars: Vec<char> = ALL_PIECES.iter().map(|p| p.char()).collect();
        let unique: std::collections::HashSet<char> = chars.iter().copied().collect();
        assert_eq!(
            chars.len(),
            unique.len(),
            "all char() values should be distinct"
        );
    }

    #[test]
    fn char_then_from_char_round_trips() {
        for piece in ALL_PIECES {
            let c = piece.char();
            assert_eq!(
                Piece::from_char(c),
                Some(piece),
                "round-trip failed for {piece:?}"
            );
        }
    }

    #[test]
    fn from_char_then_char_is_identity() {
        let valid_chars = ['P', 'N', 'B', 'R', 'Q', 'K', 'p', 'n', 'b', 'r', 'q', 'k'];
        for c in valid_chars {
            let piece = Piece::from_char(c).unwrap();
            assert_eq!(piece.char(), c);
        }
    }

    #[test]
    fn role_white_pieces() {
        assert_eq!(Piece::WPawn.role(), Role::Pawn);
        assert_eq!(Piece::WKnight.role(), Role::Knight);
        assert_eq!(Piece::WBishop.role(), Role::Bishop);
        assert_eq!(Piece::WRook.role(), Role::Rook);
        assert_eq!(Piece::WQueen.role(), Role::Queen);
        assert_eq!(Piece::WKing.role(), Role::King);
    }

    #[test]
    fn role_black_pieces() {
        assert_eq!(Piece::BPawn.role(), Role::Pawn);
        assert_eq!(Piece::BKnight.role(), Role::Knight);
        assert_eq!(Piece::BBishop.role(), Role::Bishop);
        assert_eq!(Piece::BRook.role(), Role::Rook);
        assert_eq!(Piece::BQueen.role(), Role::Queen);
        assert_eq!(Piece::BKing.role(), Role::King);
    }

    #[test]
    fn same_role_for_white_and_black_counterparts() {
        let pairs = [
            (Piece::WPawn, Piece::BPawn),
            (Piece::WKnight, Piece::BKnight),
            (Piece::WBishop, Piece::BBishop),
            (Piece::WRook, Piece::BRook),
            (Piece::WQueen, Piece::BQueen),
            (Piece::WKing, Piece::BKing),
        ];
        for (white, black) in pairs {
            assert_eq!(
                white.role(),
                black.role(),
                "{white:?} and {black:?} should share a role"
            );
        }
    }

    #[test]
    fn color_white_pieces() {
        for piece in WHITE_PIECES {
            assert_eq!(piece.color(), Color::White, "{piece:?} should be White");
        }
    }

    #[test]
    fn color_black_pieces() {
        for piece in BLACK_PIECES {
            assert_eq!(piece.color(), Color::Black, "{piece:?} should be Black");
        }
    }

    #[test]
    fn white_and_black_counterparts_have_different_colors() {
        let pairs = [
            (Piece::WPawn, Piece::BPawn),
            (Piece::WKnight, Piece::BKnight),
            (Piece::WBishop, Piece::BBishop),
            (Piece::WRook, Piece::BRook),
            (Piece::WQueen, Piece::BQueen),
            (Piece::WKing, Piece::BKing),
        ];
        for (white, black) in pairs {
            assert_ne!(white.color(), black.color());
        }
    }

    #[test]
    fn exactly_six_white_and_six_black_pieces() {
        let white_count = ALL_PIECES
            .iter()
            .filter(|p| p.color() == Color::White)
            .count();
        let black_count = ALL_PIECES
            .iter()
            .filter(|p| p.color() == Color::Black)
            .count();
        assert_eq!(white_count, 6);
        assert_eq!(black_count, 6);
    }

    #[test]
    fn of_then_role_and_color_are_consistent() {
        let roles = [
            Role::Pawn,
            Role::Knight,
            Role::Bishop,
            Role::Rook,
            Role::Queen,
            Role::King,
        ];
        for role in roles {
            for color in [Color::White, Color::Black] {
                let piece = Piece::of(role, color);
                assert_eq!(piece.role(), role, "role mismatch for {piece:?}");
                assert_eq!(piece.color(), color, "color mismatch for {piece:?}");
            }
        }
    }

    #[test]
    fn of_char_role_color_all_consistent() {
        for piece in ALL_PIECES {
            let rebuilt = Piece::of(piece.role(), piece.color());
            assert_eq!(rebuilt, piece, "of(role, color) should rebuild {piece:?}");
        }
    }
}
