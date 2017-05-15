use std::fmt;
use std::fmt::Display;

/**
 * A row or column and its index
 */
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

pub fn vectors_intersecting_at(pos: usize, size: usize) -> [VectorId; 2] {
    [
        VectorId::Row(pos / size),
        VectorId::Col(pos % size),
    ]
}

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

