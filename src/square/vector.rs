//! Module for rows and columns of a `Square`

use std::fmt;
use std::fmt::Display;

/// A row or column and its index
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(missing_docs)]
pub enum VectorId {
    Row(usize),
    Col(usize),
}

impl Display for VectorId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (label, index) = match *self {
            VectorId::Row(index) => ("Row", index),
            VectorId::Col(index) => ("Col", index),
        };
        write!(f, "{} {}", label, index)
    }
}

/// Returns an array with the row and column intersecting at the given position
pub fn vectors_intersecting_at(pos: usize, size: usize) -> [VectorId; 2] {
    [
        VectorId::Row(pos / size),
        VectorId::Col(pos % size),
    ]
}

/// Returns an iterator over the positions of the elements in a vector
pub fn iter_vector(vector_id: VectorId, size: usize) -> Box<Iterator<Item=usize>> {
    let start;
    let inc;
    match vector_id {
        VectorId::Row(index) => {
            start = index * size;
            inc = 1;
        },
        VectorId::Col(index) => {
            start = index;
            inc = size;
        },
    }
    Box::new((0..size).map(move |i| start + inc * i))
}

