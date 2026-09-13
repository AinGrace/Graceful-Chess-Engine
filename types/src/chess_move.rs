use std::fmt::Debug;

use crate::{role::Role, square::Square};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Move(u16);

const FROM_SHIFT: u16 = 0;
const TO_SHIFT: u16 = 6;
const FLAG_SHIFT: u16 = 12;

const SQ_MASK: u16 = 0b0011_1111;
const FLAG_MASK: u16 = 0b1111;

#[repr(u16)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MoveFlag {
    Quiet = 0,
    DoublePush = 1,
    KingCastle = 2,
    QueenCastle = 3,
    Capture = 4,
    EnPassant = 5,
    // 6, 7 unused
    PromoN = 8,
    PromoB = 9,
    PromoR = 10,
    PromoQ = 11,
    PromoCapN = 12,
    PromoCapB = 13,
    PromoCapR = 14,
    PromoCapQ = 15,
}

impl MoveFlag {
    fn from_bits(bits: u16) -> Self {
        // Safety: only 0..=5 and 8..=15 are ever written; 6/7 never occur
        unsafe { std::mem::transmute(bits) }
    }

    pub fn is_promotion(self) -> bool {
        (self as u16) >= 8
    }

    pub fn is_capture(self) -> bool {
        matches!(
            self,
            MoveFlag::Capture
                | MoveFlag::EnPassant
                | MoveFlag::PromoCapN
                | MoveFlag::PromoCapB
                | MoveFlag::PromoCapR
                | MoveFlag::PromoCapQ
        )
    }

    /// Role::Knight/Bishop/Rook/Queen for promotions, panics otherwise
    pub fn promo_role(self) -> Role {
        use crate::role::Role;
        match self {
            MoveFlag::PromoN | MoveFlag::PromoCapN => Role::Knight,
            MoveFlag::PromoB | MoveFlag::PromoCapB => Role::Bishop,
            MoveFlag::PromoR | MoveFlag::PromoCapR => Role::Rook,
            MoveFlag::PromoQ | MoveFlag::PromoCapQ => Role::Queen,
            _ => unreachable!("promo_role called on non-promotion move"),
        }
    }
}

impl Move {
    fn pack(from: Square, to: Square, flag: MoveFlag) -> Self {
        Move(
            (from as u16 & SQ_MASK) << FROM_SHIFT
                | (to as u16 & SQ_MASK) << TO_SHIFT
                | (flag as u16) << FLAG_SHIFT,
        )
    }

    pub fn quiet(from: Square, to: Square) -> Self {
        Self::pack(from, to, MoveFlag::Quiet)
    }

    pub fn double_push(from: Square, to: Square) -> Self {
        Self::pack(from, to, MoveFlag::DoublePush)
    }

    pub fn capture(from: Square, to: Square) -> Self {
        Self::pack(from, to, MoveFlag::Capture)
    }

    pub fn en_passant(from: Square, to: Square) -> Self {
        Self::pack(from, to, MoveFlag::EnPassant)
    }

    pub fn king_castle(king_from: Square, king_to: Square) -> Self {
        Self::pack(king_from, king_to, MoveFlag::KingCastle)
    }

    pub fn queen_castle(king_from: Square, king_to: Square) -> Self {
        Self::pack(king_from, king_to, MoveFlag::QueenCastle)
    }

    pub fn promotion(from: Square, to: Square, promo: Role, is_capture: bool) -> Self {
        use crate::role::Role;
        let flag = match (promo, is_capture) {
            (Role::Knight, false) => MoveFlag::PromoN,
            (Role::Bishop, false) => MoveFlag::PromoB,
            (Role::Rook, false) => MoveFlag::PromoR,
            (Role::Queen, false) => MoveFlag::PromoQ,
            (Role::Knight, true) => MoveFlag::PromoCapN,
            (Role::Bishop, true) => MoveFlag::PromoCapB,
            (Role::Rook, true) => MoveFlag::PromoCapR,
            (Role::Queen, true) => MoveFlag::PromoCapQ,
            _ => unreachable!("invalid promotion role"),
        };
        Self::pack(from, to, flag)
    }

    pub fn from(&self) -> Square {
        unsafe { Square::from_u32_unchecked(((self.0 >> FROM_SHIFT) & SQ_MASK) as u32) }
    }

    pub fn to(&self) -> Square {
        unsafe { Square::from_u32_unchecked(((self.0 >> TO_SHIFT) & SQ_MASK) as u32) }
    }

    pub fn flag(&self) -> MoveFlag {
        MoveFlag::from_bits((self.0 >> FLAG_SHIFT) & FLAG_MASK)
    }

    pub fn is_capture(&self) -> bool {
        self.flag().is_capture()
    }

    pub fn is_promotion(&self) -> bool {
        self.flag().is_promotion()
    }

    pub fn is_castling(&self) -> bool {
        matches!(self.flag(), MoveFlag::KingCastle | MoveFlag::QueenCastle)
    }

    #[rustfmt::skip]
    pub fn to_uci(&self) -> String {
        let (from, to) = (self.from(), self.to());
        let mut buffer = String::with_capacity(5);
        buffer.push(from.file().char());
        buffer.push(from.rank().char());
        buffer.push(to.file().char());
        buffer.push(to.rank().char());
        if self.flag().is_promotion() {
            buffer.push(self.flag().promo_role().char());
        }
        buffer
    }
}

impl Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Move").field(&format!("m => {} | f => {:?}", self.to_uci(), self.flag())).finish()
    }
}
