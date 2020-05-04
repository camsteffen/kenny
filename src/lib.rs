//! Generate and solve KenKen puzzles

#![warn(rust_2018_idioms)]
#![warn(single_use_lifetimes)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_qualifications)]

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
// todo prioritize and re_order constraint set by usage data
// todo remove fallible dependency (deprecated)
// todo add option for search_required puzzles
// todo add test cases for "no search required"
