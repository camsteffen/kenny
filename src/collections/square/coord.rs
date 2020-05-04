use crate::collections::square::VectorId;
use std::fmt;
use std::fmt::Debug;

/// A `Coord` struct represents coordinates of an element in a `Square`.
#[derive(Clone, Copy)]
pub struct Coord([usize; 2]);

impl Coord {
    pub fn new(col: usize, row: usize) -> Self {
        Self([col, row])
    }

    pub fn col(self) -> usize {
        self.0[0]
    }

    pub fn row(self) -> usize {
        self.0[1]
    }

    pub fn vectors(self) -> [VectorId; 2] {
        [VectorId::row(self.row()), VectorId::col(self.col())]
    }
}

impl Debug for Coord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.col(), self.row())
    }
}

impl From<[usize; 2]> for Coord {
    fn from(array: [usize; 2]) -> Self {
        Self(array)
    }
}
