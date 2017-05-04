extern crate num;

use puzzle::Puzzle;
use square::*;
use board::*;
use itertools::Itertools;
use std::mem;
use num::Integer;

#[derive(Clone)]
pub enum Unknown {
    Solved(i32),
    Unsolved(Domain),
}

impl Unknown {
    /*
    fn is_solved(&self) -> bool {
        match self {
            &Unknown::Solved(_) => true,
            _ => false,
        }
    }
    */

    fn is_unsolved(&self) -> bool {
        match self {
            &Unknown::Unsolved(_) => true,
            _ => false,
        }

    }

    fn unwrap_unsolved(&self) -> &Domain {
        match self {
            &Unknown::Unsolved(ref d) => d,
            _ => panic!("Not Unsolved"),
        }
    }

    fn unwrap_unsolved_mut(&mut self) -> &mut Domain {
        match self {
            &mut Unknown::Unsolved(ref mut d) => d,
            _ => panic!("Not Unsolved"),
        }
    }
}

fn unknown_from_size(size: usize) -> Unknown {
    Unknown::Unsolved(Domain::new(size))
}

#[derive(Clone)]
pub struct Domain {
    pub count: usize,
    pub domain: Vec<bool>,
}

pub struct DomainIter<'a> {
    domain: &'a [bool],
    i: i32,
}

impl<'a> Iterator for DomainIter<'a> {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        let pos = match self.domain.iter().position(|c| *c) {
            Some(pos) => pos,
            None => return None,
        };
        self.i = self.i + pos as i32 + 1;
        let domain = mem::replace(&mut self.domain, &mut []);

        let (_, remaining) = domain.split_at(pos + 1);
        self.domain = remaining;
        Some(self.i)
    }
}

impl Domain {
    fn new(size: usize) -> Domain {
        Domain {
            count: size,
            domain: vec![true; size],
        }
    }

    fn remove(&mut self, n: i32) -> bool {
        let i = n as usize - 1;
        if self.domain[i] {
            self.domain[i] = false;
            self.count = self.count - 1;
            true
        } else {
            false
        }
    }

    fn has(&self, n: i32) -> bool {
        self.domain[n as usize - 1]
    }

    fn only_value(&self) -> Option<i32> {
        match self.count {
            1 => Some(self.domain.iter().position(|p| *p).unwrap() as i32 + 1),
            _ => None,
        }
    }

    pub fn iter<'a>(&'a self) -> DomainIter<'a> {
        DomainIter {
            domain: &self.domain,
            i: 0,
        }
    }
}

