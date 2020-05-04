use std::cmp::Ord;
use std::convert::TryFrom;
use std::fmt::{Debug, Display, Formatter};
use std::fmt;
use std::iter::{Chain, Map};
use std::ops::{Deref, Index, IndexMut, Range};

pub use self::coord::Coord;
pub use self::vector::Dimension;
pub use self::vector::VectorId;

mod coord;
mod vector;

type Vectors = Chain<Map<Range<usize>, fn(usize) -> VectorId>, Map<Range<usize>, fn(usize) -> VectorId>>;

#[derive(Clone, Copy)]
pub struct SquareWidth(usize);

impl From<SquareWidth> for usize {
    fn from(width: SquareWidth) -> Self {
        width.0
    }
}

impl From<usize> for SquareWidth {
    fn from(i: usize) -> Self {
        Self(i)
    }
}

impl<T: IsSquare> From<&T> for SquareWidth {
    fn from(square: &T) -> Self {
        Self(square.width())
    }
}

pub trait IsSquare {
    fn len(&self) -> usize {
        self.width().pow(2)
    }

    fn col_at(&self, index: usize) -> usize {
        assert!(index < self.len());
        index % self.width()
    }

    fn row_at(&self, index: usize) -> usize {
        assert!(index < self.len());
        index / self.width()
    }

    fn coord_at(&self, index: usize) -> Coord {
        Coord::new(self.col_at(index), self.row_at(index))
    }

    fn index_is_in_vector(&self, index: usize, vector_id: VectorId) -> bool {
        let vector_index = match vector_id.dimension() {
            Dimension::Row => self.row_at(index),
            Dimension::Col => self.col_at(index),
        };
        vector_index == vector_id.index()
    }

    fn index_to_vector_point(&self, index: usize, vector_id: VectorId) -> usize {
        assert!(vector_id.index() < self.width());
        match vector_id.dimension() {
            Dimension::Row => self.col_at(index),
            Dimension::Col => self.row_at(index),
        }
    }

    fn shared_vector(&self, a: SquareIndex, b: SquareIndex) -> Option<VectorId> {
        let width = self.width();
        if a / width == b / width {
            Some(VectorId::row(a / width))
        } else if a % width == b % width {
            Some(VectorId::col(a % width))
        } else {
            None
        }
    }

    fn vectors(&self) -> Vectors {
        let cols = (0..self.width()).map(VectorId::col as fn(usize) -> VectorId);
        let rows = (0..self.width()).map(VectorId::row as fn(usize) -> VectorId);
        return cols.chain(rows);
    }

    fn vector_point(&self, vector_id: VectorId, position: usize) -> SquareIndex {
        assert!(vector_id.index() < self.width());
        assert!(position < self.width());
        let coord = match vector_id.dimension() {
            Dimension::Row => Coord::new(position, vector_id.index()),
            Dimension::Col => Coord::new(vector_id.index(), position),
        };
        coord.as_square_index(self.width())
    }

    fn width(&self) -> usize;
}

pub type SquareIndex = usize;

/// A value that can be converted to a [SquareIndex] given the square size
pub trait AsSquareIndex: Copy {
    fn as_square_index(self, width: usize) -> SquareIndex;
}

impl AsSquareIndex for usize {
    fn as_square_index(self, _width: usize) -> SquareIndex {
        self
    }
}

impl AsSquareIndex for Coord {
    fn as_square_index(self, size: usize) -> SquareIndex {
        self.row() * size + self.col()
    }
}

/// A container of elements represented in a square grid
#[derive(Clone, Debug, PartialEq)]
pub struct Square<T> {
    width: usize,
    elements: Vec<T>,
}

impl<T> Square<T> {
    /// Creates a new square with a specified width and fill with the default value
    pub fn with_width(width: usize) -> Square<T>
        where T: Clone + Default {
        Self {
            width,
            elements: vec![Default::default(); width.pow(2)],
        }
    }

