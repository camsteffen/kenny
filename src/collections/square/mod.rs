use std::cmp::Ord;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::iter::{Chain, Map, StepBy};
use std::ops::{Deref, Index, IndexMut, Range};

pub(crate) use self::coord::Coord;
pub(crate) use self::vector::{Dimension, Vector};

mod coord;
mod vector;

type VectorsInner = Map<Range<SquareValue>, fn(SquareValue) -> Vector>;
type Vectors = Chain<VectorsInner, VectorsInner>;
type VectorIndices = StepBy<Range<SquareIndex>>;

pub(crate) trait IsSquare {
    fn len(&self) -> usize {
        let width = self.width() as usize;
        width * width
    }

    fn cell(&self, index: impl AsSquareIndex) -> SquareCellRef<'_, Self>
    where
        Self: Sized,
    {
        let index = index.as_square_index(self.width());
        self.assert_index(index);
        SquareCellRef {
            square: self,
            index,
        }
    }

    fn shared_vector(&self, a: SquareIndex, b: SquareIndex) -> Option<Vector> {
        self.assert_index(a);
        self.assert_index(b);
        let width = self.width() as SquareIndex;
        if a / width == b / width {
            Some(Vector::row((a / width) as SquareValue))
        } else if a % width == b % width {
            Some(Vector::col((a % width) as SquareValue))
        } else {
            None
        }
    }

    fn square_vectors(&self) -> SquareVectorsIter<'_, Self>
    where
        Self: Sized,
    {
        SquareVectorsIter {
            square: self,
            vectors: self.vectors(),
        }
    }

    fn vector(&self, vector: Vector) -> SquareVector<'_, Self>
    where
        Self: Sized,
    {
        self.assert_vector(vector);
        SquareVector {
            square: self,
            vector,
        }
    }

    fn vectors(&self) -> Vectors {
        let as_col: fn(SquareValue) -> Vector = Vector::col;
        let as_row: fn(SquareValue) -> Vector = Vector::row;
        let cols = (0..self.width()).map(as_col);
        let rows = (0..self.width()).map(as_row);
        cols.chain(rows)
    }

    fn width(&self) -> SquareValue;
}

pub(crate) struct SquareCellRef<'a, S: IsSquare> {
    square: &'a S,
    index: SquareIndex,
}

// Clone and Copy cannot be derived - see https://github.com/rust-lang/rust/issues/26925
impl<S: IsSquare> Clone for SquareCellRef<'_, S> {
    fn clone(&self) -> Self {
        Self {
            square: self.square,
            index: self.index,
        }
    }
}

impl<S: IsSquare> Copy for SquareCellRef<'_, S> {}

impl<'a, S: IsSquare> SquareCellRef<'a, S> {
    pub fn square(self) -> &'a S {
        self.square
    }

    pub fn index(self) -> usize {
        self.index
    }

    pub fn col(self) -> SquareValue {
        (self.index % self.square.width() as SquareIndex) as SquareValue
    }

    pub fn row(self) -> SquareValue {
        (self.index / self.square.width() as SquareIndex) as SquareValue
    }

    pub fn coord(self) -> Coord<SquareValue> {
        Coord::new(self.col(), self.row())
    }

    pub fn dimension_index(self, dimension: Dimension) -> SquareValue {
        match dimension {
            Dimension::Col => self.col(),
            Dimension::Row => self.row(),
        }
    }

    pub fn is_in_vector(self, vector: Vector) -> bool {
        self.square.vector(vector).contains_square_index(self.index)
    }

    pub fn vectors(self) -> [Vector; 2] {
        self.coord().vectors()
    }
}

pub(crate) struct SquareVectorsIter<'a, S> {
    square: &'a S,
    vectors: Vectors,
}

impl<'a, S> Iterator for SquareVectorsIter<'a, S>
where
    S: IsSquare,
{
    type Item = SquareVector<'a, S>;

    fn next(&mut self) -> Option<Self::Item> {
        let vector = self.vectors.next()?;
        Some(self.square.vector(vector))
    }
}

trait IsSquarePrivate {
    fn assert_index(&self, index: usize);
    fn assert_vector(&self, vector: Vector);
}

impl<T: IsSquare + ?Sized> IsSquarePrivate for T {
    #[inline]
    fn assert_index(&self, index: usize) {
        assert!(index < self.len());
    }

    #[inline]
    fn assert_vector(&self, vector: Vector) {
        assert!(vector.index < self.width());
    }
}

pub type SquareIndex = usize;

/// Represents a width or row/column number within a `Square`
pub type SquareValue = u32;

/// A value that can be converted to a `SquareIndex` given the square size
pub trait AsSquareIndex: Copy {
    fn as_square_index(self, width: SquareValue) -> SquareIndex;
}

impl AsSquareIndex for usize {
    #[inline]
    fn as_square_index(self, _width: SquareValue) -> SquareIndex {
        self
    }
}

