pub use self::cage::{Cage, Operator};

use crate::collections::square::{Coord, IsSquare, Square, SquareCellRef, SquareVector};
use crate::puzzle::error::{InvalidPuzzle, ParsePuzzleError, PuzzleFromFileError};
use crate::puzzle::generate::generate_untested_puzzle;
use crate::puzzle::parse::parse_puzzle;
use crate::puzzle::solve::ValueSet;
use crate::puzzle::{CageId, CellId, Solution};
use std::borrow::Borrow;
use std::convert::TryInto;
use std::fmt::Display;
use std::ops::Deref;
use std::path::Path;
use std::{fmt, fs, mem};

mod cage;

pub(crate) type CellRef<'a> = SquareCellRef<'a, Puzzle>;

/// An unsolved KenKen puzzle
#[derive(Debug, PartialEq)]
pub struct Puzzle {
    /// the width and height of the puzzle
    width: usize,
    /// contains all cages in the puzzle
    cages: Vec<Cage>,
    cage_id_map: Square<CageId>,
}

impl Puzzle {
    /// creates a puzzle with a specified width and set of cages
    pub fn new(width: usize, cages: Vec<Cage>) -> Result<Self, InvalidPuzzle> {
        let cage_id_map = cage_id_map(width, &cages)?;
        let puzzle = Self {
            width,
            cages,
            cage_id_map,
        };
        Ok(puzzle)
    }

    pub fn from_file(path: &Path) -> Result<Self, PuzzleFromFileError> {
        let str = fs::read_to_string(path)?;
        let puzzle = Self::parse(&str)?;
        Ok(puzzle)
    }

    pub fn generate_untested(width: usize) -> Puzzle {
        generate_untested_puzzle(width)
    }

    pub fn parse(str: &str) -> Result<Self, ParsePuzzleError> {
        parse_puzzle(str)
    }

    pub fn cage(&self, id: CageId) -> CageRef<'_> {
        CageRef { puzzle: self, id }
    }

    pub fn cages(&self) -> impl Iterator<Item = CageRef<'_>> {
        (0..self.cages.len()).map(move |i| self.cage(i))
    }

    pub fn cage_count(&self) -> usize {
        self.cages.len()
    }

    pub fn cell_count(&self) -> usize {
        self.len()
    }

    pub(crate) fn cells(&self) -> impl Iterator<Item = CellRef<'_>> {
        (0..self.len()).map(move |i| self.cell(i))
    }

    pub fn cell_cage_indices(&self) -> &Square<usize> {
        &self.cage_id_map
    }

    pub fn verify_solution(&self, solution: &Solution) -> bool {
        solution.square_vectors().all(|v| self.verify_vector(v)) && self.verify_cages(solution)
    }

    fn verify_cages(&self, solution: &Solution) -> bool {
        self.cages().all(|cage| Self::verify_cage(cage, solution))
    }

    fn verify_cage(cage: CageRef<'_>, solution: &Solution) -> bool {
        let values = cage
            .cell_ids()
            .iter()
            .map(|&i| solution[i])
            .collect::<Vec<_>>();
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

    fn verify_vector<'a>(&'a self, vector: SquareVector<'a, Square<i32>>) -> bool {
        let mut set = ValueSet::new(self.width());
        vector
            .iter()
            .all(|&i| i >= 1 && i <= self.width() as i32 && set.insert(i))
    }

    pub fn width(&self) -> usize {
        self.width
    }
}

/// Create a square of values where each value represents the index of the cage
/// containing that position
fn cage_id_map(width: usize, cages: &[Cage]) -> Result<Square<usize>, InvalidPuzzle> {
    let mut cage_map = Square::with_width_and_value(width, usize::MAX);
    let mut count = 0;
    for (i, cage) in cages.iter().enumerate() {
        for &j in cage.cell_ids() {
            let old = mem::replace(&mut cage_map[j], i);
            if old != usize::MAX {
                return Err(InvalidPuzzle::new(format!(
                    "multiple cages occupy cell {}",
                    j
                )));
            }
            count += 1;
        }
    }
    if count != cage_map.len() {
        return Err(InvalidPuzzle::new("not all cells have a cage".into()));
    }
    Ok(cage_map)
}

impl Display for Puzzle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.width)?;
        for i in 0..self.width {
            for j in 0..self.width {
                let byte = b'A' + self.cage_id_map[Coord::new(j, i)] as u8;
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

impl<P: Borrow<Puzzle>> IsSquare for P {
    fn width(&self) -> usize {
        Puzzle::width(self.borrow())
    }
}

/// A reference to a cage within a Puzzle
#[derive(Clone, Copy)]
pub struct CageRef<'a> {
    puzzle: &'a Puzzle,
    id: usize,
}

impl<'a> CageRef<'a> {
    pub fn id(self) -> usize {
        self.id
    }

    pub(crate) fn cell(self, index: usize) -> CellRef<'a> {
        self.puzzle.cell(self.cell_ids()[index])
    }

    pub(crate) fn cells(self) -> impl Iterator<Item = CellRef<'a>> + 'a {
        self.cage()
            .cell_ids()
            .iter()
            .map(move |&i| self.puzzle.cell(i))
    }

    pub fn cell_count(self) -> usize {
        self.cell_ids().len()
    }

    /// Coordinates of the upper-left corner of the cage
    pub fn coord(self) -> Coord {
        self.cell(0).coord()
    }

    fn cage(self) -> &'a Cage {
        &self.puzzle.cages[self.id]
    }
}

impl Deref for CageRef<'_> {
    type Target = Cage;

    /// Note: the lifetime returned is shorter than `CageRef::cage`
    fn deref(&self) -> &Self::Target {
        self.cage()
    }
}

impl<'a> CellRef<'a> {
    pub fn cage(self) -> CageRef<'a> {
        self.puzzle().cage(self.cage_id())
    }

    pub fn cage_id(self) -> CageId {
        self.puzzle().cage_id_map[self.id()]
    }

    pub fn id(self) -> CellId {
        self.index()
    }

    pub fn puzzle(self) -> &'a Puzzle {
        self.square()
    }
}

impl<'a> SquareVector<'a, Puzzle> {
    pub fn iter(self) -> impl Iterator<Item = CellRef<'a>> + 'a {
        self.indices().map(move |i| self.square.cell(i))
    }
}
