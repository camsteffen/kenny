//! KenKen puzzles

pub use self::image::PuzzleImageBuilder;

pub mod solve;

pub mod error;
mod generate;
mod image;
mod parse;
mod puzzle;

use crate::collections::Square;
pub use puzzle::*;

pub type CageId = usize;
pub type CellId = usize;
pub type Value = i32;
pub type Solution = Square<Value>;
