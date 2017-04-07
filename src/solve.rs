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

struct BoardMarkup {
    cells: Vec<CellMarkup>,
}

impl BoardMarkup {
    fn from_board(board: &Board) -> BoardMarkup {
        BoardMarkup {
            cells: repeat(CellMarkup::new(board.size)).take(board.size.pow(2)).collect_vec(),
        }
    }
}

pub fn solve(board: &Board) {
    let markup = BoardMarkup::from_board(board);
    

    // single cell cages
    //board.cages.filter(
}

