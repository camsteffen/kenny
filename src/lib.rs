//! Generate and solve KenKen puzzles

#![cfg_attr(feature="cargo-clippy", allow(doc_markdown))]
#![feature(ascii_ctype)]
#![feature(conservative_impl_trait)]
#![feature(inclusive_range_syntax)]
#![feature(slice_patterns)]
#![feature(vec_resize_default)]
#![warn(missing_docs)]

#[macro_use] extern crate log;
extern crate itertools;
extern crate num;
extern crate png;
extern crate rand;
extern crate rusttype;

pub mod square;
pub mod puzzle;
pub mod image;
pub mod parse;

mod range_domain;
mod solve;
