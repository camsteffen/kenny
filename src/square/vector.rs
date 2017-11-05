//! Module for rows and columns of a `Square`

use std::fmt;
use std::fmt::Display;
use self::VectorId::{Col, Row};

/// A row or column and its index
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(missing_docs)]
pub enum VectorId {
    Row(usize),
    Col(usize),
}

impl VectorId {
    /// Retrives the index of the vector in its respective dimension
    pub fn index(&self) -> usize {
        match *self {
            Row(i) => i,
            Col(i) => i,
        }
    }

    /// Retrieves the VectorId as a number. Each vector in a square has a unique number in the range from 0 to 2 * size - 1
    pub fn as_number(&self, size: usize) -> usize {
        match *self {
            Row(i) => i,
            Col(i) => size + i,
        }
    }
    

    /// Retrieves the vector that intersects this vector at a given position
    pub fn intersecting_at(&self, pos: usize) -> VectorId {
        match *self {
            Row(_) => Col(pos),
            Col(_) => Row(pos),
        }
    }

    /// Calculates the position of a cell with respect to a vector, given the position of the cell with respect to the square.
    pub fn sq_pos_to_vec_pos(&self, pos: usize, size: usize) -> usize {
        match *self {
            Row(i) => {
                debug_assert!(pos / size == i);
                pos % size
            },
            Col(i) => {
                debug_assert!(pos % size == i);
                pos / size
            },
        }
    }

    /// Calculates the position of a cell with respect to a square, given the position of the cell with respect to a vector.
    pub fn vec_pos_to_sq_pos(&self, pos: usize, size: usize) -> usize {
        match *self {
            Row(i) => i * size + pos,
            Col(i) => i + pos * size,
        }
    }
}

impl Display for VectorId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (label, index) = match *self {
            Row(index) => ("Row", index),
            Col(index) => ("Col", index),
        };
        write!(f, "{} {}", label, index)
    }
}

/// Returns an array with the row and column intersecting at the given position
pub fn vectors_intersecting_at(pos: usize, size: usize) -> [VectorId; 2] {
    [
        Row(pos / size),
        Col(pos % size),
    ]
}

/// Returns an iterator over the positions of the elements in a vector
pub fn iter_vector(vector_id: VectorId, size: usize) -> impl Iterator<Item=usize> {
    let start;
    let inc;
    match vector_id {
        Row(index) => {
            start = index * size;
            inc = 1;
        },
        Col(index) => {
            start = index;
            inc = size;
        },
    }
    (0..size).map(move |i| start + inc * i)
}

