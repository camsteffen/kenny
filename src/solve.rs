extern crate itertools;
use itertools::Itertools;
use board::*;
use std::iter::repeat;

#[derive(Clone)]
struct CellMarkup {
    possible: Vec<bool>,
}

impl CellMarkup {
    fn new(size: usize) -> CellMarkup {
        CellMarkup {
            possible: vec![true; size],
        }
    }
}

pub fn solve(cages: &Vec<Cage>, size: usize) {
    let markup = Square::new(CellMarkup::new(size), size);
    
    // clear board
    let cleared = Square::new(0, size);

    // single cell cages
    //board.cages.filter(
}

