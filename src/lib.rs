//! Generate and solve KenKen puzzles

#![cfg_attr(feature="cargo-clippy", allow(doc_markdown))]
#![feature(generators, generator_trait)]
#![feature(ascii_ctype)]
#![feature(slice_patterns)]
#![feature(vec_resize_default)]

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
