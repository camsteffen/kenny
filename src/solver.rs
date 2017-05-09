extern crate num;

use cell_domain::CellDomain;
use variable::Variable;
use std::collections::BTreeSet;
use std::collections::HashSet;
use std::collections::HashMap;
use puzzle::{Puzzle, Operator};
use square::{Square, Coord};
use itertools::Itertools;
use num::Integer;

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

pub struct Solver<'a> {
    puzzle: &'a Puzzle,
    pub cells: Square<Variable>,
    pub cages: Vec<CageMarkup>,
    pub cage_map: Square<usize>,
    pub dirty_cages: HashSet<usize>,
    //vectors: [Vec<Vector>; 2],
}

impl<'a> Solver<'a> {
    pub fn new(puzzle: &Puzzle) -> Solver {
        let dirty_cages = (0..puzzle.cages.len()).collect();
        Solver {
            puzzle: puzzle,
            cells: Square::new(Variable::unsolved_with_all(puzzle.size), puzzle.size),
            //cages: (0..puzzle.cages.len()).map(|i| CageMarkup::new(puzzle, i)).collect_vec(),
            cages: vec![CageMarkup::new(); puzzle.cages.len()],
            cage_map: puzzle.cage_map(),
            dirty_cages: dirty_cages,
            //vectors: [vec![Vector::new(size); size], vec![Vector::new(size); size]],
        }
    }

