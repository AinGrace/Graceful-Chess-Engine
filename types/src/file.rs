use std::mem::transmute;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum File {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    F = 5,
    G = 6,
    H = 7,
}

impl File {
    pub const fn to_usize(self) -> usize {
        self as usize
    }

    pub const fn to_u32(self) -> u32 {
        self as u32
    }

    pub fn from_char(chr: char) -> Option<Self> {
        match chr {
            'A' | 'a' => Some(Self::A),
            'B' | 'b' => Some(Self::B),
            'C' | 'c' => Some(Self::C),
            'D' | 'd' => Some(Self::D),
            'E' | 'e' => Some(Self::E),
            'F' | 'f' => Some(Self::F),
            'G' | 'g' => Some(Self::G),
            'H' | 'h' => Some(Self::H),
            _ => None,
        }
    }

    pub fn char(self) -> char {
        match self {
            File::A => 'a',
            File::B => 'b',
            File::C => 'c',
            File::D => 'd',
            File::E => 'e',
            File::F => 'f',
            File::G => 'g',
            File::H => 'h',
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
