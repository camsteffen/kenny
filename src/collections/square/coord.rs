use std::fmt;
use std::fmt::Display;
use std::ops::Deref;
use std::ops::DerefMut;
use super::SquareIndex;

/// A `Coord` struct represents coordinates of an element in a `Square`.
#[derive(Clone, Copy)]
pub struct Coord(pub [usize; 2]);

impl Coord {

    /// Convert to the index of a `Square`.
    pub fn as_index(&self, size: usize) -> SquareIndex {
        SquareIndex(self[0] * size + self[1])
    }
}

impl Deref for Coord {
    type Target = [usize; 2];

    fn deref(&self) -> &[usize; 2] {
        &self.0
    }
}

impl DerefMut for Coord {
    fn deref_mut(&mut self) -> &mut [usize; 2] {
        &mut self.0
    }
}

impl Display for Coord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.0[0], self.0[1])
    }
}

