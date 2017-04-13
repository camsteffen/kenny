use board::*;
use std::string::ToString;
use itertools::Itertools;
use std::mem;
use num::Integer;

#[derive(Clone)]
enum Cell {
    Solved(i32),
    Unsolved(CellCandidates),
}

#[derive(Clone)]
struct CellCandidates {
    count: usize,
    candidates: Vec<bool>,
}

struct CandidateIter<'a> {
    candidates: &'a [bool],
    i: i32,
}

impl<'a> Iterator for CandidateIter<'a> {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        let pos = match self.candidates.iter().position(|c| *c) {
            Some(pos) => pos,
            None => return None,
        };
        self.i = self.i + pos as i32 + 1;
        let candidates = mem::replace(&mut self.candidates, &mut []);

        let (_, remaining) = candidates.split_at(pos + 1);
        self.candidates = remaining;
        Some(self.i)
    }
}

impl CellCandidates {
    fn new(size: usize) -> CellCandidates {
        CellCandidates {
            count: size,
            candidates: vec![true; size],
        }
    }

    fn not_candidate(&mut self, n: i32) -> bool {
        let i = n as usize - 1;
        if self.candidates[i] {
            self.candidates[i] = false;
            self.count = self.count - 1;
            true
        } else {
            false
        }
    }

    fn value(&self) -> Option<i32> {
        match self.count {
            1 => Some(self.candidates.iter().position(|p| *p).unwrap() as i32 + 1),
            _ => None,
        }
    }

    fn iter_candidates<'a>(&'a self) -> CandidateIter<'a> {
        CandidateIter {
            candidates: &self.candidates,
            i: 0,
        }
    }
}

struct BoardMarkup {
    cells: Square<Cell>,
}

impl BoardMarkup {
    fn new(size: usize) -> BoardMarkup {
        BoardMarkup {
            cells: Square::new(Cell::Unsolved(CellCandidates::new(size)), size),
        }
    }

    fn not_candidate(&mut self, pos: &Coord, n: i32) {
        debug!("BoardMarkup::not_candidate({:?}, {})", pos, n);
        let value = match self.cells[pos] {
            Cell::Unsolved(ref mut candidates) => {
                candidates.not_candidate(n);
                candidates.value()
            },
            _ => None,
        };
        if let Some(value) = value {
            self.cells[pos] = Cell::Solved(value);
            self.on_set_value(pos, n);
        }
    }

    fn set_value(&mut self, pos: &Coord, n: i32) {
        self.cells[pos] = Cell::Solved(n);
        self.on_set_value(pos, n);
    }

    fn on_set_value(&mut self, pos: &Coord, n: i32) {
        debug!("BoardMarkup::on_set_value({:?}, {})", pos, n);
        // remove possibility from cells in same row or column
        for i in 0..self.cells.size {
            self.not_candidate(&Coord::new(i, pos[1]), n);
            self.not_candidate(&Coord::new(pos[0], i), n);
        }
    }
}

pub fn solve(cages: &Vec<Cage>, size: usize) -> Board {
    let mut markup = BoardMarkup::new(size);
    
    // clear board
    let mut board = Square::new(0, size);

    debug!("solve single cell cages");
    solve_single_cell_cages(&board, &cages, &mut markup);
    println!("reduce cell candidates by cage operators");
    candidates_by_operator(&board, &cages, &mut markup);

    println!("Candidates:");
    for (pos, cell_markup) in markup.cells.iter() {
        let vals = match cell_markup {
            &Cell::Unsolved(ref candidates) => {
                candidates.iter_candidates().collect_vec()
            },
            &Cell::Solved(val) => vec![val; 1],
        };
        let candidates = vals.iter().map(ToString::to_string).join(" ");
        println!("{:?}: {}", pos, candidates);
    }

    for (pos, cell_markup) in markup.cells.iter() {
        if let &Cell::Solved(val) = cell_markup {
            board[&pos] = val;
        };
    }

    board
}

fn solve_single_cell_cages(board: &Board, cages: &Vec<Cage>, markup: &mut BoardMarkup) {
    for cage in cages.iter().filter(|cage| cage.cells.len() == 1) {
        let index = cage.cells[0];
        let pos = Coord::from_index(index, board.size);
        markup.set_value(&pos, cage.target)
    }
}

fn candidates_by_operator(board: &Board, cages: &Vec<Cage>, markup: &mut BoardMarkup) {
    for cage in cages.iter() {
        match &cage.operator {
            &Operator::Multiply => {
                let non_factors = (1..board.size as i32 - 1).filter(|n| !cage.target.is_multiple_of(n));
                for n in non_factors {
                    for pos in cage.cells.iter().map(|i| Coord::from_index(*i, board.size)) {
                        markup.not_candidate(&pos, n);
                    }
                }
            },
            &Operator::Divide => {
                let non_multiples = (1..board.size as i32 - 1).filter(|n| !n.is_multiple_of(&cage.target));
                for n in non_multiples {
                    for pos in cage.cells.iter().map(|i| Coord::from_index(*i, board.size)) {
                        markup.not_candidate(&pos, n);
                    }
                }
            },
            _ => {},
        }
    }

        /*
        let pos = cage.cells[0];
        markup.set_value(Coord::from_index(pos, board.size), cage.target)
        */
}

