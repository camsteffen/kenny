//! KenKen puzzles

pub mod solve;

pub use self::cage::Cage;
pub use self::cage::Operator;
pub use self::generate::generate_puzzle;
pub use self::generate::generate_puzzle_with_solution;
pub use self::parse::parse_puzzle as parse;
pub use self::image::PuzzleImageBuilder;

mod cage;
mod generate;
mod image;
mod parse;

use collections::square::Square;
use std::fmt::Display;
use std::fmt;
use std::ops::Index;
use collections::square::Coord;

/// An unsolved KenKen puzzle
pub struct Puzzle {
    /// the width and height of the puzzle
    pub width: u32,
    /// contains all cages in the puzzle
    pub cages: Vec<Cage>,
    pub cage_map: Square<u32>,
}

impl Puzzle {

    /// creates a puzzle with a specified width and set of cages
    pub fn new(width: u32, cages: Vec<Cage>) -> Self {
        let cage_map = cage_map(width, &cages);
        Self {
            width,
            cages,
            cage_map,
        }
    }

    pub fn get_cage(&self, cage_index: u32) -> &Cage {
        &self.cages[cage_index as usize]
    }

    pub fn get_cage_mut(&mut self, cage_index: u32) -> &mut Cage {
        &mut self.cages[cage_index as usize]
    }

    /// retrieves the index of the cage containing the cell with the given index
    pub fn cage_index_at<T>(&self, cell_index: T) -> u32
            where Square<u32> : Index<T, Output=u32> {
        self.cage_map[cell_index]
    }

}

/**
 * Create a square of values where each value represents the index of the cage
 * containing that position
 */
fn cage_map(width: u32, cages: &[Cage]) -> Square<u32> {
    let mut cage_map = Square::with_width_and_value(width as usize, 0);
    for (i, cage) in cages.iter().enumerate() {
        for &j in &cage.cells {
            cage_map[j] = i as u32;
        }
    }
    cage_map
}

impl Display for Puzzle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.width)?;
        for i in 0..self.width as usize {
            for j in 0..self.width as usize {
                let byte = b'A' + self.cage_map[Coord([i, j])] as u8;
                write!(f, "{}", byte as char)?;
            }
            writeln!(f, "")?;
        }
        for cage in &self.cages {
            write!(f, "{}", cage.target)?;
            if let Some(s) = cage.operator.symbol() {
                write!(f, "{}", s)?;
            }
            writeln!(f, "")?;
        }
        Ok(())
    }
}
