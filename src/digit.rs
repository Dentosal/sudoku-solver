use std::fmt;

/// A single digit in a Sudoku puzzle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Digit(u8);

impl Digit {
    pub const MIN: Self = Self(1);
    pub const MAX: Self = Self(9);

    pub fn new(value: u8) -> Option<Self> {
        if (Self::MIN.0..=Self::MAX.0).contains(&value) {
            Some(Self(value))
        } else {
            None
        }
    }

    pub fn unchecked(value: u8) -> Self {
        debug_assert!((Self::MIN.0..=Self::MAX.0).contains(&value));
        Self(value)
    }

    /// Index 0..9 for the digit
    pub fn index(self) -> u8 {
        self.0 - 1
    }

    pub fn from_index(value: u8) -> Option<Self> {
        Self::new(value + 1)
    }

    pub fn next(self) -> Option<Self> {
        if self.0 < Self::MAX.0 {
            Some(Self(self.0 + 1))
        } else {
            None
        }
    }
}

impl fmt::Display for Digit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
