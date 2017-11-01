//! Modlue for the `Square` struct

mod coord;
pub mod vector;

pub use self::coord::Coord;

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

/// A container of elements represented in a square grid
pub struct Square<T> {
    width: usize,
    elements: Vec<T>,
}

impl<T> Square<T> {

    /// Create a new `Square` of a specified width and fill with a specified value
    pub fn new(val: T, width: usize) -> Square<T>
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
        &self.elements[coord.to_index(self.width)]
    }
}

impl<T> IndexMut<Coord> for Square<T> {
    fn index_mut(&mut self, coord: Coord) -> &mut T {
        &mut self.elements[coord.to_index(self.width)]
    }
}

impl<T> Index<usize> for Square<T> {
    type Output = T;
    fn index(&self, i: usize) -> &T {
        &self.elements[i]
    }
}

impl<T> IndexMut<usize> for Square<T> {
    fn index_mut(&mut self, i: usize) -> &mut T {
        &mut self.elements[i]
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
        let p = (Coord::from_index(self.index, self.size), first);
        self.index += 1;
        Some(p)
    }
}