impl AsSquareIndex for Coord {
    fn as_square_index(self, size: SquareValue) -> SquareIndex {
        assert!(self.col() < size);
        assert!(self.row() < size);
        self.row() as SquareIndex * size as SquareIndex + self.col() as SquareIndex
    }
}

pub(crate) struct SquareVector<'a, T> {
    pub(crate) square: &'a T,
    vector: Vector,
}

// Clone and Copy cannot be derived - see https://github.com/rust-lang/rust/issues/26925
impl<'a, T> Clone for SquareVector<'a, T> {
    fn clone(&self) -> Self {
        Self {
            square: self.square,
            vector: self.vector,
        }
    }
}

impl<'a, T> Copy for SquareVector<'a, T> {}

impl<'a, T> SquareVector<'a, T>
where
    T: IsSquare,
{
    pub fn contains_square_index(&self, index: usize) -> bool {
        self.vector.index
            == self
                .square
                .cell(index)
                .dimension_index(self.vector.dimension)
    }

    pub fn indices(self) -> VectorIndices {
        let width = self.square.width() as SquareIndex;
        let (start, end, step) = match self.vector.dimension {
            Dimension::Row => (
                width * self.vector.index as SquareIndex,
                width * (self.vector.index as SquareIndex + 1),
                1,
            ),
            Dimension::Col => (
                self.vector.index as SquareIndex,
                self.vector.index as SquareIndex + self.square.len(),
                width,
            ),
        };
        (start..end).step_by(step)
    }

    pub fn square_index_at(self, index: SquareValue) -> SquareIndex {
        let coord = match self.vector.dimension {
            Dimension::Col => Coord::new(self.vector.index, index),
            Dimension::Row => Coord::new(index, self.vector.index),
        };
        coord.as_square_index(self.square.width())
    }
}

impl<'a, T> SquareVector<'a, Square<T>> {
    pub fn indexed(self) -> impl Iterator<Item = (usize, &'a T)> {
        self.indices().map(move |i| (i, &self.square[i]))
    }

    pub fn iter(self) -> impl Iterator<Item = &'a T> {
        self.indices().map(move |i| &self.square[i])
    }
}

/// A container of elements represented in a square grid
#[derive(Clone, Debug, PartialEq)]
pub struct Square<T> {
    width: SquareValue,
    elements: Vec<T>,
}

impl<T> Square<T> {
    /// Creates a new square with a specified width and fill with the default value
    pub fn with_width(width: SquareValue) -> Square<T>
    where
        T: Clone + Default,
    {
        Self {
            width,
            elements: vec![Default::default(); (width as usize).pow(2)],
        }
    }

    /// Create a new `Square` of a specified width and fill with a specified value
    pub fn with_width_and_value(width: SquareValue, val: T) -> Square<T>
    where
        T: Clone,
    {
        Square {
            width,
            elements: vec![val; (width as usize).pow(2)],
        }
    }

    /// Returns an iterator over the rows of the square
    pub fn cols(&self) -> impl Iterator<Item = impl Iterator<Item = &T> + '_> + '_ {
        (0..self.width())
            .map(move |col| (0..self.width()).map(move |row| &self[Coord::new(col, row)]))
    }

    /// Returns an iterator over the rows of the square
    pub fn rows(&self) -> impl Iterator<Item = &[T]> {
        self.elements.chunks(self.width as usize)
    }

    /// Returns a mutable iterator over the rows of the square
    pub fn rows_mut(&mut self) -> impl Iterator<Item = &mut [T]> {
        self.elements.chunks_mut(self.width as usize)
    }

    /// Returns an iterator over every element, paired with its `Coord`
    pub fn iter_coord(&self) -> impl Iterator<Item = (Coord, &T)> {
        self.elements
            .iter()
            .enumerate()
            .map(move |(i, e)| (self.cell(i).coord(), e))
    }
}

impl<T> Deref for Square<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.elements
    }
}

impl<T> IsSquare for Square<T> {
    fn len(&self) -> usize {
        self.elements.len()
    }

    fn width(&self) -> SquareValue {
        self.width
    }
}

impl<T, I: AsSquareIndex> Index<I> for Square<T> {
    type Output = T;

    fn index(&self, index: I) -> &Self::Output {
        &self.elements[index.as_square_index(self.width)]
    }
}

impl<T, I: AsSquareIndex> IndexMut<I> for Square<T> {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.elements[index.as_square_index(self.width)]
    }
}

