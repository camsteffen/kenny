use crate::collections::{Square, RangeSet};
use std::path::Path;
use std::fs::File;
use crate::puzzle::generate::generate_untested_puzzle;
use crate::collections::square::{SquareIndex, Coord, AsSquareIndex, VectorId, Dimension, IsSquare};
use crate::puzzle::parse::parse_puzzle;
use crate::puzzle::error::ParsePuzzleError;
use std::io::Read;
use std::fmt::Display;
use std::fmt;
use std::ops::Deref;
use failure::Fallible;

pub use self::cage::{Cage, Operator};
use std::convert::TryInto;

mod cage;

/// An unsolved KenKen puzzle
pub struct Puzzle {
    /// the width and height of the puzzle
    width: usize,
    /// contains all cages in the puzzle
    cages: Vec<Cage>,
    cage_map: Square<usize>,
}

impl Puzzle {
    /// creates a puzzle with a specified width and set of cages
    pub fn new(width: usize, cages: Vec<Cage>) -> Self {
        let cage_map = cage_map(width, &cages);
        Self {
            width,
            cages,
            cage_map,
        }
    }

    pub fn from_file(path: impl AsRef<Path>) -> Fallible<Self> {
        let mut file = File::open(path)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        let puzzle = Self::parse(&buf)?;
        Ok(puzzle)
    }

    pub fn generate_untested(width: usize) -> Puzzle {
        generate_untested_puzzle(width)
    }

    pub fn parse(str: &str) -> Result<Self, ParsePuzzleError> {
        parse_puzzle(str)
    }

    pub fn cage(&self, index: usize) -> CageRef {
        CageRef { puzzle: self, index }
    }

    pub fn cages(&self) -> Cages {
        Cages { puzzle: self }
    }

    pub fn cell(&self, index: impl AsSquareIndex) -> CellRef {
        CellRef {
            puzzle: self,
            index: index.as_square_index(self.width),
        }
    }

    pub fn cell_count(&self) -> usize {
        self.width.pow(2)
    }

    pub fn cells(&self) -> impl Iterator<Item=CellRef> {
        (0..self.width * self.width)
            .map(move |i| self.cell(i))
    }

    pub fn cell_cage_indices(&self) -> &Square<usize> {
        &self.cage_map
    }

    pub fn vector_cells(&self, vector_id: VectorId) -> impl Iterator<Item=CellRef> {
        let (start, end, step_by) = match vector_id.dimension() {
            Dimension::Row => (self.width * vector_id.index(), (self.width + 1) * vector_id.index(), 1),
            Dimension::Col => (vector_id.index(), vector_id.index() + self.cell_count(), self.width),
        };
        (start..end).step_by(step_by).map(move |i| self.cell(i))
    }

    pub fn verify_solution(&self, solution: &Square<i32>) -> bool {
        solution.rows().all(|v| self.verify_vector(v.iter().copied()))
            && solution.cols().all(|v| self.verify_vector(v.copied()))
            && self.verify_cages(solution)
    }

    fn verify_cages(&self, solution: &Square<i32>) -> bool {
        self.cages().iter().all(|cage| self.verify_cage(cage, solution))
    }

    fn verify_cage(&self, cage: CageRef, solution: &Square<i32>) -> bool {
        let values = cage.cell_indices.iter().map(|&i| solution[i]).collect::<Vec<_>>();
        match cage.operator() {
            Operator::Add => values.iter().sum::<i32>() == cage.target(),
            Operator::Subtract => {
                let mut values: [_; 2] = values[..].try_into().unwrap();
                values.sort_unstable();
                values[1] - values[0] == cage.target()
            }
            Operator::Multiply => values.iter().product::<i32>() == cage.target(),
            Operator::Divide => {
                let mut values: [_; 2] = values[..].try_into().unwrap();
                values.sort_unstable();
                let [a, b] = values;
                b % a == 0 && b / a == cage.target()
            }
            Operator::Nop => {
                let [v]: [_; 1] = values[..].try_into().unwrap();
                v == cage.target()
            }
        }
    }

    fn verify_vector(&self, mut vector: impl Iterator<Item=i32>) -> bool {
        let mut set = RangeSet::new(self.width());
        vector.all(|i| i >= 1 && i <= self.width() as i32 && set.insert(i as usize))
    }

    pub fn width(&self) -> usize {
        self.width
    }
}

/**
 * Create a square of values where each value represents the index of the cage
 * containing that position
 */
fn cage_map(width: usize, cages: &[Cage]) -> Square<usize> {
    let mut cage_map = Square::with_width_and_value(width, 0);
    for (i, cage) in cages.iter().enumerate() {
        for &j in &cage.cell_indices {
            cage_map[j] = i;
        }
    }
    cage_map
}

impl Display for Puzzle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.width)?;
        for i in 0..self.width {
            for j in 0..self.width {
                let byte = b'A' + self.cage_map[Coord::new(j, i)] as u8;
                write!(f, "{}", byte as char)?;
            }
            writeln!(f)?;
        }
        for cage in &self.cages {
            write!(f, "{}", cage.target())?;
            if let Some(s) = cage.operator().symbol() {
                write!(f, "{}", s)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl IsSquare for Puzzle {
    fn width(&self) -> usize {
        self.width()
    }
}

#[derive(Clone, Copy)]
pub struct Cages<'a> {
    puzzle: &'a Puzzle,
}

impl<'a> Cages<'a> {
    pub fn iter(self) -> impl Iterator<Item=CageRef<'a>> {
        (0..self.puzzle.cages.len())
            .map(move |i| self.puzzle.cage(i))
    }
}

#[derive(Clone, Copy)]
pub struct CageRef<'a> {
    puzzle: &'a Puzzle,
    index: usize,
}

impl<'a> CageRef<'a> {
    pub fn cage(self) -> &'a Cage {
        &self.puzzle.cages[self.index]
    }

    pub fn cell(self, index: usize) -> CellRef<'a> {
        self.puzzle.cell(self.cage().cell_indices[index])
    }

    pub fn cells(self) -> impl Iterator<Item=CellRef<'a>> {
        self.cage().cell_indices.iter().map(move |&i| self.puzzle.cell(i))
    }

    pub fn cell_count(self) -> usize {
        self.cage().cell_indices.len()
    }

    pub fn index(self) -> usize {
        self.index
    }
}

impl<'a> Deref for CageRef<'a> {
    type Target = Cage;

    fn deref(&self) -> &Self::Target {
        self.cage()
    }
}

#[derive(Clone, Copy)]
pub struct CellRef<'a> {
    puzzle: &'a Puzzle,
    index: SquareIndex,
}

impl<'a> CellRef<'a> {
    pub fn cage(self) -> CageRef<'a> {
        self.puzzle.cage(self.cage_index())
    }

    pub fn coord(self) -> Coord {
        self.puzzle.coord_at(self.index)
    }

    pub fn index(self) -> SquareIndex {
        self.index
    }

    pub fn intersects_vector(self, vector_id: VectorId) -> bool {
        vector_id.intersects_coord(self.coord())
    }

    pub fn vectors(self) -> [VectorId; 2] {
        self.coord().vectors()
    }

    fn cage_index(self) -> usize {
        self.puzzle.cage_map[self.index]
    }
}
