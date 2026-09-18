use std::{fmt::Display, ops::{Neg, Sub}};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Score {
    Centipawn(i16),
    Mate(i8),
    Draw,
    Abort,
}

impl Score {
    pub fn value(self) -> i16 {
        match self {
            Score::Mate(val) if val > 0 => 10_000 - val as i16,
            Score::Mate(val) => -10_000 - val as i16,
            Score::Centipawn(val) => val,
            Score::Draw => 0,
            Score::Abort => 0,
        }
    }

    pub fn step(self) -> Self {
        match self {
            Score::Mate(val) if val >= 0 => Self::Mate(val + 1),
            Score::Mate(val) => Self::Mate(val - 1),
            rest => rest,
        }
    }
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value().cmp(&other.value())
    }
}

impl Neg for Score {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Score::Centipawn(val) => Score::Centipawn(-val),
            Score::Mate(val) => Score::Mate(-val),
            Score::Draw => Score::Draw,
            Score::Abort => Score::Abort,
        }
    }
}

impl Sub for Score {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Centipawn(self.value() - rhs.value())
    }
}

impl Sub<i16> for Score {
    type Output = Self;

    fn sub(self, rhs: i16) -> Self::Output {
        Self::Centipawn(self.value() - rhs)
    }
}

impl Display for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Score::Centipawn(val) => format!("cp {}", val),
                Score::Mate(val) => format!("mate {}", val),
                Score::Draw => format!("cp 0"),
                Score::Abort => format!("ABORTED"),
            }
        )
    }
}

impl Default for Score {
    fn default() -> Self {
        Self::Mate(-1)
    }
}
