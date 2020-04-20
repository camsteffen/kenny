//! Module for rows and columns of a `Square`

use std::fmt;
use std::fmt::Debug;
use super::SquareIndex;
use self::Dimension::{Col, Row};
use super::Coord;
use crate::collections::square::AsSquareIndex;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Dimension { Row, Col }

/// A row or column and its index
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VectorId(pub usize);

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

    /// Retrives the index of the vector in its respective dimension
    pub fn index(self) -> usize {
        self.0 / 2
    }

    pub fn contains_sq_index(self, index: SquareIndex, square_width: usize) -> bool {
        let index = match self.dimension() {
            Row => index.row(square_width),
            Col => index.col(square_width),
        };
        index == self.index()
    }

    /// Creates an iterator over the positions of the cells in this vector with respect to the square
    pub fn iter_sq_pos(self, size: usize) -> impl Iterator<Item=SquareIndex> {
        (0..size).map(move |n| self.vec_pos_to_sq_pos(n, size))
    }

    /// Retrieves the vector that intersects this vector at a given position
    pub fn intersect_at(self, index: usize) -> VectorId {
        match self.dimension() {
            Row => Self::col(index),
            Col => Self::row(index),
        }
    }

    /// Calculates the position of a cell with respect to a vector, given the position of the cell with respect to the square.
    pub fn sq_pos_to_vec_pos(self, pos: SquareIndex, size: usize) -> usize {
        match self.dimension() {
            Row => pos.col(size),
            Col => pos.row(size),
        }
    }

    /// Calculates the position of a cell with respect to a square, given the position of the cell with respect to a vector.
    pub fn vec_pos_to_sq_pos(self, pos: usize, size: usize) -> SquareIndex {
        debug_assert!(pos < size);
        let coord = match self.dimension() {
            Row => [self.index(), pos],
            Col => [pos, self.index()],
        };
        Coord(coord).as_index(size)
    }
}

impl Debug for VectorId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
