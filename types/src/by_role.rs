use crate::{bitboard::Bitboard, role::Role, square::Square};

#[derive(Clone, Copy)]
pub struct ByRole {
    pawn: Bitboard,
    knight: Bitboard,
    bishop: Bitboard,
    rook: Bitboard,
    queen: Bitboard,
    king: Bitboard,
}

impl ByRole {
    pub fn new(
        pawn: Bitboard,
        knight: Bitboard,
        bishop: Bitboard,
        rook: Bitboard,
        queen: Bitboard,
        king: Bitboard,
    ) -> Self {
        Self {
            pawn,
            knight,
            bishop,
            rook,
            queen,
            king,
        }
    }

    pub fn get(&self, role: Role) -> Bitboard {
        match role {
            Role::Pawn => self.pawn,
            Role::Knight => self.knight,
            Role::Bishop => self.bishop,
            Role::Rook => self.rook,
            Role::Queen => self.queen,
            Role::King => self.king,
        }
    }

    pub fn get_mut(&mut self, role: Role) -> &mut Bitboard {
        match role {
            Role::Pawn => &mut self.pawn,
            Role::Knight => &mut self.knight,
            Role::Bishop => &mut self.bishop,
            Role::Rook => &mut self.rook,
            Role::Queen => &mut self.queen,
            Role::King => &mut self.king,
        }
    }

    pub fn pawns(&self) -> Bitboard {
        self.pawn
    }

    pub fn pawns_mut(&mut self) -> &mut Bitboard {
        &mut self.pawn
    }

    pub fn knights(&self) -> Bitboard {
        self.knight
    }

    pub fn knights_mut(&mut self) -> &mut Bitboard {
        &mut self.knight
    }

    pub fn bishops(&self) -> Bitboard {
        self.bishop
    }

    pub fn bishops_mut(&mut self) -> &mut Bitboard {
        &mut self.bishop
    }

    pub fn rooks(&self) -> Bitboard {
        self.rook
    }

    pub fn rooks_mut(&mut self) -> &mut Bitboard {
        &mut self.rook
    }

    pub fn queens(&self) -> Bitboard {
        self.queen
    }

    pub fn queens_mut(&mut self) -> &mut Bitboard {
        &mut self.queen
    }

    pub const fn kings(&self) -> Bitboard {
        self.king
    }

    pub fn kings_mut(&mut self) -> &mut Bitboard {
        &mut self.king
    }

    pub fn peek_role(&self, square: Square) -> Option<Role> {
        match self {
            Self { pawn, .. } if pawn.is_square_set(square) => Some(Role::Pawn),
            Self { knight, .. } if knight.is_square_set(square) => Some(Role::Knight),
            Self { bishop, .. } if bishop.is_square_set(square) => Some(Role::Bishop),
            Self { rook, .. } if rook.is_square_set(square) => Some(Role::Rook),
            Self { queen, .. } if queen.is_square_set(square) => Some(Role::Queen),
            Self { king, .. } if king.is_square_set(square) => Some(Role::King),
            _ => None,
        }
    }

    pub fn peek_role_checked(&self, square: Square) -> Role {
        if self.pawn.is_square_set(square) {
            return Role::Pawn;
        }
        if self.knight.is_square_set(square) {
            return Role::Knight;
        }
        if self.bishop.is_square_set(square) {
            return Role::Bishop;
        }
        if self.rook.is_square_set(square) {
            return Role::Rook;
        }
        if self.queen.is_square_set(square) {
            return Role::Queen;
        }
        if self.king.is_square_set(square) {
            return Role::King;
        }

        unreachable!("peek_role_checked on empty square")
    }
}
