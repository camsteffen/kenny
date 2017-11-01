//! Generate and solve KenKen puzzles

#![feature(inclusive_range_syntax)]
#![feature(slice_patterns)]
#![warn(missing_docs)]
#![cfg_attr(feature="cargo-clippy", allow(doc_markdown))]

#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;
extern crate itertools;
extern crate num;
extern crate png;
extern crate rand;
extern crate rusttype;

pub mod square;
pub mod puzzle;
pub mod image;

mod cell_domain;
mod solve;

