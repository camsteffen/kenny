//! Module for rows and columns of a `Square`

use std::fmt;
use std::fmt::Debug;

use self::Dimension::{Col, Row};
use super::{Coord, SquareValue};
use crate::collections::square::SquareIndex;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Dimension {
    Col = 0,
    Row = 1,
}

/// A row or column
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Vector {
    pub dimension: Dimension,
    pub index: SquareValue,
}

impl Vector {
    pub fn new(dimension: Dimension, index: SquareValue) -> Vector {
        Self { dimension, index }
    }

    /// Creates a column Vector
    pub fn col(index: SquareValue) -> Vector {
        Self::new(Col, index)
    }

    /// Creates a row Vector
    pub fn row(index: SquareValue) -> Vector {
        Self::new(Row, index)
    }

    pub fn id(self) -> SquareIndex {
        self.index as SquareIndex * 2 + self.dimension as SquareIndex
    }

    pub fn intersects_coord(self, coord: Coord) -> bool {
        self.index == coord.get(self.dimension)
    }
}

impl Debug for Vector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self.dimension {
            Row => "Row",
            Col => "Col",
        };
        write!(f, "{} {}", label, self.index)
    }
}

#[cfg(test)]
mod tests {
    use crate::collections::square::vector::Dimension::{Col, Row};
    use crate::collections::square::{Coord, Vector};

    #[test]
    fn col() {
        let col = Vector::col(3);
        assert_eq!(col.dimension, Col);
        assert_eq!(col.index, 3);
        assert!(col.intersects_coord(Coord::new(3, 1)));
        assert!(!col.intersects_coord(Coord::new(1, 3)));
    }

    #[test]
    fn row() {
        let row = Vector::row(3);
        assert_eq!(row.dimension, Row);
        assert_eq!(row.index, 3);
        assert!(!row.intersects_coord(Coord::new(3, 1)));
        assert!(row.intersects_coord(Coord::new(1, 3)));
    }
}
