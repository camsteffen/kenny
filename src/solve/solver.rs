extern crate num;

use super::CellDomain;
use itertools::Itertools;
use num::Integer;
use puzzle::Operator;
use puzzle::Puzzle;
use collections::square::{Coord, Square, SquareIndex};
use collections::square::vector::VectorId;
use collections::square::vector::vectors_intersecting_at;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashSet;
use puzzle::Cage;
use super::vector_value_domain::VectorValueDomainSet;
use super::state_writer::StateWriter;
use super::variable::Variable;
use ::collections::RangeSetStack;

pub struct Solver<'a> {
    pub puzzle: &'a Puzzle,
    pub cells: Square<Variable>,
    cage_map: Square<usize>,
    dirty_cages: RangeSetStack<u32>,
    dirty_vecs: RangeSetStack<u32>,
    dirty_vec_vals: RangeSetStack<(u32, i32)>,
    solved_cells: Vec<SquareIndex>,

    // for every vec_val_dom[i][j][k], cell k is a possible position for value j in vector i
    vec_val_doms: VectorValueDomainSet,
    //vectors: [Vec<Vector>; 2],
    unsolved_cell_count: u32,
    domain_size_sum: u32,
}

impl<'a> Solver<'a> {
    pub fn new(puzzle: &Puzzle) -> Solver {
        let size = puzzle.size;
        let cells = Square::new(Variable::unsolved_with_all(size), size as usize);
        let num_cells = cells.len();
        let unsolved_cell_count = num_cells as u32;
        let domain_size_sum = (num_cells * size) as u32;
        Solver {
            puzzle,
            cells,
            constraints: Constraints::new(),
            //cages: (0..puzzle.cages.len()).map(|i| CageMarkup::new(puzzle, i)).collect_vec(),
            //cages: vec![CageMarkup::new(); puzzle.cages.len()],
            cage_map: puzzle.cage_map(),
            dirty_cages: (0..puzzle.cages.len() as u32).collect(),
            vec_val_doms: VectorValueDomainSet::new(size),
            dirty_vecs: RangeSetStack::new(),
            dirty_vec_vals: RangeSetStack::new(),
            solved_cells: Vec::new(),
            //vectors: [vec![Vector::new(size); size], vec![Vector::new(size); size]],
            unsolved_cell_count,
            domain_size_sum,
        }
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.puzzle.size
    }

    fn cage_first_coord(&self, cage_index: usize) -> Coord {
        Coord::from_index(self.puzzle.cages[cage_index].cells[0], self.size() as usize)
    }

    pub fn solve(&mut self) {
        // clear board
        let mut board = Square::new(0, self.size() as usize);

        self.reduce_domains_by_cage();
        self.solve_main();

        for (pos, cell_markup) in self.cells.iter_coord() {
            if let Variable::Solved(val) = *cell_markup {
                board[pos] = val;
            };
        }
    }

    pub fn two_cell_cage_domains(&self, cage_index: usize) -> [&CellDomain; 2] {
        let cage = &self.puzzle.cages[cage_index];
        debug_assert_eq!(2, cage.cells.len());
        let SquareIndex(a) = cage.cells[0];
        let SquareIndex(b) = cage.cells[1];
        let (left, right) = self.cells.split_at(b);
        [
            left[a].unwrap_unsolved(),
            right[0].unwrap_unsolved(),
        ]
    }

    fn remove_from_cell_domain(&mut self, pos: SquareIndex, n: i32) -> bool {
        let size = self.puzzle.size;

        // remove n from cell domain
        match self.cells[pos] {
            Variable::Unsolved(ref mut domain) => {
                if !domain.remove(n) {
                    return false
                }
            },
            _ => return false,
        };
        self.domain_size_sum -= 1;

        debug!("removed {} from cell {} domain", n, Coord::from_index(pos, self.size() as usize));

        // update vector value domains
        for &v in &vectors_intersecting_at(pos, size) {
            self.dirty_vecs.insert(v.as_number(size) as u32);
            if let Some(dom) = self.vec_val_doms[v][n as usize - 1].as_mut() {
                let vec_pos = v.sq_pos_to_vec_pos(pos, size);
                let removed = dom.remove(vec_pos);
                debug_assert!(removed);
                self.dirty_vec_vals.insert((v.as_number(size) as u32, n));
            };
        }

        true
    }

    fn check_vector_value(&mut self, vector_id: VectorId, n: i32) {
        let vec_val_pos = match self.vec_val_doms[vector_id][n as usize - 1].as_ref().and_then(|dom| dom.single_value()) {
            Some(v) => v,
            None => return,
        };
        let sq_pos = vector_id.vec_pos_to_sq_pos(vec_val_pos as usize, self.puzzle.size);
        debug!("the only possible position for {} in {} is {}", n, vector_id, Coord::from_index(sq_pos, self.puzzle.size));
        let v2 = vector_id.intersecting_at(vec_val_pos);
        self.vec_val_doms.remove_vector_value(vector_id, n);
        self.vec_val_doms.remove_vector_value(v2, n);
        self.solve_cell(sq_pos, n);
    }

    fn remove_from_cell_domain_and_solve(&mut self, pos: SquareIndex, n: i32, mark_cage_dirty: bool) -> bool {
        let removed = self.remove_from_cell_domain(pos, n);
        if removed {
            self.solve_cell_from_domain(pos, mark_cage_dirty);
        }
        removed
    }

