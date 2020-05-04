//! Generate and solve KenKen puzzles

#![warn(trivial_numeric_casts)]

#[macro_use]
extern crate log;

pub use self::puzzle::Puzzle;

pub mod puzzle;

mod collections;

// todo unit tests
// todo documentation
// todo license
// todo investigate KenKen license
// todo integrate "preemptive set"
// todo see if any constraint is redundant with another constraint, especially preemptive sets
// todo identify constraints that "require" other constraints to be applied first
// todo prioritize and re-order constraint set by usage data
// todo remove fallible dependency (deprecated)
// todo add option for search-required puzzles
// todo add test cases for "no search required"
