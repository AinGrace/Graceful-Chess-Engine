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