    fn on_change_vector(&mut self, vector_id: VectorId) {
        let size = self.size();

        // organize cells by domain size
        let mut cells_by_domain_size = vec![Vec::new(); size - 3];
        for i in 0..size {
            let sq_pos = vector_id.vec_pos_to_sq_pos(i, size);
            if let Some(domain) = self.cells[sq_pos].unsolved() {
                let len = domain.len();
                if len < size - 1 {
                    cells_by_domain_size[len - 2].push(sq_pos);
                }
            }
        }

        // find a set of cells where the collective domain size < the size of the set
        let mut cells: Vec<SquareIndex> = Vec::with_capacity(size - 1);
        for (i, cells2) in cells_by_domain_size.iter().enumerate().filter(|&(_, cells)| !cells.is_empty()) {
            cells.extend(cells2);
            let max_size = i + 2;
            'combinations: for cells in cells.iter().cloned().combinations(max_size) {
                let mut domain = CellDomain::new(size);
                for &cell in &cells {
                    for j in self.cells[cell].unsolved().unwrap() {
                        if domain.insert(j) {
                            if domain.len() > max_size {
                                continue 'combinations
                            }
                        }
                    }
                }
                debug!("values {:?} are in cells {:?}", domain.iter().collect_vec(), cells);
                let mut iter = cells.iter().cloned();
                let mut cell = iter.next();
                for i in 0..size {
                    let sq_pos = vector_id.vec_pos_to_sq_pos(i, size);
                    let in_group = cell.map_or(false, |i| sq_pos == i);
                    if in_group {
                        cell = iter.next();
                    } else {
                        for val in &domain {
                            self.remove_from_cell_domain_and_solve(sq_pos, val, true);
                        }
                    }
                }
                break
            }
        }
    }

    // mark cell solved if one candidate remains
    fn solve_cell_from_domain(&mut self, pos: SquareIndex, mark_cage_dirty: bool) -> bool {
        if self.cells[pos].is_solved() {
            return false
        }
        if let Some(value) = self.cells[pos].unwrap_unsolved().single_value() {
            self.solve_cell_complete(pos, value, mark_cage_dirty);
            true
        } else {
            let cage_index = self.cage_map[pos];
            self.on_update_cage(cage_index, mark_cage_dirty);
            false
        }
    }

    fn solve_cell(&mut self, pos: SquareIndex, value: i32) {
        if self.cells[pos].is_solved() {
            return
        }
        let to_remove = self.cells[pos].unwrap_unsolved().iter()
                .filter(|&n| n != value)
                .collect::<Vec<_>>();
        for n in to_remove {
            self.remove_from_cell_domain(pos, n);
        }
        self.solve_cell_complete(pos, value, true);
    }

    fn solve_cell_complete(&mut self, pos: SquareIndex, value: i32, mark_cage_dirty: bool) {
        if self.cells[pos].is_solved() {
            return
        }
        debug_assert!(self.cells[pos].unwrap_unsolved().single_value().is_some());

        self.unsolved_cell_count -= 1;
        let size = self.size() as usize;
        self.cells[pos] = Variable::Solved(value);
        debug!("solved cell at {}, value={}", Coord::from_index(pos, size), value);
        self.solved_cells.push(pos);

        let cage_index = self.cage_map[pos];
        self.on_update_cage(cage_index, mark_cage_dirty);
    }

    fn on_solve_cell(&mut self, pos: SquareIndex) {
        let value = self.cells[pos].unwrap_solved();
        let size = self.size() as usize;
        for &vector_id in vectors_intersecting_at(pos, size).iter() {
            for pos in (0..size).map(|n| vector_id.vec_pos_to_sq_pos(n, size)) {
                self.remove_from_cell_domain_and_solve(pos, value, true);
            }
        }
    }

    fn on_update_cage(&mut self, cage_index: usize, mark_cage_dirty: bool) {
        if self.is_cage_solved(&self.puzzle.cages[cage_index]) {
            self.dirty_cages.remove(&(cage_index as u32));
        } else if mark_cage_dirty && self.dirty_cages.insert(cage_index as u32) {
            debug!("marked cage at {} dirty", self.cage_first_coord(cage_index));
        }
    }

    pub fn solved(&self) -> bool {
        self.unsolved_cell_count == 0
    }

    fn is_cage_solved(&self, cage: &Cage) -> bool {
        cage.cells.iter().all(|&p| self.cells[p].is_solved())
    }

    fn solve_main(&mut self) {
        debug!("solving cages");
        let size = self.size();
        let mut count = 0;
        let mut state_writer_last = self.domain_size_sum;
        let mut state_writer = StateWriter::new();
        while !self.solved() {
            count += 1;
            if let Some((v, value)) = self.dirty_vec_vals.pop() {
                let v = VectorId::from_number(size, v as usize);
                self.check_vector_value(v, value);
            } else if let Some(pos) = self.solved_cells.pop() {
                self.on_solve_cell(pos);
            } else if let Some(cage_index) = self.dirty_cages.pop() {
                self.solve_cage(cage_index);
            } else if let Some(v) = self.dirty_vecs.pop() {
                let v = VectorId::from_number(size, v as usize);
                self.on_change_vector(v);
            } else {
                break
            }
            if self.domain_size_sum < state_writer_last {
                state_writer_last = self.domain_size_sum;
                state_writer.write(self);
            }
        }
        println!("Iterations: {}", count);
    }

    fn solve_cage(&mut self, cage_index: u32) {
        let cage_index = cage_index as usize;
        debug!("solving cage at {}", self.cage_first_coord(cage_index));

        let cage = &self.puzzle.cages[cage_index];
        let mut unsolved = cage.cells.iter()
            .cloned()
            .filter(|&pos| self.cells[pos].is_unsolved())
            .collect_vec();
    }

}
