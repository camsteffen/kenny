use std::fmt;
use std::fmt::Display;
use std::ops::Deref;
use std::ops::DerefMut;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Coord(pub [usize; 2]);

impl Coord {
    pub fn from_index(index: usize, size: usize) -> Coord {
        Coord([index / size, index % size])
    }

    pub fn to_index(&self, size: usize) -> usize {
        self[0] * size + self[1]
    }
}

impl Deref for Coord {
    type Target = [usize; 2];

    fn deref(&self) -> &[usize; 2] {
        &self.0
    }
}

impl DerefMut for Coord {
    fn deref_mut(&mut self) -> &mut [usize; 2] {
        &mut self.0
    }
}

impl Display for Coord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.0[0], self.0[1])
    }
}

