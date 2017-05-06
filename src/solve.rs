extern crate num;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashSet;
use std::collections::HashMap;
use puzzle::Puzzle;
use square::*;
use board::*;
use itertools::Itertools;
use std::mem;
use num::Integer;

#[derive(Clone)]
pub enum Variable {
    Solved(i32),
    Unsolved(Domain),
}

impl Variable {
    /*
    fn is_solved(&self) -> bool {
        match self {
            &Variable::Solved(_) => true,
            _ => false,
        }
    }
    */

    fn is_unsolved(&self) -> bool {
        match self {
            &Variable::Unsolved(_) => true,
            _ => false,
        }
    }

    fn unsolved(&self) -> Option<&Domain> {
        match self {
            &Variable::Unsolved(ref domain) => Some(domain),
            _ => None,
        }
    }

    fn unwrap_unsolved(&self) -> &Domain {
        match self {
            &Variable::Unsolved(ref d) => d,
            _ => panic!("Not Unsolved"),
        }
    }

    /*
    fn unwrap_unsolved_mut(&mut self) -> &mut Domain {
        match self {
            &mut Variable::Unsolved(ref mut d) => d,
            _ => panic!("Not Unsolved"),
        }
    }
    */
}

fn unknown_from_size(size: usize) -> Variable {
    Variable::Unsolved(Domain::new(size))
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
    pos_domain: Vec<Variable>,
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
    pub cells: Square<Variable>,
    pub cages: Vec<CageMarkup>,
    pub cage_map: Square<usize>,
    pub dirty_cages: BTreeSet<usize>,
    //vectors: [Vec<Vector>; 2],
}

impl BoardMarkup {
    fn new(puzzle: &Puzzle) -> BoardMarkup {
        let mut dirty_cages = BTreeSet::new();
        for i in 0..puzzle.cages.len() {
            dirty_cages.insert(i);
        }
        BoardMarkup {
            cells: Square::new(unknown_from_size(puzzle.size), puzzle.size),
            cages: (0..puzzle.cages.len()).map(|i| CageMarkup::new(puzzle, i)).collect_vec(),
            cage_map: puzzle.cage_map(),
            dirty_cages: dirty_cages,
            //vectors: [vec![Vector::new(size); size], vec![Vector::new(size); size]],
        }
    }

    fn remove(&mut self, pos: usize, n: i32) {
        debug!("BoardMarkup::remove({}, {})", pos, n);

        // remove cell candidate
        let (removed, only_value) = match self.cells[pos] {
            Variable::Unsolved(ref mut domain) => {
                (domain.remove(n), domain.only_value())
            },
            _ => (false, None),
        };

        // mark cell solved if one candidate remains
        if let Some(value) = only_value {
            self.solve_cell(pos, value);
        }

        if removed {
            let cage_index = self.cage_map[pos];
            self.dirty_cages.insert(cage_index);
        }

        /*
        // remove vector domain
        for i in 0..2 {
            self.vectors[i][pos[i]].pos_domain[index] = Variable::Solved(pos[i]);
            for j in 0..self.size() {
                if let Variable::Unsolved(ref domain) = self.vectors[i][j] {

                }
            }
        }
        */
    }

    fn solve_cell(&mut self, pos: usize, value: i32) {
        self.cells[pos] = Variable::Solved(value);
        self.on_solve_cell(pos, value);
    }

    fn on_solve_cell(&mut self, pos: usize, value: i32) {
        debug!("BoardMarkup::on_solve_cell({}, {})", pos, value);
        let size = self.cells.size;
        let pos = Coord::from_index(pos, size);
        // remove possibility from cells in same row or column
        for i in 0..size {
            self.remove(Coord::new(i, pos[1]).to_index(size), value);
            self.remove(Coord::new(pos[0], i).to_index(size), value);
        }
    }

