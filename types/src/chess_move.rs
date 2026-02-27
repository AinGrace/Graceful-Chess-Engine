use crate::{role::Role, square::Square};

pub enum CastlingSide {
    WShort,
    WLong,
    BShort,
    BLong,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Standart {
        role: Role,
        from: Square,
        to: Square,
        capture: Option<Role>,
        promotion: Option<Role>,
    },

    EnPassant {
        from: Square,
        to: Square,
    },

    Castling {
        king: Square,
        rook: Square,
    },
}

impl Move {
    pub fn standart(
        role: Role,
        from: Square,
        to: Square,
        capture: Option<Role>,
        promotion: Option<Role>,
    ) -> Self {
        Self::Standart {
            role,
            from,
            to,
            capture,
            promotion,
        }
    }

    pub fn castling(castling: CastlingSide) -> Self {
        match castling {
            CastlingSide::WShort => Self::Castling {
                king: Square::E1,
                rook: Square::H1,
            },
            CastlingSide::WLong => Self::Castling {
                king: Square::E1,
                rook: Square::A1,
            },
            CastlingSide::BShort => Self::Castling {
                king: Square::E8,
                rook: Square::H8,
            },
            CastlingSide::BLong => Self::Castling {
                king: Square::E8,
                rook: Square::A8,
            },
        }
    }

    pub fn quiet(role: Role, from: Square, to: Square) -> Self {
        Self::Standart {
            role,
            from,
            to,
            capture: None,
            promotion: None,
        }
    }

    pub fn capture(role: Role, from: Square, to: Square, capture: Role) -> Self {
        Self::Standart {
            role,
            from,
            to,
            capture: Some(capture),
            promotion: None,
        }
    }

    pub fn promotion(from: Square, to: Square, promotion: Role) -> Self {
        Self::Standart {
            role: Role::Pawn,
            from,
            to,
            capture: None,
            promotion: Some(promotion),
        }
    }

    pub fn capture_promotion(from: Square, to: Square, capture: Role, promotion: Role) -> Self {
        Self::Standart {
            role: Role::Pawn,
            from,
            to,
            capture: Some(capture),
            promotion: Some(promotion),
        }
    }

    pub fn from(&self) -> Square {
        match self {
            Move::Standart { from, .. } => *from,
            Move::EnPassant { from, .. } => *from,
            Move::Castling { king, .. } => *king,
        }
    }

    pub fn to(&self) -> Square {
        match self {
            Move::Standart { to, .. } => *to,
            Move::EnPassant { to, .. } => *to,
            Move::Castling { rook, .. } => *rook,
        }
    }

    pub fn to_uci(&self) -> String {
        let (from, to, promotion) = match *self {
            Move::Standart {
                from,
                to,
                promotion,
                ..
            } => (from, to, promotion),
            Move::EnPassant { from, to } => (from, to, None),
            Move::Castling { king, rook } => {
                let to = match rook {
                    Square::A1 | Square::A8 => rook.offset_checked(2),
                    Square::H1 | Square::H8 => rook.offset_checked(-2),
                    _ => unreachable!(),
                };
                (king, to, None)
            }
        };

        // UCI move is always 4 or 5 ASCII bytes long
        let mut s = String::with_capacity(5);

        s.push(from.file().char());
        s.push(from.rank().char());
        s.push(to.file().char());
        s.push(to.rank().char());

        if let Some(p) = promotion {
            s.push(p.char());
        }

        s
    }
}
