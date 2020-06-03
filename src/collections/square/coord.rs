use crate::collections::square::{Dimension, Vector};
use std::fmt;
use std::fmt::Debug;

/// Cartesian coordinates
#[derive(Clone, Copy, PartialEq)]
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

    pub fn get(self, dimension: Dimension) -> usize {
        self.0[dimension as usize]
    }

    pub fn vectors(self) -> [Vector; 2] {
        let col = Vector::col(self.col());
        let row = Vector::row(self.row());
        [col, row]
    }
}

impl Debug for Coord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.col(), self.row())
    }
}

#[cfg(test)]
mod test {
    use crate::collections::square::{Coord, Vector};

    #[test]
    fn test() {
        let coord = Coord::new(1, 2);
        assert_eq!(coord.col(), 1);
        assert_eq!(coord.row(), 2);
        assert_eq!(coord.vectors(), [Vector::col(1), Vector::row(2)]);
    }
}
