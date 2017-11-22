extern crate num;

use range_domain::CellDomain;
use itertools::Itertools;
use num::Integer;
use puzzle::Operator;
use puzzle::Puzzle;
use square::Coord;
use square::Square;
use square::vector::VectorId;
use square::vector::vectors_intersecting_at;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashSet;
use puzzle::Cage;
use super::vector_value_domain::VectorValueDomainSet;
use super::cage_markup::CageMarkup;
use super::state_writer::StateWriter;
use super::variable::Variable;
use ::collections::RangeSetStack;

pub struct Solver<'a> {
    pub puzzle: &'a Puzzle,
    pub cells: Square<Variable>,
    cages: Vec<CageMarkup>,
    cage_map: Square<usize>,
    dirty_cages: RangeSetStack<u32>,
    dirty_vecs: RangeSetStack<u32>,
    dirty_vec_vals: RangeSetStack<(u32, i32)>,
    solved_cells: Vec<usize>,

    // for every vec_val_dom[i][j][k], cell k is a possible position for value j in vector i
    vec_val_doms: VectorValueDomainSet,
    //vectors: [Vec<Vector>; 2],
    unsolved_cell_count: u32,
}

impl<'a> Solver<'a> {
    pub fn new(puzzle: &Puzzle) -> Solver {
        let size = puzzle.size;
        let cells = Square::new(Variable::unsolved_with_all(size), size as usize);
        let unsolved_cell_count = cells.len() as u32;
        Solver {
            puzzle,
            cells,
            //cages: (0..puzzle.cages.len()).map(|i| CageMarkup::new(puzzle, i)).collect_vec(),
            cages: vec![CageMarkup::new(); puzzle.cages.len()],
            cage_map: puzzle.cage_map(),
            dirty_cages: (0..puzzle.cages.len() as u32).collect(),
            vec_val_doms: VectorValueDomainSet::new(size),
            dirty_vecs: RangeSetStack::new(),
            dirty_vec_vals: RangeSetStack::new(),
            solved_cells: Vec::new(),
            //vectors: [vec![Vector::new(size); size], vec![Vector::new(size); size]],
            unsolved_cell_count,
        }
    }

    #[inline]
    fn size(&self) -> usize {
        self.puzzle.size
    }

    fn cage_first_coord(&self, cage_index: usize) -> Coord {
        Coord::from_index(self.puzzle.cages[cage_index].cells[0], self.size() as usize)
    }

    pub fn solve(&mut self) {
        // clear board
        let mut board = Square::new(0, self.size() as usize);

        self.solve_single_cell_cages();
        self.reduce_domains_by_cage();
        self.solve_main();

        for (pos, cell_markup) in self.cells.iter_coord() {
            if let Variable::Solved(val) = *cell_markup {
                board[pos] = val;
            };
        }
    }

