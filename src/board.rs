use itertools::Itertools;
use std::fmt::Debug;
use itertools::repeat_call;
use rand::Rng;
use std::cmp::Ord;
use rand::thread_rng;
use std::fmt::Display;
use std::fmt;
use std::mem;
use std::ops::Index;
use std::ops::IndexMut;
use std::slice::{Chunks, ChunksMut};

pub type Board = Square<i32>;

#[derive(Clone)]
pub struct Coord([usize; 2]);

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

impl Debug for Coord {
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

    fn rows(&self) -> Chunks<T> {
        self.elements.chunks(self.size)
    }

    fn rows_mut(&mut self) -> ChunksMut<T> {
        self.elements.chunks_mut(self.size)
    }

    pub fn iter(&mut self) -> SquareCoordDataIter<T> {
        SquareCoordDataIter {
            size: self.size,
            index: 0,
            data: self.elements.as_mut_slice(),
        }
    }

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

impl<'a, T> Index<&'a Coord> for Square<T> {
    type Output = T;
    fn index(&self, coord: &'a Coord) -> &T {
        &self.elements[coord.to_index(self.size)]
    }
}

impl<'a, T> IndexMut<&'a Coord> for Square<T> {
    fn index_mut(&mut self, coord: &'a Coord) -> &mut T {
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

/**
 * Create a square of values where each value represents the index of the cage
 * containing that position
 */
pub fn cage_indices(cages: &[Cage], size: usize) -> Square<usize> {
    let mut indices = Square::new(0, size);
    for (i, cage) in cages.iter().enumerate() {
        for j in cage.cells.iter() {
            indices[*j] = i;
        }
    }
    indices
}

#[derive(Clone, Debug)]
pub enum Operator { Add, Subtract, Multiply, Divide }

pub fn operator_symbol(op: &Operator) -> char {
    match op {
        &Operator::Add      => '+',
        &Operator::Subtract => '-',
        &Operator::Multiply => '*',
        &Operator::Divide   => '/',
    }
}

#[derive(Debug)]
pub struct Cage {
    pub operator: Operator,
    pub target: i32,
    pub cells: Vec<usize>,
}

fn find_cage_operator(cells: &Board, indices: &[usize]) -> (Operator, i32) {
    let mut rng = thread_rng();
    let mut operators = Vec::with_capacity(4);
    let mut min: i32 = -1;
    let mut max: i32 = -1;
    let vals = indices.iter()
        .map(|i| cells[*i])
        .collect_vec();
    operators.push(Operator::Add);
    operators.push(Operator::Multiply);
    if indices.len() == 2 {
        min = *vals.iter().min().unwrap();
        max = *vals.iter().max().unwrap();
        operators.push(Operator::Subtract);
        if max % min == 0 {
            operators.push(Operator::Divide);
        }
    }
    let operator = rng.choose(&operators).unwrap().clone();
    let target = match operator {
        Operator::Add => vals.iter().sum(),
        Operator::Subtract => max - min,
        Operator::Multiply => vals.iter().product(),
        Operator::Divide => max / min,
    };
    (operator, target)
}

/**
 * Generate a random latin square with values from 1 to size
 */
fn random_latin_square(size: usize) -> Square<i32> {
    let mut rng = thread_rng();
    let seed = repeat_call(|| {
        let mut seed = (0..size as i32).collect_vec();
        rng.shuffle(&mut seed);
        seed
    }).take(2).collect_vec();
    let mut square: Square<i32> = Square::new(0, size);
    for (i, row) in square.rows_mut().enumerate() {
        for (j, element) in row.iter_mut().enumerate() {
            *element = (seed[0][i] + seed[1][j]) % size as i32 + 1;
        }
    }
    square
}

fn generate_cages(cells: &Board) -> Vec<Cage> {
    let size = cells.size;
    let min_cage_size = 2;
    let max_cage_size = 4;
    let num_cells = cells.elements.len();
    let no_cage = -1;
    let mut cage_ids = Square::new(no_cage, cells.size);
    let mut uncaged = num_cells;
    let mut cur_cage = 0;
    let mut pos = Coord::from_index(0, size);
    let mut rng = thread_rng();
    //let directions = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];
    'cages: loop {
        let cage_size = rng.gen_range(min_cage_size, max_cage_size + 1);
        for _ in 0..cage_size {
            cage_ids[&pos] = cur_cage;
            uncaged = uncaged - 1;
            if uncaged == 0 {
                break 'cages
            }
            let mut available_positions = Vec::with_capacity(4);
            for i in 0..2 {
                if pos[i] > 0 {
                    pos[i] = pos[i] - 1;
                    available_positions.push(pos.clone());
                    pos[i] = pos[i] + 1;
                }
                if pos[i] < size - 1 {
                    pos[i] = pos[i] + 1;
                    available_positions.push(pos.clone());
                    pos[i] = pos[i] - 1;
                }
            }
            available_positions = available_positions.into_iter()
                .filter(|next| cage_ids[next] == no_cage)
                .collect_vec();
            match rng.choose(&available_positions) {
                Some(p) => pos = p.clone(),
                None => {
                    let index = cage_ids.elements.iter()
                        .position(|c| *c == no_cage)
                        .unwrap();
                    pos = Coord::from_index(index, size);
                    break
                }
            }
        }
        cur_cage = cur_cage + 1;
    }
    let num_cages = cur_cage as usize + 1;

    // for every cage_cells[i][j], cell j is in cage i
    let mut cage_cells = vec![Vec::new(); num_cages];
    for i in 0..num_cells {
        let cage_index = cage_ids[i] as usize;
        cage_cells[cage_index].push(i);
    }
    let mut cages = Vec::with_capacity(num_cages);
    for cage_cells in cage_cells.into_iter() {
        let (operator, target) = find_cage_operator(cells, &cage_cells);
        cages.push(Cage {
            operator: operator,
            target: target,
            cells: cage_cells,
        });
    }
    cages
}

pub fn generate_puzzle(size: usize) -> (Board, Vec<Cage>) {
    let solution = random_latin_square(size);
    let cages = generate_cages(&solution);
    (solution, cages)
}

