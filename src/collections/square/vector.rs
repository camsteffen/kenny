//! Module for rows and columns of a `Square`

use std::fmt;
use std::fmt::Debug;
use self::Dimension::{Col, Row};
use super::Coord;
use crate::collections::square::{IsSquare, SquareIndex};
use std::iter::StepBy;
use std::ops::Range;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Dimension { Row, Col }

/// A row or column and its index
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VectorId(usize);

type VectorIndices = StepBy<Range<SquareIndex>>;

impl VectorId {
    
    /// Creates a column VectorId
    pub fn col(index: usize) -> VectorId {
        VectorId(index * 2 + 1)
    }
    
    /// Creates a row VectorId
    pub fn row(index: usize) -> VectorId {
        VectorId(index * 2)
    }

    pub fn dimension(self) -> Dimension {
        if self.0 % 2 == 0 { Row } else { Col }
    }

    /// Retrieves the index of the vector in its respective dimension
    pub fn index(self) -> usize {
        self.0 / 2
    }

    pub fn indices(self, square: impl IsSquare) -> VectorIndices {
        let width = square.width();
        assert!(self.index() < width);
        let (start, end, step) = match self.dimension() {
            Dimension::Row => (
                width * self.index(),
                width * (self.index() + 1),
                1
            ),
            Dimension::Col => (
                self.index(),
                self.index() + square.len(),
                width
            ),
        };
        (start..end).step_by(step)
    }

    pub fn intersects_coord(self, coord: Coord) -> bool {
        self.index() == match self.dimension() {
            Row => coord.row(),
            Col => coord.col(),
        }
    }
}

impl Debug for VectorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self.dimension() {
            Row => "Row",
            Col => "Col",
        };
        write!(f, "{} {}", label, self.index())
    }
}

impl From<VectorId> for usize {
    fn from(vector_id: VectorId) -> Self {
        vector_id.0
    }
}

#[cfg(test)]
mod test {
    use crate::collections::square::{UnitSquare, VectorId};

    #[test]
    fn indices() {
        assert_eq!(
            vec![0, 3, 6],
            VectorId::col(0)
                .indices(UnitSquare::new(3))
                .map(usize::from)
                .collect::<Vec<usize>>());
    }
}
