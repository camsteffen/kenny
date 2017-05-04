use std::cmp::Ord;
use std::fmt::Display;
use std::fmt;
use std::mem;
use std::ops::Index;
use std::ops::IndexMut;
use std::slice::{Chunks, ChunksMut};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Coord(pub [usize; 2]);

impl Coord {
    pub fn new(x: usize, y: usize) -> Coord {
        Coord([x, y])
    }

    pub fn from_index(index: usize, size: usize) -> Coord {
        Coord([index / size, index % size])
    }

    pub fn to_index(&self, size: usize) -> usize {
        self[0] * size + self[1]
    }
}

impl Display for Coord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.0[0], self.0[1])
    }
}

impl Index<usize> for Coord {
    type Output = usize;
    fn index(&self, i: usize) -> &usize {
        &self.0[i]
    }
}

impl IndexMut<usize> for Coord {
    fn index_mut(&mut self, i: usize) -> &mut usize {
        &mut self.0[i]
    }
}

pub struct Square<T> {
    pub size: usize,
    pub elements: Vec<T>,
}

impl<T> Square<T> {
    pub fn new(val: T, size: usize) -> Square<T>
        where T: Clone {
        Square {
            size: size,
            elements: vec![val; size.pow(2)],
        }
    }

    /*
    fn cols<'a>(&'a self) -> ColIter<'a, T> {
        ColIter(self.rows().collect_vec())
    }

    fn cols_mut<'a>(&'a mut self) -> ColIterMut<'a, T> {
        ColIterMut(self.rows_mut().collect_vec())
    }
    */

    pub fn rows(&self) -> Chunks<T> {
        self.elements.chunks(self.size)
    }

    pub fn rows_mut(&mut self) -> ChunksMut<T> {
        self.elements.chunks_mut(self.size)
    }

    /*
    pub fn iter<'a>(&'a self) -> Box<Iterator<Item=&T> + 'a> {
        Box::new(self.elements.iter())
    }
    */

    pub fn iter_coord(&self) -> SquareCoordDataIter<T> {
        SquareCoordDataIter {
            size: self.size,
            index: 0,
            data: self.elements.as_slice(),
        }
    }

    /*
    pub fn iter_mut(&mut self) -> SquareCoordDataIter<T> {
        SquareCoordDataIter {
            size: self.size,
            index: 0,
            data: self.elements.as_mut_slice(),
        }
    }
    */

    pub fn print(&self) where T: Display + Ord {
        let len = self.elements.iter().max().unwrap()
            .to_string().len();
        for row in self.rows() {
            for element in row {
                print!("{:>1$} ", element, len);
            }
            println!();
        }
    }
}

impl<T> Index<Coord> for Square<T> {
    type Output = T;
    fn index(&self, coord: Coord) -> &T {
        &self.elements[coord.to_index(self.size)]
    }
}

impl<T> IndexMut<Coord> for Square<T> {
    fn index_mut(&mut self, coord: Coord) -> &mut T {
        &mut self.elements[coord.to_index(self.size)]
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

/*
impl<T> IntoIterator for Square<T> {
    type Item = T;
    type IntoIter = <Vec<T> as IntoIterator>::IntoIter;

    fn into_iter(&self) -> Self::IntoIter {
    }
}
*/

impl<T: Display> fmt::Display for Square<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in self.rows() {
            for element in row {
                write!(f, "{:>2} ", element).unwrap();
            }
            write!(f, "\n").unwrap();
        }
        Ok(())
    }
}

/*
struct SquareCoordIter {
    size: usize,
    i: usize,
}

impl SquareCoordIter {
    fn new(size: usize) -> SquareCoordIter {
        SquareCoordIter {
            size: size,
            i: 0,
        }
    }
}

impl Iterator for SquareCoordIter {
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i == self.size.pow(2) {
            return None;
        }
        let coord = Coord::from_index(self.i, self.size);
        self.i = self.i + 1;
        Some(coord)
    }
}
*/

pub struct SquareCoordDataIter<'a, T: 'a> {
    size: usize,
    index: usize,
    data: &'a [T],
}

impl<'a, T> Iterator for SquareCoordDataIter<'a, T> {
    type Item = (Coord, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.size.pow(2) {
            return None
        }
        let data = mem::replace(&mut self.data, &mut []);
        let (first, remaining) = data.split_first().unwrap();
        self.data = remaining;
        let p = (Coord::from_index(self.index, self.size), first);
        self.index = self.index + 1;
        Some(p)
    }
}

/*
struct ColIter<'a, T: 'a>(Vec<&'a [T]>);

impl<'a, T> Iterator for ColIter<'a, T> {
    type Item = Vec<&'a T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
            return None
        }
        Some((0..self.0.len()).map(|i| {
            let row = mem::replace(&mut self.0[i], &mut []);
            let (cell, remaining) = row.split_first().unwrap();
            self.0[i] = remaining;
            cell
        }).collect_vec())
    }
}

struct ColIterMut<'a, T: 'a>(Vec<&'a mut [T]>);

impl<'a, T> Iterator for ColIterMut<'a, T> {

    type Item = Vec<&'a mut T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0[0].is_empty() {
            return None
        }
        Some((0..self.0.len()).map(|i| {
            let row = mem::replace(&mut self.0[i], &mut []);
            let (cell, remaining) = row.split_first_mut().unwrap();
            self.0[i] = remaining;
            cell
        }).collect_vec())
    }

}
*/

