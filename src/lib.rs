//! Generate and solve KenKen puzzles

#![warn(trivial_numeric_casts)]

#[macro_use]
extern crate log;
extern crate itertools;
extern crate num;
extern crate image;
extern crate rand;
extern crate rusttype;
extern crate linked_hash_map;
extern crate linked_hash_set;

pub mod puzzle;

pub use self::puzzle::Puzzle;

mod collections;
