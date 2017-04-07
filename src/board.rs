extern crate itertools;
extern crate rand;

use std::fmt::Display;
use itertools::{Itertools, repeat_call};
use rand::{thread_rng, Rng};
use std::mem;
use std::slice::{Chunks, ChunksMut};
use std::fmt;

type Cell = i32;

struct Square<T> {
    size: usize,
    elements: Vec<T>,
}

pub struct Board {
    pub size: usize,
    pub cells: Vec<i32>,
    pub cages: Vec<Cage>,
}

struct ColIter<'a>(Vec<&'a [Cell]>);

impl<'a> Iterator for ColIter<'a> {
    type Item = Vec<&'a Cell>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0[0].is_empty() {
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

struct ColIterMut<'a>(Vec<&'a mut [Cell]>);

impl<'a> Iterator for ColIterMut<'a> {

    type Item = Vec<&'a mut Cell>;

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

impl Board {

    pub fn cage_indices(&self) -> Vec<usize> {
        let mut indices = vec![0; self.size.pow(2)];
        for (i, cage) in self.cages.iter().enumerate() {
            for cell in cage.cells.iter() {
                indices[*cell] = i;
            }
        }
        indices
    }

    fn cell_at(&mut self, x: usize, y: usize) -> &mut Cell {
        &mut self.cells[x * self.size + y]
    }

    fn cols<'a>(&'a mut self) -> ColIter<'a> {
        ColIter(self.rows().collect_vec())
    }

    fn cols_mut<'a>(&'a mut self) -> ColIterMut<'a> {
        ColIterMut(self.rows_mut().collect_vec())
    }

    fn rows(&self) -> Chunks<Cell> {
        self.cells.chunks(self.size)
    }

    fn rows_mut(&mut self) -> ChunksMut<Cell> {
        self.cells.chunks_mut(self.size)
    }

}

impl fmt::Display for Board {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in self.rows() {
            for cell in row {
                write!(f, "{:>2} ", cell).unwrap();
            }
            write!(f, "\n").unwrap();
        }
        Ok(())
    }

}

pub fn print_square<T>(cells: &[T], size: usize) where T: Display {
    for row in cells.chunks(size) {
        for cell in row {
            print!("{:>3} ", cell);
        }
        println!();
    }
}

#[derive(Clone, Debug)]
pub enum Operator { Add, Subtract, Multiply, Divide }
static OPERATORS: [Operator; 4] = [Operator::Add, Operator::Subtract, Operator::Multiply, Operator::Divide];

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

fn find_cage_operator(cells: &[i32], indices: &[usize]) -> (Operator, i32) {
    let mut rng = thread_rng();
    let mut operators = Vec::with_capacity(4);
    let mut min: i32 = -1;
    let mut max: i32 = -1;
    let vals = (0..indices.len()).map(|i| cells[i]).collect_vec();
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

fn random_latin_square(size: usize) -> Vec<i32> {
    let mut cells = Vec::with_capacity(size.pow(2));
    let mut rng = thread_rng();
    let seed = repeat_call(|| {
        let mut seed = (0..size as i32).collect_vec();
        rng.shuffle(&mut seed);
        seed
    }).take(2).collect_vec();
    for i in 0..size {
        for j in 0..size {
            cells.push((seed[0][i] + seed[1][j]) % size as i32 + 1);
        }
    }
    cells
}

enum Direction { Up, Down, Left, Right }

fn move_position(position: i32, direction: &Direction, size: usize) -> Option<i32> {
    let size = size as i32;
    match direction {
        &Direction::Up => {
            if position >= size { Some(position - size) }
            else { None }
        },
        &Direction::Down => {
            let val = position + size;
            if val < size.pow(2) { Some(val) }
            else { None }
        },
        &Direction::Left => {
            if position % size != 0 { Some(position - 1) }
            else { None }
        },
        &Direction::Right => {
            if position % size != size - 1 { Some(position + 1) }
            else { None }
        },
    }
}

fn generate_cages(cells: &[i32], size: usize) -> Vec<Cage> {
    let min_cage_size = 2;
    let max_cage_size = 4;
    let num_cells = size.pow(2);
    let no_cage = -1;
    let mut cage_ids = vec![no_cage; num_cells];
    let mut uncaged = num_cells;
    let mut cur_cage = 0;
    let mut pos = 0;
    let mut rng = thread_rng();
    let directions = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];
    'cages: loop {
        let cage_size = rng.gen_range(min_cage_size, max_cage_size + 1);
        for _ in 0..cage_size {
            cage_ids[pos as usize] = cur_cage;
            uncaged = uncaged - 1;
            if uncaged == 0 {
                break 'cages
            }
            let available_positions = directions.iter()
                .filter_map(|d| move_position(pos, d, size))
                .filter(|pos| cage_ids[*pos as usize] == no_cage)
                .collect_vec();
            match rng.choose(&available_positions) {
                Some(p) => pos = *p,
                None => {
                    pos = cage_ids.iter().position(|c| *c == no_cage).unwrap() as i32;
                    break
                }
            }
        }
        cur_cage = cur_cage + 1;
    }
    let num_cages = cur_cage as usize + 1;
    //(cage_ids, num_cages)


    let mut cage_cells = vec![Vec::new(); num_cages];
    for i in 0..num_cells {
        let cage_index = cage_ids[i] as usize;
        let val = cells[i];
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

pub fn generate_board(size: usize) -> Board {
    let cells = random_latin_square(size);
    let cages = generate_cages(&cells, size);
    Board {
        size: size,
        cells: cells,
        cages: cages,
    }
}