    pub fn solved(&self) -> bool {
        self.cells.elements.iter().all(|u| match u {
            &Variable::Solved(_) => true,
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

fn vector_ids(pos: usize, size: usize) -> [i32; 2] {
    [
        (pos / size * 2) as i32,
        (pos % size * 2 + 1) as i32,
    ]
}

fn vector_str(vector_id: i32) -> String {
    let n = vector_id / 2;
    let s = if vector_id % 2 == 0 { "Row" } else { "Col" };
    format!("{} {}", s, n)
}

/**
 * A CageVector is a set of 2 or more cells that are in the same cage
 * and are also in the same row or column (vector). It contains values that
 * are known to be in one of its cells.
 */
pub struct CageVector {
    cells: Vec<usize>,
    values: HashSet<i32>,
}

pub struct CageMarkup {
    vectors: HashMap<i32, CageVector>,
}

impl CageMarkup {
    fn new(puzzle: &Puzzle, cage_index: usize) -> CageMarkup {
        let mut vectors = HashMap::new();
        for &cell in puzzle.cages[cage_index].cells.iter() {
            for &vector_id in vector_ids(cell, puzzle.size).into_iter() {
                let cage_vector = vectors.entry(vector_id).or_insert(CageVector {
                    cells: Vec::new(),
                    values: HashSet::new(),
                });
                cage_vector.cells.push(cell);
            }
        }
        vectors.retain(|_, cage_vector| cage_vector.cells.len() > 1);
        CageMarkup {
            vectors: vectors,
        }
    }
}

pub fn solve(puzzle: &Puzzle) -> BoardMarkup {
    let mut markup = BoardMarkup::new(puzzle);
    
    // clear board
    let mut board = Square::new(0, puzzle.size);

    debug!("solve single cell cages");
    solve_single_cell_cages(&puzzle, &mut markup);

    debug!("reduce cell domain by binary cage operator consistency");
    binary_operator_consistency(&puzzle, &mut markup);

    debug!("reduce domain by cage consistency");
    inner_cages_consistency(&puzzle, &mut markup);

    /*
    println!("Domain:");
    for (pos, cell_markup) in markup.cells.iter() {
        let vals = match cell_markup {
            &Variable::Unsolved(ref domain) => {
                domain.iter().collect_vec()
            },
            &Variable::Solved(val) => vec![val; 1],
        };
        let domain = vals.iter().map(ToString::to_string).join(" ");
        println!("{}: {}", pos, domain);
    }
    */

    for (pos, cell_markup) in markup.cells.iter_coord() {
        if let &Variable::Solved(val) = cell_markup {
            board[pos] = val;
        };
    }

    markup
}

fn solve_single_cell_cages(puzzle: &Puzzle, markup: &mut BoardMarkup) {
    for cage in puzzle.cages.iter().filter(|cage| cage.cells.len() == 1) {
        let index = cage.cells[0];
        markup.solve_cell(index, cage.target)
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

fn inner_cages_consistency(puzzle: &Puzzle, markup: &mut BoardMarkup) {
    while markup.dirty_cages.is_empty() == false {
        let mut best_cage = None;
        for &cage_index in markup.dirty_cages.iter() {
            let cage_rank: i32 = puzzle.cages[cage_index].cells.iter()
                .filter_map(|&cell_index| markup.cells[cell_index].unsolved())
                .map(|domain| domain.count as i32)
                .product();
            if cage_rank == 1 { continue }
            let better = match best_cage {
                Some((_, best_cage_rank)) => cage_rank < best_cage_rank,
                None => true,
            };
            if better {
                best_cage = Some((cage_index, cage_rank));
            }
        }
        let (best_cage_index, _) = best_cage.unwrap();
        if markup.dirty_cages.remove(&best_cage_index) == false {
            panic!("expected {} in dirty cages", best_cage_index)
        }
        inner_cage_consistency(puzzle, markup, best_cage_index);
    }
    /*
    for cage in puzzle.cages.iter()
        .filter(|cage| cage.cells.len() <= max_cage_size)
    {
        if cage.cells.iter().any(|&pos| markup.cells[pos].is_unsolved()) {
        }
    }
    */
}

fn inner_cage_consistency(puzzle: &Puzzle, markup: &mut BoardMarkup, cage_index: usize) {
    match puzzle.cages[cage_index].operator {
        Operator::Add => inner_cage_consistency_add(puzzle, markup, cage_index),
        _ => (),
    }
}

fn inner_cage_consistency_add(puzzle: &Puzzle, markup: &mut BoardMarkup, cage_index: usize) {
    let cage = &puzzle.cages[cage_index];
    let solved_sum: i32 = cage.cells.iter()
        .filter_map(|&i| {
            match markup.cells[i] {
                Variable::Solved(n) => Some(n),
                _ => None,
            }
        })
        .sum();
    let remain_sum = cage.target - solved_sum;
    let unsolved = cage.cells.iter()
        .cloned()
        .filter(|&i| markup.cells[i].is_unsolved())
        .collect_vec();

    // find all solutions for the cage
    let mut solutions = Vec::new();
    let mut solution = vec![0; unsolved.len()];
    find_cage_solutions(0, markup, &unsolved, remain_sum, &mut solution, &mut solutions);

    // assemble domain for each unsolved cell from cell solutions
    let mut domain = vec![vec![false; puzzle.size]; unsolved.len()];
    for solution in solutions.iter() {
        for i in 0..unsolved.len() {
            domain[i][solution[i] as usize - 1] = true;
        }
    }

    // remove values from cell domains that are not in a cage solution
    for i in 0..unsolved.len() {
        let index = unsolved[i];
        let no_solutions = markup.cells[index].unwrap_unsolved().iter()
            .filter(|&n| domain[i][n as usize - 1] == false)
            .collect_vec();
        for n in no_solutions {
            markup.remove(index, n);
        }
    }

    // find cage-vector-values

    let mut vectors = HashMap::new();
    for i in 0..unsolved.len() {
        for &vector_id in vector_ids(unsolved[i], puzzle.size).into_iter() {
            vectors.entry(vector_id).or_insert(BTreeSet::new()).insert(i);
        }
    }
    vectors.retain(|_, unsolved_ids| unsolved_ids.len() > 1);

    for (vector_id, unsolved_ids) in vectors {
        let mut solutions_iter = solutions.iter();
        let solution = solutions_iter.next().unwrap();
        let mut values: BTreeSet<_> = unsolved_ids.iter()
            .map(|&unsolved_id| solution[unsolved_id]).collect();
        for solution in solutions_iter {
            let next_values = unsolved_ids.iter()
                .map(|&unsolved_id| solution[unsolved_id]).collect();
            values = values.intersection(&next_values).cloned().collect();
        }
        let values = values.into_iter().collect_vec();
        debug!("Cage {} {} Values {:?}", cage_index, vector_str(vector_id), values);
    }
}

fn find_cage_solutions(
    i: usize,
    markup: &BoardMarkup,
    pos: &[usize],
    remain_sum: i32,
    solution: &mut [i32],
    solutions: &mut Vec<Vec<i32>>)
{
    let size = markup.cells.size;
    let collides = |n: i32, solution: &[i32]| {
        (0..i).filter(|&j| solution[j] == n)
            .any(|j| have_shared_vector(pos[j], pos[i], size))
    };
    if remain_sum <= 0 { return }
    if i == solution.len() - 1 {
        if remain_sum > markup.cells.size as i32 { return }
        let domain = markup.cells[pos[i]].unwrap_unsolved();
        if domain.has(remain_sum) == false { return }
        if collides(remain_sum, solution) { return }
        solution[i] = remain_sum;
        solutions.push(solution.to_vec());
    } else {
        for n in markup.cells[pos[i]].unwrap_unsolved().iter() {
            if n >= remain_sum { break }
            if collides(n, solution) { continue }
            solution[i] = n;
            find_cage_solutions(i + 1, markup, pos, remain_sum - n, solution, solutions);
        }
    }
}

fn have_shared_vector(pos1: usize, pos2: usize, size: usize) -> bool {
    pos1 / size == pos2 / size || pos1 % size == pos2 % size
}
