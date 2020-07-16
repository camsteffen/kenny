use std::fmt;
use std::fmt::{Debug, Display};

use crate::collections::square::{Dimension, Vector};

/// Cartesian coordinates
#[derive(Clone, Copy, PartialEq)]
pub struct Coord<T: Copy = usize>([T; 2]);

impl<T: Copy> Coord<T> {
    pub fn new(col: T, row: T) -> Self {
        Self([col, row])
    }

    pub fn col(self) -> T {
        self.0[0]
    }

    pub fn row(self) -> T {
        self.0[1]
    }

    pub fn get(self, dimension: Dimension) -> T {
        self.0[dimension as usize]
    }

    pub fn as_array(self) -> [T; 2] {
        self.0
    }

    pub fn as_tuple(self) -> (T, T) {
        self.into()
    }

    pub fn transpose(self) -> Coord<T> {
        Coord([self.0[1], self.0[0]])
    }
}

impl Coord<usize> {
    pub fn vectors(self) -> [Vector; 2] {
        let col = Vector::col(self.col());
        let row = Vector::row(self.row());
        [col, row]
    }
}

impl<T> Debug for Coord<T>
where
    T: Copy + Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.col(), self.row())
    }
}

impl<T: Copy> From<Coord<T>> for (T, T) {
    fn from(coord: Coord<T>) -> Self {
        (coord.0[0], coord.0[1])
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
