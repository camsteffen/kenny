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

pub struct Solver<'a> {
    pub puzzle: &'a Puzzle,
    pub cells: Square<Variable>,
    cages: Vec<CageMarkup>,
    cage_map: Square<usize>,
    dirty_cages: HashSet<usize>,

    // for every vec_val_dom[i][j][k], cell k is a possible position for value j in vector i
    vec_val_doms: VectorValueDomainSet,
    //vectors: [Vec<Vector>; 2],
}

impl<'a> Solver<'a> {
    pub fn new(puzzle: &Puzzle) -> Solver {
        let dirty_cages = (0..puzzle.cages.len()).collect();
        let size = puzzle.size;
        Solver {
            puzzle: puzzle,
            cells: Square::new(Variable::unsolved_with_all(size), size as usize),
            //cages: (0..puzzle.cages.len()).map(|i| CageMarkup::new(puzzle, i)).collect_vec(),
            cages: vec![CageMarkup::new(); puzzle.cages.len()],
            cage_map: puzzle.cage_map(),
            dirty_cages: dirty_cages,
            vec_val_doms: VectorValueDomainSet::new(size),
            //vectors: [vec![Vector::new(size); size], vec![Vector::new(size); size]],
        }
    }

    #[inline]
    fn size(&self) -> usize {
        self.puzzle.size
    }

    fn cage_first_coord(&self, cage_index: usize) -> Coord {
        Coord::from_index(self.puzzle.cages[cage_index].cells[0], self.size() as usize)
    }

    /// retrives the sum of the number of values in the domain of every cell
    fn total_domain(&self) -> u32 {
        self.cells.iter()
            .map(|v| match *v {
                Variable::Solved(_) => 1,
                Variable::Unsolved(ref domain) => domain.size() as u32,
            })
            .sum()
    }

    pub fn solve(&mut self) {
        // clear board
        let mut board = Square::new(0, self.size() as usize);

        self.solve_single_cell_cages();
        self.reduce_domains_by_cage();
        self.solve_cages();

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
        for &v in &vectors_intersecting_at(pos, self.size()) {
            let vec_pos = self.vec_val_doms[v][n as usize - 1].as_mut().and_then(|dom| {
                let vec_pos = v.sq_pos_to_vec_pos(pos, size);
                if dom.remove(vec_pos) {
                    dom.single_value()
                } else {
                    None
                }
            });
            if let Some(vec_pos) = vec_pos {
                let sq_pos = v.vec_pos_to_sq_pos(vec_pos as usize, self.puzzle.size);
                debug!("the only possible position for {} in {} is {}", n, v, Coord::from_index(pos, self.puzzle.size));
                let v2 = v.intersecting_at(vec_pos);
                self.vec_val_doms.remove_vector_value(v, n);
                self.vec_val_doms.remove_vector_value(v2, n);
                self.solve_cell(sq_pos, n);
            }
        }

        true
    }

    fn remove_and_solve(&mut self, pos: usize, n: i32, mark_cage_dirty: bool) -> bool {
        let removed = self.remove_from_cell_domain(pos, n);
        if removed {
            self.solve_cell_from_domain(pos, mark_cage_dirty);
        }
        removed
    }

    // mark cell solved if one candidate remains
    fn solve_cell_from_domain(&mut self, pos: usize, mark_cage_dirty: bool) -> bool {
        if self.cells[pos].is_solved() {
            return false
        }
        if let Some(value) = self.cells[pos].unwrap_unsolved().single_value() {
            /*
            for &v in &vectors_intersecting_at(pos, self.size()) {
                self.vec_val_doms.remove_vector_value(v, value);
            }
            */
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
        let to_remove = self.cells[pos].unwrap_unsolved().iter().filter(|&n| n != value).collect::<Vec<_>>();
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

        let size = self.size() as usize;
        self.cells[pos] = Variable::Solved(value);
        /*
        for &v in &vectors_intersecting_at(pos, size) {
            self.vec_val_doms.remove_vector_value(v, value);
        }
        */
        debug!("solved cell at {}, value={}", Coord::from_index(pos, size), value);

        // remove possibility from cells in same row or column
        for &vector_id in vectors_intersecting_at(pos, size).iter() {
            for pos in (0..size).map(|n| vector_id.vec_pos_to_sq_pos(n, size)) {
                self.remove_and_solve(pos, value, true);
            }
        }

        let cage_index = self.cage_map[pos];
        self.on_update_cage(cage_index, mark_cage_dirty);
    }

    fn on_update_cage(&mut self, cage_index: usize, mark_cage_dirty: bool) {
        if self.is_cage_solved(&self.puzzle.cages[cage_index]) {
            self.dirty_cages.remove(&cage_index);
        } else if mark_cage_dirty && self.dirty_cages.insert(cage_index) {
            debug!("marked cage at {} dirty", self.cage_first_coord(cage_index));
        }
    }

    pub fn solved(&self) -> bool {
        self.cells.iter().all(|v| v.is_solved())
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
                            self.remove_and_solve(pos, n, false);
                        }
                    }
                },
                Operator::Multiply => {
                    let non_factors = (2..=self.size() as i32)
                        .filter(|n| !cage.target.is_multiple_of(n))
                        .collect_vec();
                    for &pos in &cage.cells {
                        for &n in &non_factors {
                            self.remove_and_solve(pos, n, false);
                        }
                    }
                },
                Operator::Subtract => {
                    let size = self.size() as i32;
                    if cage.target > size / 2 {
                        for &pos in &cage.cells {
                            for n in size - cage.target + 1..=cage.target {
                                self.remove_and_solve(pos, n, false);
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
                    if non_domain.size() > 0 {
                        for &pos in &cage.cells {
                            for n in non_domain.iter() {
                                self.remove_and_solve(pos, n, false);
                            }
                        }
                    }
                },
            }
        }
    }

    fn solve_cages(&mut self) {
        debug!("solving cages");
        let mut state_writer = StateWriter::new();

        let mut total_domain = 0;
        //let mut to_remove = Vec::new();
        while !self.dirty_cages.is_empty() {

            let next_total_domain = self.total_domain();
            if next_total_domain != total_domain {
                total_domain = next_total_domain;
                state_writer.write(&self);
            }

            let mut best_cage = None;
            for &cage_index in &self.dirty_cages {
                let cage_rank: i32 = self.puzzle.cages[cage_index].cells.iter()
                    .filter_map(|&cell_index| self.cells[cell_index].unsolved())
                    .map(|domain| domain.size() as i32)
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
            //to_remove.push(best_cage_index);
            //for index in to_remove.drain(..) {
                if !self.dirty_cages.remove(&best_cage_index) {
                    panic!("expected {} in dirty cages", best_cage_index)
                }
            //}
            self.solve_cage(best_cage_index);
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

    fn solve_cage(&mut self, cage_index: usize) {
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
                self.remove_and_solve(index, n, false);
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
                self.remove_and_solve(pos, n, true);
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