    fn two_cell_cage_domains(&self, cage_index: usize) -> [&CellDomain; 2] {
        let cage = &self.puzzle.cages[cage_index];
        debug_assert_eq!(2, cage.cells.len());
        let index = cage.cells[1];
        let (left, right) = self.cells.split_at(index);
        [
            left[cage.cells[0]].unwrap_unsolved(),
            right[0].unwrap_unsolved(),
        ]
    }

fn remove_from_cell_domain(&mut self, pos: usize, n: i32) -> bool {
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

    fn remove_from_cell_domain_and_solve(&mut self, pos: usize, n: i32, mark_cage_dirty: bool) -> bool {
        let removed = self.remove_from_cell_domain(pos, n);
        if removed {
            self.solve_cell_from_domain(pos, mark_cage_dirty);
        }
        removed
    }

    fn on_change_vector(&mut self, vector_id: VectorId) {
        let size = self.size();
        let mut cells = (0..size).filter_map(|i| {
            let i = vector_id.vec_pos_to_sq_pos(i, size);
            self.cells[i].unsolved().and_then(|domain| {
                let len = domain.len();
                if len < size {
                    Some((i, domain.len()))
                } else {
                    None
                }
            })
        }).collect_vec();
        cells.sort_unstable_by(|&(_, a), &(_, b)| a.cmp(&b));
        for (i, domain_size) in cells {
            
        }
    }

    // mark cell solved if one candidate remains
    fn solve_cell_from_domain(&mut self, pos: usize, mark_cage_dirty: bool) -> bool {
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

    fn solve_cell(&mut self, pos: usize, value: i32) {
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

    fn solve_cell_complete(&mut self, pos: usize, value: i32, mark_cage_dirty: bool) {
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

    fn on_solve_cell(&mut self, pos: usize) {
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

    fn solve_single_cell_cages(&mut self) {
        debug!("solving single cell cages");

        for cage in self.puzzle.cages.iter().filter(|cage| cage.cells.len() == 1) {
            let index = cage.cells[0];
            self.solve_cell(index, cage.target);
        }
    }

    fn reduce_domains_by_cage(&mut self) {
        debug!("reducing cell domains by cage-specific info");

        for cage in &self.puzzle.cages {
            match cage.operator {
                Operator::Add => {
                    for &pos in &cage.cells {
                        for n in cage.target - cage.cells.len() as i32 + 2..=self.size() as i32 {
                            self.remove_from_cell_domain_and_solve(pos, n, false);
                        }
                    }
                },
                Operator::Multiply => {
                    let non_factors = (2..=self.size() as i32)
                        .filter(|n| !cage.target.is_multiple_of(n))
                        .collect_vec();
                    for &pos in &cage.cells {
                        for &n in &non_factors {
                            self.remove_from_cell_domain_and_solve(pos, n, false);
                        }
                    }
                },
                Operator::Subtract => {
                    let size = self.size() as i32;
                    if cage.target > size / 2 {
                        for &pos in &cage.cells {
                            for n in size - cage.target + 1..=cage.target {
                                self.remove_from_cell_domain_and_solve(pos, n, false);
                            }
                        }
                    }
                },
                Operator::Divide => {
                    let mut non_domain = CellDomain::with_all(self.size());
                    for n in 1..=self.size() as i32 / cage.target {
                        non_domain.remove(n);
                        non_domain.remove(n * cage.target);
                    }
                    if non_domain.len() > 0 {
                        for &pos in &cage.cells {
                            for n in non_domain.iter() {
                                self.remove_from_cell_domain_and_solve(pos, n, false);
                            }
                        }
                    }
                },
            }
        }
    }

    fn solve_main(&mut self) {
        debug!("solving cages");
        let size = self.size();
        let mut count = 0;
        while !self.solved() {
            count += 1;
            if let Some((v, value)) = self.dirty_vec_vals.pop() {
                let v = VectorId::from_number(size, v as usize);
                self.check_vector_value(v, value);
            } else if let Some(v) = self.dirty_vecs.pop() {
                let v = VectorId::from_number(size, v as usize);
                self.on_change_vector(v);
            } else if let Some(pos) = self.solved_cells.pop() {
                self.on_solve_cell(pos);
            } else if let Some(cage_index) = self.dirty_cages.pop() {
                self.solve_cage(cage_index);
            } else {
                break
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

        // find all solutions for the cage
        let solutions = match self.puzzle.cages[cage_index].operator {
            Operator::Add => self.cage_solutions_add(cage_index, &unsolved),
            Operator::Multiply => self.cage_solutions_multiply(cage_index, &unsolved),
            Operator::Subtract => self.cage_solutions_subtract(cage_index, &unsolved),
            Operator::Divide => self.cage_solutions_divide(cage_index, &unsolved),
        };

        debug!("solutions: {:?}", solutions);

        // assemble domain for each unsolved cell from cell solutions
        let mut soln_domain = vec![CellDomain::with_none(self.size()); unsolved.len()];
        for solution in &solutions {
            for i in 0..unsolved.len() {
                soln_domain[i].insert(solution[i]);
            }
        }

        // remove values from cell domains that are not in a cage solution
        let mut to_remove = Vec::new();
        for (i, &index) in unsolved.iter().enumerate() {
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
                    .filter(|&n| !soln_domain[i].contains(n))
                    .collect_vec();
            }
            for n in no_solutions {
                self.remove_from_cell_domain_and_solve(index, n, false);
            }
        }

        // remove solved cells from unsolved
        if !to_remove.is_empty() {
            let mut to_remove = to_remove.iter().peekable();
            unsolved = unsolved.into_iter()
                .enumerate()
                .filter(|&(ref i, _)| {
                    let remove = to_remove.peek().map_or(false, |&r| r == i);
                    if remove {
                        to_remove.next();
                    }
                    !remove
                })
                .map(|(_, i)| i)
                .collect()
        }

        let mut vectors = BTreeMap::new();
        for ((i1, &p1), (i2, &p2)) in unsolved.iter().enumerate().tuple_combinations() {
            if let Some(vector_id) = shared_vector(p1, p2, self.size()) {
                let vector_positions = vectors.entry(vector_id).or_insert_with(BTreeSet::new);
                vector_positions.insert(i1);
                vector_positions.insert(i2);
            }
        }
        let vectors = vectors.into_iter()
            .map(|(vector_id, unsolved_indices)| {
                (vector_id, unsolved_indices.into_iter().collect_vec())
            });

        for (vector_id, unsolved_indices) in vectors {
            self.find_vector_values(cage_index, vector_id, &unsolved_indices, &solutions);
        }
    }

    fn find_vector_values(&mut self,
                          cage_index: usize,
                          vector_id: VectorId,
                          unsolved_indices: &[usize],
                          solutions: &[Vec<i32>])
    {
        let mut values: HashSet<i32>;
        {
            let cage = &mut self.cages[cage_index];
            let mut solutions_iter = solutions.iter();
            let solution = solutions_iter.next().unwrap();
            values = unsolved_indices.iter()
                .map(|&unsolved_id| solution[unsolved_id])
                .filter(|n| {
                    cage.vector_vals.get(&vector_id)
                        .map_or(true, |vals| !vals.contains(n))
                })
                .collect();
            for solution in solutions_iter {
                if values.is_empty() { break }
                values = unsolved_indices.iter()
                    .map(|&unsolved_id| solution[unsolved_id])
                    .filter(|n| values.contains(n))
                    .collect();
            }

            cage.vector_vals.entry(vector_id)
                .or_insert_with(HashSet::new)
                .extend(&values);
        }

        let remove_from = vector_id.iter_sq_pos(self.size())
            .filter(|&pos| self.cage_map[pos] != cage_index)
            .collect_vec();
        for n in values {
            debug!("value {} exists in cage at {}, in {}", n, self.cage_first_coord(cage_index), vector_id);

            for &pos in &remove_from {
                self.remove_from_cell_domain_and_solve(pos, n, true);
            }
        }
    }

    fn cage_solutions_add(&self, cage_index: usize, unsolved: &[usize]) -> Vec<Vec<i32>> {
        let cage = &self.puzzle.cages[cage_index];
        let solved_sum: i32 = cage.cells.iter()
            .filter_map(|&i| self.cells[i].solved())
            .sum();
        let remain_sum = cage.target - solved_sum;
        let mut solutions = Vec::new();
        let mut solution = vec![0; unsolved.len()];
        self.cage_solutions_add_next(0, unsolved, remain_sum, &mut solution, &mut solutions);
        solutions
    }

    fn cage_solutions_multiply(&self, cage_index: usize, unsolved: &[usize]) -> Vec<Vec<i32>> {
        let cage = &self.puzzle.cages[cage_index];
        let solved_product: i32 = cage.cells.iter()
            .filter_map(|&i| self.cells[i].solved())
            .product();
        let remain_product = cage.target / solved_product;
        let mut solutions = Vec::new();
        let mut solution = vec![0; unsolved.len()];
        self.cage_solutions_product_next(0, unsolved, remain_product, &mut solution, &mut solutions);
        solutions
    }

    fn cage_solutions_subtract(&self, cage_index: usize, unsolved: &[usize]) -> Vec<Vec<i32>> {
        let cage = &self.puzzle.cages[cage_index];
        let mut solutions = Vec::new();
        if unsolved.len() == 1 {
            let known_val;
            let domain;
            if cage.cells[0] == unsolved[0] {
                domain = self.cells[cage.cells[0]].unwrap_unsolved();
                known_val = self.cells[cage.cells[1]].unwrap_solved();
            } else {
                known_val = self.cells[cage.cells[0]].unwrap_solved();
                domain = self.cells[cage.cells[1]].unwrap_unsolved();
            }
            let n = known_val - cage.target;
            if n > 0 && domain.contains(n) {
                solutions.push(vec![n; 1]);
            }
            let n = known_val + cage.target;
            if n <= self.size() as i32 && domain.contains(n) {
                solutions.push(vec![n; 1]);
            }
        } else {
            let domains = self.two_cell_cage_domains(cage_index);
            for n in domains[0].iter() {
                let m = n - cage.target;
                if m > 0 && domains[1].contains(m) {
                    solutions.push(vec![n, m]);
                }
                let m = n + cage.target;
                if m <= self.size() as i32 && domains[1].contains(m) {
                    solutions.push(vec![n, m]);
                }
            }
        }
        solutions
    }

    fn cage_solutions_divide(&self, cage_index: usize, unsolved: &[usize]) -> Vec<Vec<i32>> {
        let size = self.size() as i32;
        let cage = &self.puzzle.cages[cage_index];
        let mut solutions = Vec::new();
        if unsolved.len() == 1 {
            let known_val;
            let domain;
            if cage.cells[0] == unsolved[0] {
                domain = self.cells[cage.cells[0]].unwrap_unsolved();
                known_val = self.cells[cage.cells[1]].unwrap_solved();
            } else {
                known_val = self.cells[cage.cells[0]].unwrap_solved();
                domain = self.cells[cage.cells[1]].unwrap_unsolved();
            }
            let n = known_val / cage.target;
            if n > 0 && domain.contains(n) {
                solutions.push(vec![n; 1]);
            }
            let n = known_val * cage.target;
            if n <= size && domain.contains(n) {
                solutions.push(vec![n; 1]);
            }
        } else {
            let cells = &self.puzzle.cages[cage_index].cells;
            debug_assert_eq!(2, cells.len());
            let domains = self.two_cell_cage_domains(cage_index);
            for n in domains[0].iter() {
                let m = n / cage.target;
                if m > 0 && n % cage.target == 0 && domains[1].contains(m) {
                    solutions.push(vec![n, m]);
                }
                let m = n * cage.target;
                if m <= size && domains[1].contains(m) {
                    solutions.push(vec![n, m]);
                }
            }
        }
        solutions
    }

    fn cage_solutions_add_next(
        &self,
        i: usize,
        pos: &[usize],
        remain_sum: i32,
        solution: &mut [i32],
        solutions: &mut Vec<Vec<i32>>)
    {
        let size = self.size();
        let collides = |n: i32, vals: &[i32]| {
            (0..i).filter(|&j| vals[j] == n)
                .any(|j| shared_vector(pos[j], pos[i], size as usize).is_some())
        };
        if remain_sum <= 0 { return }
        if i == solution.len() - 1 {
            if remain_sum > self.cells.width() as i32 { return }
            if !self.cells[pos[i]].unwrap_unsolved().contains(remain_sum) { return }
            if collides(remain_sum, &solution[..i]) { return }
            solution[i] = remain_sum;
            solutions.push(solution.to_vec());
        } else {
            for n in self.cells[pos[i]].unwrap_unsolved().iter() {
                if n >= remain_sum { break }
                if collides(n, &solution[..i]) { continue }
                solution[i] = n;
                self.cage_solutions_add_next(i + 1, pos, remain_sum - n, solution, solutions);
            }
        }
    }

    fn cage_solutions_product_next(
        &self,
        i: usize,
        pos: &[usize],
        remain_product: i32,
        solution: &mut [i32],
        solutions: &mut Vec<Vec<i32>>)
    {
        let size = self.size();
        let collides = |n: i32, vals: &[i32]| {
            (0..i).filter(|&j| vals[j] == n)
                .any(|j| shared_vector(pos[j], pos[i], size as usize).is_some())
        };
        if remain_product <= 0 { return }
        if i == solution.len() - 1 {
            if remain_product > self.cells.width() as i32 { return }
            if !self.cells[pos[i]].unwrap_unsolved().contains(remain_product) { return }
            if collides(remain_product, &solution[..i]) { return }
            solution[i] = remain_product;
            solutions.push(solution.to_vec());
        } else {
            for n in self.cells[pos[i]].unwrap_unsolved().iter() {
                if n > remain_product { break }
                if remain_product % n != 0 { continue }
                if collides(n, &solution[..i]) { continue }
                solution[i] = n;
                self.cage_solutions_product_next(i + 1, pos, remain_product / n, solution, solutions);
            }
        }
    }
}

fn shared_vector(pos1: usize, pos2: usize, size: usize) -> Option<VectorId> {
    if pos1 / size == pos2 / size {
        Some(VectorId::Row(pos1 / size))
    } else if pos1 % size == pos2 % size {
        Some(VectorId::Col(pos1 % size))
    } else {
        None
    }
}
