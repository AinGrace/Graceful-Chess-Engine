use crate::{
    role::Role::{self, Pawn},
    square::Square,
};

pub enum CastlingSide {
    WShort,
    WLong,
    BShort,
    BLong,
}

/// TODO: use bits to reduce the memory usage, as in the Castlings struct
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Standard {
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
        Self::Standard {
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
        Self::Standard {
            role,
            from,
            to,
            capture: None,
            promotion: None,
        }
    }

    pub fn capture(role: Role, from: Square, to: Square, capture: Role) -> Self {
        Self::Standard {
            role,
            from,
            to,
            capture: Some(capture),
            promotion: None,
        }
    }

    pub fn promotion(from: Square, to: Square, promotion: Role) -> Self {
        Self::Standard {
            role: Role::Pawn,
            from,
            to,
            capture: None,
            promotion: Some(promotion),
        }
    }

    pub fn capture_promotion(from: Square, to: Square, capture: Role, promotion: Role) -> Self {
        Self::Standard {
            role: Role::Pawn,
            from,
            to,
            capture: Some(capture),
            promotion: Some(promotion),
        }
    }

    pub fn from(&self) -> Square {
        match self {
            Move::Standard { from, .. } => *from,
            Move::EnPassant { from, .. } => *from,
            Move::Castling { king, .. } => *king,
        }
    }

    pub fn to(&self) -> Square {
        match self {
            Move::Standard { to, .. } => *to,
            Move::EnPassant { to, .. } => *to,
            Move::Castling { rook, .. } => *rook,
        }
    }

    pub fn role(&self) -> Role {
        match self {
            Move::Standard {
                role,
                from,
                to,
                capture,
                promotion,
            } => *role,
            Move::EnPassant { from, to } => Pawn,
            Move::Castling { king, rook } => Role::King,
        }
    }

    #[rustfmt::skip]
    pub fn to_uci(&self) -> String {
        let (from, to, maybe_prom) = match *self {
            Move::Standard { from, to, promotion, .. } => (from, to, promotion),
            Move::EnPassant { from, to } => (from, to, None),
            Move::Castling { king, rook } if matches!(rook, Square::A1 | Square::A8) => {
                (king, rook.offset_checked(2), None)
            }
            Move::Castling { king, rook } if matches!(rook, Square::H1 | Square::H8) => {
                (king, rook.offset_checked(-1), None)
            }
            Move::Castling { .. } => unreachable!("all valid possibilities are handled above"),
        };

        // UCI move is always 4 or 5 ASCII bytes long
        let mut buffer = String::with_capacity(5);

        buffer.push(from.file().char());
        buffer.push(from.rank().char());
        buffer.push(to.file().char());
        buffer.push(to.rank().char());

        if let Some(prom) = maybe_prom {
            buffer.push(prom.char());
        }

        buffer
    }
}
