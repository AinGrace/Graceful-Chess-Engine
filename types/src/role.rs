#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl Role {
    pub fn new(chr: char) -> Option<Self> {
        match chr {
            'p' | 'P' => Some(Self::Pawn),
            'n' | 'N' => Some(Self::Knight),
            'b' | 'B' => Some(Self::Bishop),
            'r' | 'R' => Some(Self::Rook),
            'q' | 'Q' => Some(Self::Queen),
            'k' | 'K' => Some(Self::King),
            _unknown => None,
        }
    }

    pub fn char(self) -> char {
        match self {
            Role::Pawn => 'p',
            Role::Knight => 'n',
            Role::Bishop => 'b',
            Role::Rook => 'r',
            Role::Queen => 'q',
            Role::King => 'k',
        }
    }
}
