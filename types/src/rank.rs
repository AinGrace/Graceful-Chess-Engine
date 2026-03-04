use std::mem::transmute;

#[rustfmt::skip]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Rank {
    First   = 0,
    Second  = 1,
    Third   = 2,
    Fourth  = 3,
    Fifth   = 4,
    Sixth   = 5,
    Seventh = 6,
    Eighth  = 7,
}

impl Rank {
    pub const fn to_u32(self) -> u32 {
        self as u32
    }

    pub fn from_char(chr: char) -> Option<Self> {
        let idx = chr.to_digit(10)?;
        Self::new(idx - 1)
    }

    pub fn char(self) -> char {
        match self {
            Rank::First => '1',
            Rank::Second => '2',
            Rank::Third => '3',
            Rank::Fourth => '4',
            Rank::Fifth => '5',
            Rank::Sixth => '6',
            Rank::Seventh => '7',
            Rank::Eighth => '8',
        }
    }

    pub const fn new(index: u32) -> Option<Self> {
        if index > 7 {
            None
        } else {
            Some(Self::new_checked(index))
        }
    }

    /// Will panic if index is >= 8
    pub const fn new_checked(index: u32) -> Self {
        assert!(index < 8);

        // SAFETY: index is always at valid range
        unsafe { transmute(index as u8) }
    }
}