impl<T> Display for Square<T>
where
    T: Display + Ord,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let len = self.elements.iter().max().unwrap().to_string().len();
        for row in self.rows() {
            for element in row {
                write!(f, "{:>1$} ", element, len)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub(crate) struct EmptySquare {
    width: SquareValue,
}

impl EmptySquare {
    pub fn new(width: SquareValue) -> Self {
        Self { width }
    }
}

impl IsSquare for EmptySquare {
    fn width(&self) -> SquareValue {
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
        let width = (elements.len() as f32).sqrt() as SquareValue;
        if elements.len() != (width as usize).pow(2) {
            return Err(NonSquareLength(elements.len()));
        }
        Ok(Self { width, elements })
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use crate::collections::square::NonSquareLength;
    use crate::collections::square::Square;

    #[test]
    fn try_from_vec() {
        assert!(Square::try_from(vec![1; 9]).is_ok())
    }

    #[test]
    fn try_from_non_square_vec() {
        assert_eq!(Err(NonSquareLength(8)), Square::try_from(vec![1; 8]))
    }

    mod is_square {
        use crate::collections::square::{Coord, EmptySquare, IsSquare, Vector};
        use itertools::assert_equal;

        #[test]
        fn col() {
            assert_eq!(EmptySquare::new(4).cell(0).col(), 0);
            assert_eq!(EmptySquare::new(4).cell(1).col(), 1);
            assert_eq!(EmptySquare::new(4).cell(3).col(), 3);
            assert_eq!(EmptySquare::new(4).cell(4).col(), 0);
            assert_eq!(EmptySquare::new(4).cell(5).col(), 1);
        }

        #[test]
        fn row() {
            assert_eq!(EmptySquare::new(4).cell(0).row(), 0);
            assert_eq!(EmptySquare::new(4).cell(1).row(), 0);
            assert_eq!(EmptySquare::new(4).cell(3).row(), 0);
            assert_eq!(EmptySquare::new(4).cell(4).row(), 1);
        }

        #[test]
        fn coord() {
            assert_eq!(EmptySquare::new(4).cell(0).coord(), Coord::new(0, 0));
            assert_eq!(EmptySquare::new(4).cell(1).coord(), Coord::new(1, 0));
            assert_eq!(EmptySquare::new(4).cell(3).coord(), Coord::new(3, 0));
            assert_eq!(EmptySquare::new(4).cell(4).coord(), Coord::new(0, 1));
        }

        #[test]
        fn shared_vector() {
            assert_eq!(
                Some(Vector::row(0)),
                EmptySquare::new(3).shared_vector(0, 1)
            );
            assert_eq!(
                Some(Vector::row(0)),
                EmptySquare::new(3).shared_vector(0, 2)
            );
            assert_eq!(
                Some(Vector::col(0)),
                EmptySquare::new(3).shared_vector(0, 3)
            );
            assert_eq!(None, EmptySquare::new(3).shared_vector(0, 4));
            assert_eq!(
                Some(Vector::row(1)),
                EmptySquare::new(3).shared_vector(4, 5)
            );
            assert_eq!(
                Some(Vector::col(0)),
                EmptySquare::new(3).shared_vector(0, 3)
            );
            assert_eq!(
                Some(Vector::col(0)),
                EmptySquare::new(3).shared_vector(0, 6)
            );
            assert_eq!(
                Some(Vector::col(1)),
                EmptySquare::new(3).shared_vector(1, 7)
            );
            assert_eq!(None, EmptySquare::new(3).shared_vector(1, 8));
            assert_eq!(None, EmptySquare::new(3).shared_vector(1, 3));
        }

        #[test]
        fn vectors() {
            assert_equal(
                EmptySquare::new(3).vectors(),
                vec![
                    Vector::col(0),
                    Vector::col(1),
                    Vector::col(2),
                    Vector::row(0),
                    Vector::row(1),
                    Vector::row(2),
                ],
            );
        }
    }

    mod vector {
        use crate::collections::square::{EmptySquare, IsSquare, Vector};
        use itertools::assert_equal;

        #[test]
        fn contains_square_index() {
            assert!(EmptySquare::new(3)
                .vector(Vector::col(1))
                .contains_square_index(1));
            assert!(!EmptySquare::new(3)
                .vector(Vector::col(1))
                .contains_square_index(0));
            assert!(!EmptySquare::new(3)
                .vector(Vector::col(2))
                .contains_square_index(0));
            assert!(EmptySquare::new(3)
                .vector(Vector::col(0))
                .contains_square_index(6));
            assert!(EmptySquare::new(3)
                .vector(Vector::row(0))
                .contains_square_index(0));
            assert!(EmptySquare::new(3)
                .vector(Vector::row(0))
                .contains_square_index(2));
            assert!(!EmptySquare::new(3)
                .vector(Vector::row(0))
                .contains_square_index(3));
            assert!(!EmptySquare::new(5)
                .vector(Vector::row(2))
                .contains_square_index(0));
        }

        #[test]
        fn indices_col() {
            assert_equal(
                EmptySquare::new(3).vector(Vector::col(0)).indices(),
                vec![0, 3, 6],
            );
        }

        #[test]
        fn indices_row() {
            assert_equal(
                EmptySquare::new(3).vector(Vector::row(2)).indices(),
                vec![6, 7, 8],
            );
        }

        #[test]
        fn square_index_at() {
            assert_eq!(
                EmptySquare::new(3)
                    .vector(Vector::row(2))
                    .square_index_at(2),
                8
            );
        }
    }
}
