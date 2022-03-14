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

#[cfg(debug_assertions)]
use ahash::AHasher;
use ahash::{AHashMap, AHashSet};
#[cfg(debug_assertions)]
use std::hash::BuildHasherDefault;

pub mod collections;
pub mod error;
pub mod image;
pub mod puzzle;
pub mod solve;

mod generate;
mod parse;

// enable default hasher for debugging to remove randomness
#[cfg(debug_assertions)]
type HashMap<K, V> = AHashMap<K, V, BuildHasherDefault<AHasher>>;
#[cfg(debug_assertions)]
type HashSet<T> = AHashSet<T, BuildHasherDefault<AHasher>>;
#[cfg(not(debug_assertions))]
type HashMap<K, V> = AHashMap<K, V>;
#[cfg(not(debug_assertions))]
type HashSet<T> = AHashSet<T>;

// todo documentation
// todo lazily initialize cage solutions as needed, starting with smaller cages
// todo prioritize and re_order constraint set by usage data
// todo determine puzzle difficulty levels
