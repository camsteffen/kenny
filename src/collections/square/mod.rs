//! Modlue for the `Square` struct

mod coord;
mod vector;

pub use self::coord::Coord;
pub use self::vector::VectorId;

use std::cmp::Ord;
use std::fmt::Display;
use std::fmt;
use std::mem;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ops::Index;
use std::ops::IndexMut;
use std::slice::Chunks;
use std::slice::ChunksMut;

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct SquareIndex(pub usize);

impl SquareIndex {

    pub fn row(&self, square_width: usize) -> usize {
        self.0 / square_width
    }

    pub fn col(&self, square_width: usize) -> usize {
        self.0 % square_width
    }

    /// Create a `Coord` using the index of an elment in a `Square` and the size of the `Square`.
    pub fn as_coord(&self, size: usize) -> Coord {
        Coord([self.row(size), self.col(size)])
    }

    pub fn shared_vector(&self, other: SquareIndex, width: usize) -> Option<VectorId> {
        let SquareIndex(pos1) = *self;
        let SquareIndex(pos2) = other;
        if pos1 / width == pos2 / width {
            Some(VectorId::row(pos1 / width))
        } else if pos1 % width == pos2 % width {
            Some(VectorId::col(pos1 % width))
        } else {
            None
        }
    }

    /// Returns an array with the row and column intersecting at the given position
    pub fn vectors_intersecting_at(pos: SquareIndex, size: usize) -> [VectorId; 2] {
        let SquareIndex(pos) = pos;
        [
            VectorId::row(pos / size),
            VectorId::col(pos % size),
        ]
    }
}

impl Deref for SquareIndex {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A container of elements represented in a square grid
pub struct Square<T> {
    width: usize,
    elements: Vec<T>,
}

impl<T> Square<T> {

    /// Creates a new square with a specified width and fill with the default value
    pub fn with_width(width: usize) -> Square<T>
            where T: Default {
        Self {
            width,
            elements: vec![Default::default(); width.pow(2)],
        }
    }

    /// Create a new `Square` of a specified width and fill with a specified value
    pub fn with_width_and_value(width: usize, val: T) -> Square<T>
            where T: Clone {
        Square {
            width: width,
            elements: vec![val; width.pow(2)],
        }
    }

    /// Returns the width (and height) of the grid
    pub fn width(&self) -> usize {
        self.width
    }

    /// Returns an iterator over the rows of the square
    pub fn rows(&self) -> Chunks<T> {
        self.elements.chunks(self.width)
    }

    /// Returns a mutable iterator over the rows of the square
    pub fn rows_mut(&mut self) -> ChunksMut<T> {
        self.elements.chunks_mut(self.width)
    }

    /// Returns an iterator over every element, paired with its `Coord`
    pub fn iter_coord(&self) -> CoordIter<T> {
        CoordIter {
            size: self.width,
            index: 0,
            data: self.elements.as_slice(),
        }
    }
}

impl<T> Deref for Square<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        &self.elements
    }
}

impl<T> DerefMut for Square<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.elements
    }
}

impl<T> Index<Coord> for Square<T> {
    type Output = T;
    fn index(&self, coord: Coord) -> &T {
        &self[coord.as_index(self.width)]
    }
}

impl<T> IndexMut<Coord> for Square<T> {
    fn index_mut(&mut self, coord: Coord) -> &mut T {
        &mut self[coord.as_index(self.width)]
    }
}

impl<T> Index<SquareIndex> for Square<T> {
    type Output = T;
    fn index(&self, i: SquareIndex) -> &T {
        &self.elements[*i]
    }
}

impl<T> IndexMut<SquareIndex> for Square<T> {
    fn index_mut(&mut self, i: SquareIndex) -> &mut T {
        &mut self.elements[*i]
    }
}

impl<T> fmt::Display for Square<T>
where T: Display + Ord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

/// An iterator over the elements of a `Square` where each item is a tuple
/// with the `Coord` of the element
pub struct CoordIter<'a, T: 'a> {
    size: usize,
    index: usize,
    data: &'a [T],
}

impl<'a, T> Iterator for CoordIter<'a, T> {
    type Item = (Coord, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.size.pow(2) {
            return None
        }
        let data = mem::replace(&mut self.data, &[]);
        let (first, remaining) = data.split_first().unwrap();
        self.data = remaining;
        let p = (SquareIndex(self.index).as_coord(self.size), first);
        self.index += 1;
        Some(p)
    }
}

