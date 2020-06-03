//! Generate and solve KenKen puzzles

#![warn(rust_2018_idioms)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_qualifications)]

// uncomment for pedantic check
/*
#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::filter_map)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
*/

#[macro_use]
extern crate log;

pub mod collections;
pub mod puzzle;

// mod experimental;

// todo unit tests
// todo documentation
// todo license
// todo investigate KenKen license
// todo identify constraints that "require" other constraints to be applied first
// todo consider making cage solutions "lazy" or somehow prevent recording too many cage solutions, maybe start with smaller cages or those with fewer unsolved cells or fewer vectors
// todo prioritize and re_order constraint set by usage data
// todo add test cases for puzzles with and without backtracking required
// todo determine puzzle difficulty levels