    /// Create a new `Square` of a specified width and fill with a specified value
    pub fn with_width_and_value(width: usize, val: T) -> Square<T>
        where T: Clone {
        Square {
            width,
            elements: vec![val; width.pow(2)],
        }
    }

    /// Returns the width (and height) of the grid
    pub fn width(&self) -> usize {
        self.width
    }

    /// Returns an iterator over the rows of the square
    pub fn cols(&self) -> impl Iterator<Item=impl Iterator<Item=&T> + '_> + '_ {
        (0..self.width())
            .map(move |col| (0..self.width())
                .map(move |row| &self[Coord::new(col, row)]))
    }

    /// Returns an iterator over the rows of the square
    pub fn rows(&self) -> impl Iterator<Item=&[T]> {
        self.elements.chunks(self.width)
    }

    /// Returns a mutable iterator over the rows of the square
    pub fn rows_mut(&mut self) -> impl Iterator<Item=&mut [T]> {
        self.elements.chunks_mut(self.width)
    }

    /// Returns an iterator over every element, paired with its `Coord`
    pub fn iter_coord(&self) -> impl Iterator<Item=(Coord, &T)> {
        self.elements.iter().enumerate()
            .map(move |(i, e)| (self.coord_at(i), e))
    }

    pub fn vector(&self, vector_id: VectorId) -> impl Iterator<Item=&T> {
        vector_id.indices(self).map(move |i| &self[i])
    }

    pub fn vector_with_indices(&self, vector_id: VectorId) -> impl Iterator<Item=(usize, &T)> {
        vector_id.indices(self).map(move |i| (i, &self[i]))
    }
}

impl<T> Deref for Square<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.elements
    }
}

impl<T> IsSquare for &Square<T> {
    fn len(&self) -> usize {
        self.elements.len()
    }

    fn width(&self) -> usize {
        self.width
    }
}

impl<T, I: AsSquareIndex> Index<I> for Square<T> {
    type Output = T;

    fn index(&self, index: I) -> &Self::Output {
        &self.elements[usize::from(index.as_square_index(self.width))]
    }
}

impl<T, I: AsSquareIndex> IndexMut<I> for Square<T> {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.elements[usize::from(index.as_square_index(self.width))]
    }
}

impl<T> fmt::Display for Square<T>
    where T: Display + Ord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let len = self.elements.iter().max().unwrap()
            .to_string().len();
        for row in self.rows() {
            for element in row {
                write!(f, "{:>1$} ", element, len)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

pub struct UnitSquare {
    width: usize,
}

impl UnitSquare {
    #[cfg(test)]
    pub fn new(width: usize) -> Self {
        Self { width }
    }
}

impl IsSquare for UnitSquare {
    fn width(&self) -> usize {
        self.width
    }
}

#[derive(PartialEq)]
pub struct NonSquareLength(usize);

impl Debug for NonSquareLength {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "The length of elements ({}) is not square", self.0)
    }
}

impl<T> TryFrom<Vec<T>> for Square<T> {
    type Error = NonSquareLength;

    fn try_from(elements: Vec<T>) -> Result<Self, Self::Error> {
        let width = (elements.len() as f32).sqrt() as usize;
        if elements.len() != width.pow(2) {
            return Err(NonSquareLength(elements.len()));
        }
        Ok(Self { width, elements })
    }
}


#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use crate::collections::Square;
    use crate::collections::square::NonSquareLength;

    #[test]
    fn try_from_vec() {
        assert!(Square::try_from(vec![1; 9]).is_ok())
    }

    #[test]
    fn try_from_non_square_vec() {
        assert_eq!(Err(NonSquareLength(8)), Square::try_from(vec![1; 8]))
    }

    mod is_square {
        use crate::collections::square::{IsSquare, UnitSquare, VectorId};

        #[test]
        fn index_to_vector_point() {
            assert_eq!(2, UnitSquare::new(3).index_to_vector_point(7, VectorId::col(1)));
        }

        #[test]
        fn vector_point() {
            assert_eq!(8, UnitSquare::new(3).vector_point(VectorId::row(2), 2));
        }
    }
}