    pub fn solve(&mut self) {
        // clear board
        let mut board = Square::new(0, self.puzzle.size);

        debug!("solve single cell cages");
        self.solve_single_cell_cages();

        debug!("reduce cell domain by binary cage operator consistency");
        self.binary_operator_consistency();

        debug!("reduce domain by cage consistency");
        self.inner_cages_consistency();

        /*
        println!("Domain:");
        for (pos, cell_markup) in solver.cells.iter() {
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

        for (pos, cell_markup) in self.cells.iter_coord() {
            if let &Variable::Solved(val) = cell_markup {
                board[pos] = val;
            };
        }
    }

    fn remove(&mut self, pos: usize, n: i32) {
        debug!("Solver::remove({}, {})", pos, n);

        // remove cell candidate
        let (removed, solution) = match self.cells[pos] {
            Variable::Unsolved(ref mut domain) => {
                let removed = domain.remove(n);
                let solution = domain.single_value();
                (removed, solution)
            },
            _ => (false, None),
        };

        // mark cell solved if one candidate remains
        if let Some(value) = solution {
            self.solve_cell(pos, value);
        } else if removed {
            let cage_index = self.cage_map[pos];
            self.on_update_cage(cage_index);
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
        let cage_index = self.cage_map[pos];
        self.on_update_cage(cage_index);
    }

    fn on_solve_cell(&mut self, pos: usize, value: i32) {
        debug!("Solver::on_solve_cell({}, {})", pos, value);
        let size = self.cells.size;
        let pos = Coord::from_index(pos, size);

        // remove possibility from cells in same row or column
        for i in 0..size {
            self.remove(Coord::new(i, pos[1]).to_index(size), value);
            self.remove(Coord::new(pos[0], i).to_index(size), value);
        }
    }

    fn on_update_cage(&mut self, cage_index: usize) {
        if self.puzzle.cages[cage_index].cells.iter()
            .all(|&p| self.cells[p].is_solved())
        {
            self.dirty_cages.remove(&cage_index);
        } else {
            self.dirty_cages.insert(cage_index);
        }
    }

    pub fn solved(&self) -> bool {
        self.cells.elements.iter().all(|u| match u {
            &Variable::Solved(_) => true,
            _ => false,
        })
    }

    fn solve_single_cell_cages(&mut self) {
        for cage in self.puzzle.cages.iter().filter(|cage| cage.cells.len() == 1) {
            let index = cage.cells[0];
            self.solve_cell(index, cage.target)
        }
    }

    fn binary_operator_consistency(&mut self) {
        for cage in self.puzzle.cages.iter() {
            match &cage.operator {
                &Operator::Add => {
                    for &pos in cage.cells.iter() {
                        for n in (cage.target + 1)..(self.puzzle.size as i32 + 1) {
                            self.remove(pos, n);
                        }
                    }
                },
                &Operator::Multiply => {
                    let non_factors = (1..self.puzzle.size as i32 - 1)
                        .filter(|n| !cage.target.is_multiple_of(n))
                        .collect_vec();
                    for &pos in cage.cells.iter() {
                        for n in non_factors.iter() {
                            self.remove(pos, *n);
                        }
                    }
                },
                _ => (),
            }
        }
    }

    fn inner_cages_consistency(&mut self) {
        let mut to_remove = Vec::new();
        while self.dirty_cages.is_empty() == false {
            let mut best_cage = None;
            for &cage_index in self.dirty_cages.iter() {
                let cage_rank: i32 = self.puzzle.cages[cage_index].cells.iter()
                    .filter_map(|&cell_index| self.cells[cell_index].unsolved())
                    .map(|domain| domain.len() as i32)
                    .product();
                if cage_rank == 1 {
                    panic!("dirty cage is solved");
                    /*
                    to_remove.push(cage_index);
                    continue
                    */
                }
                let better = match best_cage {
                    Some((_, best_cage_rank)) => cage_rank < best_cage_rank,
                    None => true,
                };
                if better {
                    best_cage = Some((cage_index, cage_rank));
                }
            }
            let (best_cage_index, _) = best_cage.unwrap();
            to_remove.push(best_cage_index);
            for index in to_remove.drain(..) {
                if self.dirty_cages.remove(&index) == false {
                    panic!("expected {} in dirty cages", index)
                }
            }
            self.inner_cage_consistency(best_cage_index);
        }
        /*
        for cage in puzzle.cages.iter()
            .filter(|cage| cage.cells.len() <= max_cage_size)
        {
            if cage.cells.iter().any(|&pos| solver.cells[pos].is_unsolved()) {
            }
        }
        */
    }

    fn inner_cage_consistency(&mut self, cage_index: usize) {
        match self.puzzle.cages[cage_index].operator {
            Operator::Add => self.inner_cage_consistency_add(cage_index),
            _ => (),
        }
    }

    fn inner_cage_consistency_add(&mut self, cage_index: usize) {
        let cage = &self.puzzle.cages[cage_index];
        let solved_sum: i32 = cage.cells.iter()
            .filter_map(|&i| self.cells[i].solved())
            .sum();
        let remain_sum = cage.target - solved_sum;
        let mut unsolved = cage.cells.iter()
            .cloned()
            .filter(|&i| self.cells[i].is_unsolved())
            .collect_vec();

        // find all solutions for the cage
        let mut solutions = Vec::new();
        let mut solution = vec![0; unsolved.len()];
        self.find_cage_solutions(0, &unsolved, remain_sum, &mut solution, &mut solutions);

        // assemble domain for each unsolved cell from cell solutions
        let mut soln_domain = vec![CellDomain::with_none(self.puzzle.size); unsolved.len()];
        for solution in solutions.iter() {
            for i in 0..unsolved.len() {
                soln_domain[i].insert(solution[i]);
            }
        }

        // remove values from cell domains that are not in a cage solution
        let mut to_remove = Vec::new();
        for i in 0..unsolved.len() {
            let index = unsolved[i];
            let no_solutions;
            {
                let domain = match self.cells[index].unsolved() {
                    Some(domain) => domain,
                    None => {
                        to_remove.push(i);
                        continue
                    },
                };
                no_solutions = domain.iter()
                    .filter(|&n| soln_domain[i].contains(n) == false)
                    .collect_vec();
            }
            for n in no_solutions {
                self.remove(index, n);
            }
        }
        if to_remove.is_empty() == false {
            let mut to_remove = to_remove.iter().peekable();
            unsolved = unsolved.into_iter()
                .enumerate()
                .filter(|&(ref i, _)| {
                    if to_remove.peek().map_or(false, |&r| r == i) {
                        to_remove.next();
                        false
                    } else {
                        true
                    }
                })
                .map(|(_, i)| i)
                .collect()
        }

        let mut vectors = HashMap::new();
        for i in 0..unsolved.len() {
            for &vector_id in vector_ids(unsolved[i], self.puzzle.size).into_iter() {
                vectors.entry(vector_id).or_insert(BTreeSet::new()).insert(i);
            }
        }
        vectors.retain(|_, unsolved_ids| unsolved_ids.len() > 1);

        for (vector_id, unsolved_ids) in vectors {
            let mut solutions_iter = solutions.iter();
            let solution = solutions_iter.next().unwrap();
            let mut values: HashSet<i32> = unsolved_ids.iter()
                .map(|&unsolved_id| solution[unsolved_id])
                .filter(|n| {
                    self.cages[cage_index].vector_vals.get(&vector_id)
                        .map_or(true, |vals| vals.contains(n) == false)
                })
                .collect();
            for solution in solutions_iter {
                if values.is_empty() { break }
                let next_values = unsolved_ids.iter()
                    .map(|&unsolved_id| solution[unsolved_id])
                    .filter(|n| values.contains(n))
                    .collect();
                values = next_values;
                //mem::swap(&mut values, &mut next_values);
            }
            self.cages[cage_index].vector_vals.entry(vector_id)
                .or_insert(HashSet::new())
                .extend(values.iter());
            for n in values {
                for pos in self.iter_vector(vector_id) {
                    if self.cage_map[pos] == cage_index { break }
                    self.remove(pos, n);
                }
            }
        }
    }

    fn find_cage_solutions(
        &self,
        i: usize,
        pos: &[usize],
        remain_sum: i32,
        solution: &mut [i32],
        solutions: &mut Vec<Vec<i32>>)
    {
        let size = self.puzzle.size;
        let collides = |n: i32, vals: &[i32]| {
            (0..i).filter(|&j| vals[j] == n)
                .any(|j| have_shared_vector(pos[j], pos[i], size))
        };
        if remain_sum <= 0 { return }
        if i == solution.len() - 1 {
            if remain_sum > self.cells.size as i32 { return }
            let domain = self.cells[pos[i]].unwrap_unsolved();
            if domain.contains(remain_sum) == false { return }
            if collides(remain_sum, &solution[..i]) { return }
            solution[i] = remain_sum;
            solutions.push(solution.to_vec());
        } else {
            for n in self.cells[pos[i]].unwrap_unsolved().iter() {
                if n >= remain_sum { break }
                if collides(n, &solution[..i]) { continue }
                solution[i] = n;
                self.find_cage_solutions(i + 1, pos, remain_sum - n, solution, solutions);
            }
        }
    }

    fn iter_vector(&self, vector_id: i32) -> Box<Iterator<Item=usize>> {
        let vector_id = vector_id as usize;
        if vector_id % 2 == 0 {
            let start = vector_id / 2 * self.puzzle.size;
            Box::new((0..self.puzzle.size).map(move |i| start + i))
        } else {
            let start = vector_id / 2;
            let inc = self.puzzle.size;
            Box::new((0..self.puzzle.size).map(move |i| start + inc * i))
        }
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

/*
fn vector_str(vector_id: i32) -> String {
    let n = vector_id / 2;
    let s = if vector_id % 2 == 0 { "Row" } else { "Col" };
    format!("{} {}", s, n)
}
*/

/*
/**
 * A CageVector is a set of 2 or more cells that are in the same cage
 * and are also in the same row or column (vector). It contains values that
 * are known to be in one of its cells.
 */
pub struct CageVector {
    cells: Vec<usize>,
    values: HashSet<i32>,
}
*/

#[derive(Clone)]
pub struct CageMarkup {
    //vectors: HashMap<i32, CageVector>,
    vector_vals: HashMap<i32, HashSet<i32>>,
}

impl CageMarkup {
    fn new(/*puzzle: &Puzzle, cage_index: usize*/) -> CageMarkup {
        /*
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
        */
        CageMarkup {
            //vectors: vectors,
            vector_vals: HashMap::new(),
        }
    }
}

fn have_shared_vector(pos1: usize, pos2: usize, size: usize) -> bool {
    pos1 / size == pos2 / size || pos1 % size == pos2 % size
}
