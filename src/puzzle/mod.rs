//! KenKen puzzles

pub use puzzle::*;

pub mod solve;

pub mod error;
mod generate;
pub mod image;
mod parse;
mod puzzle;

use crate::collections::square::Square;

pub type CageId = usize;
pub type CellId = usize;
pub type Value = i32;
pub type Solution = Square<Value>;