/*
#[derive(Clone)]
struct Vector {
    pos_domain: Vec<Unknown>,
}

impl Vector {
    fn new(size: usize) -> Vector {
        Vector {
            pos_domain: vec![unknown_from_size(size); size],
        }
    }
}
*/

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

    fn remove(&mut self, pos: usize, n: i32) {
        debug!("BoardMarkup::remove({}, {})", pos, n);

        // remove cell candidate
        let only_value = match self.cells[pos] {
            Unknown::Unsolved(ref mut domain) => {
                domain.remove(n);
                domain.only_value()
            },
            _ => None,
        };

        // mark cell solved if one candidate remains
        if let Some(value) = only_value {
            self.cells[pos] = Unknown::Solved(value);
            self.on_set_value(pos, n);
        }

        /*
        // remove vector domain
        for i in 0..2 {
            self.vectors[i][pos[i]].pos_domain[index] = Unknown::Solved(pos[i]);
            for j in 0..self.size() {
                if let Unknown::Unsolved(ref domain) = self.vectors[i][j] {

                }
            }
        }
        */
    }

    fn set_value(&mut self, pos: usize, n: i32) {
        self.cells[pos] = Unknown::Solved(n);
        self.on_set_value(pos, n);
    }

    fn on_set_value(&mut self, pos: usize, n: i32) {
        debug!("BoardMarkup::on_set_value({}, {})", pos, n);
        let size = self.cells.size;
        let pos = Coord::from_index(pos, size);
        // remove possibility from cells in same row or column
        for i in 0..size {
            self.remove(Coord::new(i, pos[1]).to_index(size), n);
            self.remove(Coord::new(pos[0], i).to_index(size), n);
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

pub fn solve(puzzle: &Puzzle) -> BoardMarkup {
    let mut markup = BoardMarkup::new(puzzle.size);
    
    // clear board
    let mut board = Square::new(0, puzzle.size);

    debug!("solve single cell cages");
    solve_single_cell_cages(&puzzle, &mut markup);

    debug!("reduce cell domain by binary cage operator consistency");
    binary_operator_consistency(&puzzle, &mut markup);

    debug!("reduce domain by cage consistency");
    inner_cages_consistency(&puzzle, &mut markup, 4);

    /*
    println!("Domain:");
    for (pos, cell_markup) in markup.cells.iter() {
        let vals = match cell_markup {
            &Unknown::Unsolved(ref domain) => {
                domain.iter().collect_vec()
            },
            &Unknown::Solved(val) => vec![val; 1],
        };
        let domain = vals.iter().map(ToString::to_string).join(" ");
        println!("{}: {}", pos, domain);
    }
    */

    for (pos, cell_markup) in markup.cells.iter_coord() {
        if let &Unknown::Solved(val) = cell_markup {
            board[pos] = val;
        };
    }

    markup
}

fn solve_single_cell_cages(puzzle: &Puzzle, markup: &mut BoardMarkup) {
    for cage in puzzle.cages.iter().filter(|cage| cage.cells.len() == 1) {
        let index = cage.cells[0];
        markup.set_value(index, cage.target)
    }
}

fn binary_operator_consistency(puzzle: &Puzzle, markup: &mut BoardMarkup) {
    for cage in puzzle.cages.iter() {
        match &cage.operator {
            &Operator::Add => {
                for &pos in cage.cells.iter() {
                    for n in (cage.target + 1)..(puzzle.size as i32 + 1) {
                        markup.remove(pos, n);
                    }
                }
            },
            &Operator::Multiply => {
                let non_factors = (1..puzzle.size as i32 - 1)
                    .filter(|n| !cage.target.is_multiple_of(n))
                    .collect_vec();
                for &pos in cage.cells.iter() {
                    for n in non_factors.iter() {
                        markup.remove(pos, *n);
                    }
                }
            },
            _ => (),
        }
    }
}

fn inner_cages_consistency(puzzle: &Puzzle, markup: &mut BoardMarkup, max_cage_size: usize) {
    for cage in puzzle.cages.iter()
        .filter(|cage| cage.cells.len() <= max_cage_size)
    {
        if cage.cells.iter().any(|&pos| markup.cells[pos].is_unsolved()) {
            inner_cage_consistency(puzzle, markup, cage);
        }
    }
}

fn inner_cage_consistency(puzzle: &Puzzle, markup: &mut BoardMarkup, cage: &Cage) {
    match cage.operator {
        Operator::Add => inner_cage_consistency_add(puzzle, markup, cage),
        _ => (),
    }
}

fn inner_cage_consistency_add(puzzle: &Puzzle, markup: &mut BoardMarkup, cage: &Cage) {
    let solved_sum: i32 = cage.cells.iter()
        .filter_map(|&i| {
            match markup.cells[i] {
                Unknown::Solved(n) => Some(n),
                _ => None,
            }
        })
        .sum();
    let remain_sum = cage.target - solved_sum;
    let unsolved = cage.cells.iter()
        .cloned()
        .filter(|&i| markup.cells[i].is_unsolved())
        .collect_vec();
    let mut verified_domain = vec![vec![false; puzzle.size + 1]; unsolved.len()];
    let mut solution = vec![0; unsolved.len() - 1];
    for i in 0..unsolved.len() {
        let index = unsolved[i];
        for n in markup.cells[index].unwrap_unsolved_mut().clone().iter() {
            if verified_domain[i][n as usize] { continue }
            let mut others = Vec::with_capacity(unsolved.len() - 1);
            for j in (0..unsolved.len()).filter(|&j| j != i) {
                others.push(unsolved[j]);
            }
            //let others = unsolved.iter().enumerate().filter(|&(j, n)| j != i).map(|(j, &n)| n).collect_vec();
            if find_solution(markup, &others, remain_sum - n, &mut solution) {
                for j in 0..others.len() {
                    verified_domain[j + 1][solution[j] as usize] = true;
                }
            } else {
                markup.remove(index, n);
            }

            /*
            let mut d = (0..unsolved.len())
                .filter(|&j| j != i)
                .map(|j| markup.cells[j].unwrap_unsolved().iter().peekable())
                .collect_vec();
            // TODO BinaryHeap with peekable domain iterators
            d.sort_by_key(|&mut iter| iter.peek().cloned().unwrap_or(i32::max_value()))
            */
        }
    }
}

fn find_solution(markup: &BoardMarkup, pos: &[usize], remain_sum: i32, solution: &mut [i32]) -> bool {
    if remain_sum <= 0 {
        false
    } else if solution.len() == 1 {
        if remain_sum > markup.cells.size as i32 { return false }
        if markup.cells[pos[0]].unwrap_unsolved().has(remain_sum) {
            solution[0] = remain_sum;
            true
        } else {
            false
        }
    } else {
        for n in markup.cells[pos[0]].unwrap_unsolved().iter() {
            if n >= remain_sum { break }
            else {
                solution[0] = n;
                if find_solution(markup, &pos[1..], remain_sum - n, &mut solution[1..]) {
                    return true
                }
            }
        }
        false
    }
}

