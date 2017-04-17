extern crate num;

use square::*;
use board::*;
use std::string::ToString;
use itertools::Itertools;
use std::mem;
use num::Integer;

#[derive(Clone)]
pub enum Unknown {
    Solved(i32),
    Unsolved(Candidates),
}

fn unknown_from_size(size: usize) -> Unknown {
    Unknown::Unsolved(Candidates::new(size))
}

#[derive(Clone)]
pub struct Candidates {
    pub count: usize,
    pub candidates: Vec<bool>,
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

impl Candidates {
    fn new(size: usize) -> Candidates {
        Candidates {
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

    fn only_value(&self) -> Option<i32> {
        match self.count {
            1 => Some(self.candidates.iter().position(|p| *p).unwrap() as i32 + 1),
            _ => None,
        }
    }

    pub fn iter_candidates<'a>(&'a self) -> CandidateIter<'a> {
        CandidateIter {
            candidates: &self.candidates,
            i: 0,
        }
    }
}

#[derive(Clone)]
struct Vector {
    pos_candidates: Vec<Unknown>,
}

impl Vector {
    fn new(size: usize) -> Vector {
        Vector {
            pos_candidates: vec![unknown_from_size(size); size],
        }
    }
}

pub struct BoardMarkup {
    pub cells: Square<Unknown>,
    //vectors: [Vec<Vector>; 2],
}

impl BoardMarkup {
    fn new(size: usize) -> BoardMarkup {
        BoardMarkup {
            cells: Square::new(unknown_from_size(size), size),
            //vectors: [vec![Vector::new(size); size], vec![Vector::new(size); size]],
        }
    }

    fn not_candidate(&mut self, pos: Coord, n: i32) {
        debug!("BoardMarkup::not_candidate({}, {})", pos, n);

        // remove cell candidate
        let only_value = match self.cells[pos] {
            Unknown::Unsolved(ref mut candidates) => {
                candidates.not_candidate(n);
                candidates.only_value()
            },
            _ => None,
        };

        // mark cell solved if one candidate remains
        if let Some(value) = only_value {
            self.cells[pos] = Unknown::Solved(value);
            self.on_set_value(pos, n);
        }

        /*
        // remove vector candidates
        for i in 0..2 {
            self.vectors[i][pos[i]].pos_candidates[index] = Unknown::Solved(pos[i]);
            for j in 0..self.size() {
                if let Unknown::Unsolved(ref candidates) = self.vectors[i][j] {

                }
            }
        }
        */
    }

    fn size(&self) -> usize {
        self.cells.size
    }

    fn set_value(&mut self, pos: Coord, n: i32) {
        self.cells[pos] = Unknown::Solved(n);
        self.on_set_value(pos, n);
    }

    fn on_set_value(&mut self, pos: Coord, n: i32) {
        debug!("BoardMarkup::on_set_value({}, {})", pos, n);
        // remove possibility from cells in same row or column
        for i in 0..self.cells.size {
            self.not_candidate(Coord::new(i, pos[1]), n);
            self.not_candidate(Coord::new(pos[0], i), n);
        }
    }

    pub fn solved(&self) -> bool {
        self.cells.elements.iter().all(|u| match u {
            &Unknown::Solved(_) => true,
            _ => false,
        })
    }

    /*
    fn vectors_at_pos<'a>(&'a self, pos: Coord) -> [&'a Vector; 2] {
        [&self.vectors[pos[0]], &self.vectors[self.cells.size + pos[1]]]
    }

    fn vectors_at_pos_mut<'a>(&'a mut self, pos: Coord) -> [&'a mut Vector; 2] {
        let vectors = &mut self.vectors;
        let(a, b) = vectors.split_at_mut(pos[0] + 1);
        [&mut a[pos[0]], &mut b[pos[1]]
    }
    */
}

pub fn solve(cages: &Vec<Cage>, size: usize) -> BoardMarkup {
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
            &Unknown::Unsolved(ref candidates) => {
                candidates.iter_candidates().collect_vec()
            },
            &Unknown::Solved(val) => vec![val; 1],
        };
        let candidates = vals.iter().map(ToString::to_string).join(" ");
        println!("{}: {}", pos, candidates);
    }

    for (pos, cell_markup) in markup.cells.iter() {
        if let &Unknown::Solved(val) = cell_markup {
            board[pos] = val;
        };
    }

    markup
}

fn solve_single_cell_cages(board: &Board, cages: &Vec<Cage>, markup: &mut BoardMarkup) {
    for cage in cages.iter().filter(|cage| cage.cells.len() == 1) {
        let index = cage.cells[0];
        let pos = Coord::from_index(index, board.size);
        markup.set_value(pos, cage.target)
    }
}

fn candidates_by_operator(board: &Board, cages: &Vec<Cage>, markup: &mut BoardMarkup) {
    for cage in cages.iter() {
        match &cage.operator {
            &Operator::Add => {
                for pos in cage.iter_cell_pos(board.size) {
                    for n in (cage.target + 1)..(board.size as i32 + 1) {
                        markup.not_candidate(pos, n);
                    }
                }
            },
            &Operator::Subtract => (),
            &Operator::Multiply => {
                let non_factors = (1..board.size as i32 - 1)
                    .filter(|n| !cage.target.is_multiple_of(n))
                    .collect_vec();
                for pos in cage.iter_cell_pos(board.size) {
                    for n in non_factors.iter() {
                        markup.not_candidate(pos, *n);
                    }
                }
            },
            &Operator::Divide => {
                let non_multiples = (1..board.size as i32 - 1)
                    .filter(|n| !n.is_multiple_of(&cage.target))
                    .collect_vec();
                for pos in cage.iter_cell_pos(board.size) {
                    for n in non_multiples.iter() {
                        markup.not_candidate(pos, *n);
                    }
                }
            },
        }
    }

        /*
        let pos = cage.cells[0];
        markup.set_value(Coord::from_index(pos, board.size), cage.target)
        */
}

