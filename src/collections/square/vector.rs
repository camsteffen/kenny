//! Module for rows and columns of a `Square`

use self::Dimension::{Col, Row};
use super::Coord;
use std::fmt;
use std::fmt::Debug;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Dimension {
    Col = 0,
    Row = 1,
}

/// A row or column
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Vector(usize);

impl Vector {
    pub fn new(dimension: Dimension, index: usize) -> Vector {
        Vector(index * 2 + dimension as usize)
    }

    /// Creates a column Vector
    pub fn col(index: usize) -> Vector {
        Self::new(Col, index)
    }

    /// Creates a row Vector
    pub fn row(index: usize) -> Vector {
        Self::new(Row, index)
    }
}

pub(crate) trait AsVector: Copy {
    fn id(self) -> usize;

    fn dimension(self) -> Dimension {
        if self.id() % 2 == 0 {
            Col
        } else {
            Row
        }
    }

    /// Retrieves the index of the vector in its respective dimension
    fn index(self) -> usize {
        self.id() / 2
    }

    fn intersects_coord(self, coord: Coord) -> bool {
        self.index() == coord.get(self.dimension())
    }
}

impl AsVector for Vector {
    #[inline]
    fn id(self) -> usize {
        self.0
    }
}

impl Debug for Vector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self.dimension() {
            Row => "Row",
            Col => "Col",
        };
        write!(f, "{} {}", label, self.index())
    }
}

#[cfg(test)]
mod test {
    use crate::collections::square::vector::Dimension::{Col, Row};
    use crate::collections::square::{AsVector, Coord, Vector};

    #[test]
    fn col() {
        let col = Vector::col(3);
        assert_eq!(col.dimension(), Col);
        assert_eq!(col.index(), 3);
        assert!(col.intersects_coord(Coord::new(3, 1)));
        assert!(!col.intersects_coord(Coord::new(1, 3)));
    }

    #[test]
    fn row() {
        let row = Vector::row(3);
        assert_eq!(row.dimension(), Row);
        assert_eq!(row.index(), 3);
        assert!(!row.intersects_coord(Coord::new(3, 1)));
        assert!(row.intersects_coord(Coord::new(1, 3)));
    }
}
