//! Generate and solve KenKen puzzles

#![cfg_attr(feature="cargo-clippy", allow(doc_markdown))]
#![feature(generators, generator_trait)]
#![feature(ascii_ctype)]
#![feature(conservative_impl_trait)]
#![feature(inclusive_range_syntax)]
#![feature(match_default_bindings)]
#![feature(slice_patterns)]
#![feature(vec_resize_default)]
#![warn(missing_docs)]

#[macro_use]
extern crate log;
extern crate itertools;
extern crate num;
extern crate image;
extern crate rand;
extern crate rusttype;
extern crate fnv;
extern crate linked_hash_map;
extern crate linked_hash_set;

pub mod gen_utils;
pub mod puzzle;

pub use self::puzzle::Puzzle;

mod collections;
