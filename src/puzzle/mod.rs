//! KenKen puzzles

pub use self::cage::Cage;
pub use self::cage::Operator;
pub use self::generate::generate_puzzle as generate;
pub use self::parse::parse_puzzle as parse;

mod cage;
mod generate;
mod image;
mod parse;
mod solve;

use ::image::RgbImage;
use collections::square::Square;
use self::solve::markup::PuzzleMarkup;
use self::solve::solve_puzzle;
use std::ops::Index;

/// An unsolved KenKen puzzle
pub struct Puzzle {
    /// the width and height of the puzzle
    pub width: usize,
    /// contains all cages in the puzzle
    pub cages: Vec<Cage>,
    pub cage_map: Square<usize>,
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

    /// attempts to solve the puzzle and return the PuzzleMarkup with the solution
    pub fn solve(&self) -> PuzzleMarkup {
        solve_puzzle(self)
    }

    /// retrieves the index of the cage containing the cell with the given index
    pub fn cage_index_at<T>(&self, cell_index: T) -> usize
            where Square<usize> : Index<T, Output=usize> {
        self.cage_map[cell_index]
    }

    /// creates an image of puzzle
    pub fn image(&self) -> RgbImage {
        image::puzzle_image(self)
    }

    /// creates an image of the puzzle with markup
    pub fn image_with_markup(&self, markup: &PuzzleMarkup) -> RgbImage {
        image::puzzle_image_with_markup(self, markup)
    }

}

/**
 * Create a square of values where each value represents the index of the cage
 * containing that position
 */
fn cage_map(width: usize, cages: &[Cage]) -> Square<usize> {
    let mut cage_map = Square::with_width_and_value(width, 0);
    for (i, cage) in cages.iter().enumerate() {
        for &j in &cage.cells {
            cage_map[j] = i;
        }
    }
    cage_map
}
