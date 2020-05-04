use std::mem;
use std::ops::Deref;
use std::ops::DerefMut;

use ahash::{AHashMap, AHashSet};
use vec_map::VecMap;

use crate::collections::square::IsSquare;
use crate::collections::Square;
use crate::puzzle::{CageRef, CellId, CellRef, Value};
use crate::puzzle::Operator;
use crate::puzzle::Puzzle;
use crate::puzzle::solve::ValueSet;
use crate::puzzle::solve::CellVariable::{Solved, Unsolved};

use super::CellVariable;
use super::markup::PuzzleMarkupChanges;

#[derive(Clone)]
pub struct CageSolutionsSet {
    data: Vec<CageSolutions>,
}

impl CageSolutionsSet {
    pub fn new(puzzle: &Puzzle) -> Self {
        Self {
            data: puzzle.cages().map(|cage| {
                let ids = cage.cells().map(CellRef::id).collect();
                CageSolutions::new(ids)
            }).collect(),
        }
    }

    pub fn init(&mut self, puzzle: &Puzzle, cell_variables: &Square<CellVariable>) {
        let mut cages_iter = puzzle.cages();
        for cage_solutions in &mut self.data {
            let cage = cages_iter.next().unwrap();
            if cage.operator() != Operator::Nop {
                let cell_variables = cage.cells().map(|cell| &cell_variables[cell.id()]).collect::<Vec<_>>();
                cage_solutions.init(puzzle, cage, &cell_variables);
            }
        }
    }

    pub fn apply_changes(&mut self, puzzle: &Puzzle, changes: &PuzzleMarkupChanges) {
        struct CageData {
            remove_indices: Vec<usize>,
            solved_cells: Vec<(CellId, Value)>,
            removed_values: Vec<(CellId, Value)>,
        }
        impl CageData {
            fn new() -> Self {
                Self {
                    remove_indices: Vec::new(),
                    solved_cells: Vec::new(),
                    removed_values: Vec::new(),
                }
            }
        }

        let mut data = VecMap::default();

        for (&cage_id, values) in &changes.cage_solution_removals {
            data.entry(cage_id).or_insert_with(CageData::new).remove_indices.extend(values);
        }

        for &(id, value) in &changes.cell_solutions {
            let cage_id = puzzle.cell(id).cage_id();
            data.entry(cage_id).or_insert_with(CageData::new).solved_cells.push((id, value));
        }

        let solved_cells = changes.cell_solutions.iter().map(|&(i, _)| i).collect::<AHashSet<_>>();

        for (&id, values) in &changes.cell_domain_value_removals {
            let cage_id = puzzle.cell(id).cage_id();
            if solved_cells.contains(&id) { continue }
            let cage_data = data.entry(cage_id).or_insert_with(CageData::new);
            for &value in values {
                cage_data.removed_values.push((id, value));
            }
        }

        for (cage_id, cage_data) in data {
            self[cage_id].apply_changes(&cage_data.remove_indices, &cage_data.solved_cells, &cage_data.removed_values);
        }
    }
}

impl Deref for CageSolutionsSet {
    type Target = Vec<CageSolutions>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for CageSolutionsSet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

/// A set of possible solutions for one cage
#[derive(Clone)]
pub struct CageSolutions {
    /// the indices of the unsolved cells in the cage (not every cell in the cage)
    pub cell_ids: Vec<CellId>,
    /// a list of all possible solutions for a cage. the numbers correspond to the cells represented in cell_ids
    pub solutions: Vec<Vec<i32>>,
    /// Cell index -> Cage unsolved cell index
    index_map: AHashMap<CellId, usize>,
}

impl CageSolutions {

    pub fn new(indices: Vec<CellId>) -> Self {
        let index_map = Self::build_index_map(&indices);
        Self {
            cell_ids: indices,
            solutions: Vec::new(),
            index_map,
        }
    }

    pub fn init(&mut self, puzzle: &Puzzle, cage: CageRef, cell_variables: &[&CellVariable]) {
        self.solutions = match cage.operator() {
            Operator::Add => Self::init_add(puzzle, cage, cell_variables),
            Operator::Multiply => Self::init_multiply(puzzle, cage, cell_variables),
            Operator::Subtract => Self::init_subtract(puzzle, cage, cell_variables),
            Operator::Divide => Self::init_divide(puzzle, cage, cell_variables),
            Operator::Nop => panic!("cannot init CageSolutions with a Nop cage"),
        };

        debug!("cage at {:?} solutions: {:?}", cage.coord(), self.solutions);
    }

    pub fn num_cells(&self) -> usize {
        self.cell_ids.len()
    }

    pub fn apply_changes(&mut self, remove_indices: &[usize], solved_cells: &[(CellId, i32)], removed_values: &[(CellId, i32)]) {
        let remove_indices = remove_indices.iter().copied().collect::<AHashSet<_>>();

        let solved_cells = solved_cells.iter().map(|&(i, v)| (self.index_map[&i], v)).collect::<Vec<_>>();
        let remove_cells = solved_cells.iter().map(|&(i, _)| i).collect::<AHashSet<_>>();

        let len = self.cell_ids.len() - solved_cells.len();
        let cell_ids = mem::replace(&mut self.cell_ids, Vec::with_capacity(len));
        let mut keep_indices = Vec::with_capacity(len);
        let cell_ids = cell_ids.into_iter().enumerate().filter(|&(i, _)| !remove_cells.contains(&i));
        for (i, sq_idx) in cell_ids {
            self.cell_ids.push(sq_idx);
            keep_indices.push(i);
        }
        let removed_values = removed_values.iter().map(|&(i, v)| (self.index_map[&i], v)).collect::<Vec<_>>();
        let solutions = mem::replace(&mut self.solutions, Vec::new());
        self.solutions = solutions.into_iter().enumerate()
            .filter_map(|(i, solution)| {
                if remove_indices.contains(&i) { return None }
                for &(index, value) in &solved_cells {
                    if solution[index] != value { return None }
                }
                for &(index, value) in &removed_values {
                    if solution[index] == value { return None }
                }
                Some(keep_indices.iter().map(|&i| solution[i]).collect::<Vec<_>>())
            })
            .collect();
        self.reset_index_map();
    }

    pub fn remove_indices(&mut self, indices: &[CellId]) {
        let capacity = self.cell_ids.len() - indices.len();
        if capacity == 0 {
            self.cell_ids.clear();
            self.solutions.clear();
            self.index_map.clear();
            return
        }
        let indices_set = indices.iter().copied().collect::<AHashSet<_>>();
        let solution_indices_set = indices.iter()
            .map(|&i| self.cell_ids.iter().position(|&j| j == i).unwrap())
            .collect::<AHashSet<_>>();
        self.cell_ids.retain(|i| !indices_set.contains(i));
        for solution in &mut self.solutions {
            *solution = solution.iter_mut().enumerate()
                .filter_map(|(i, &mut j)| {
                    if solution_indices_set.contains(&i) { None } else { Some(j) }
                })
                .collect();
        }
        self.reset_index_map();
    }

    pub fn remove_cell_domain_value(&mut self, index: CellId, value: i32) {
        let index = self.index_map[&index];
        self.solutions.retain(|solution| solution[index] != value);
    }

    fn build_index_map(indices: &[CellId]) -> AHashMap<CellId, usize> {
        indices.iter().copied().enumerate().map(|(i, sq_i)| (sq_i, i)).collect()
    }

    fn reset_index_map(&mut self) {
        self.index_map = Self::build_index_map(&self.cell_ids);
    }

    fn init_add(puzzle: &Puzzle, cage: CageRef, cell_variables: &[&CellVariable]) -> Vec<Vec<i32>> {
        let mut indices = Vec::new();
        let mut cell_domains = Vec::new();
        let mut solved_sum = 0_i32;
        for (i, &cell_variable) in cell_variables.iter().enumerate() {
            match cell_variable {
                &Solved(v) => solved_sum += v,
                Unsolved(domain) => {
                    indices.push(cage.cell(i).id());
                    cell_domains.push(domain);
                },
            }
        }
        let remain_sum = cage.target() - solved_sum;
        let mut solution = vec![0; indices.len()];
        let mut solutions = Vec::new();
        Self::init_add_next(0, puzzle, remain_sum, &indices, &cell_domains, &mut solution, &mut solutions);
        solutions
    }

    fn init_multiply(puzzle: &Puzzle, cage: CageRef, cell_variables: &[&CellVariable]) -> Vec<Vec<i32>> {
        let mut indices = Vec::new();
        let mut cell_domains = Vec::new();
        let mut solved_product = 1;
        for (i, &cell_variable) in cell_variables.iter().enumerate() {
            match cell_variable {
                &Solved(v) => solved_product *= v,
                Unsolved(domain) => {
                    indices.push(cage.cell(i).id());
                    cell_domains.push(domain);
                },
            }
        }
        let remain_product = cage.target() / solved_product;
        let mut solution = vec![0; indices.len()];
        let mut solutions = Vec::new();
        Self::init_multiply_next(0, puzzle, remain_product, &indices, &cell_domains, &mut solution, &mut solutions);
        solutions
    }

    fn init_subtract(puzzle: &Puzzle, cage: CageRef, cell_variables: &[&CellVariable]) -> Vec<Vec<i32>> {
        debug_assert_eq!(2, cage.cell_count());
        let mut solutions = Vec::new();
        if let Some(solved_pos) = cell_variables.iter().position(|v| v.is_solved()) {
            let known_val = cell_variables[solved_pos].unwrap_solved();
            let domain = cell_variables[(solved_pos + 1) % 2].unwrap_unsolved();
            let n = known_val - cage.target();
            if n > 0 && domain.contains(n) {
                solutions.push(vec![n; 1]);
            }
            let m = known_val + cage.target();
            if m <= puzzle.width() as i32 && domain.contains(m) {
                solutions.push(vec![m; 1]);
            }
        } else {
            let domains = cell_variables.iter().map(|variable| variable.unwrap_unsolved()).collect::<Vec<_>>();
            for n in domains[0] {
                let m = n - cage.target();
                if m > 0 && domains[1].contains(m) {
                    solutions.push(vec![n, m]);
                }
                let m = n + cage.target();
                if m <= puzzle.width() as i32 && domains[1].contains(m) {
                    solutions.push(vec![n, m]);
                }
            }
        }
        solutions
    }

    fn init_divide(puzzle: &Puzzle, cage: CageRef, cell_variables: &[&CellVariable]) -> Vec<Vec<i32>> {
        debug_assert_eq!(2, cage.cell_count());
        let mut solutions = Vec::new();
        if let Some(solved_pos) = cell_variables.iter().position(|v| v.is_solved()) {
            let known_val = cell_variables[solved_pos].unwrap_solved();
            let domain = cell_variables[(solved_pos + 1) % 2].unwrap_unsolved();
            let n = known_val / cage.target();
            if n > 0 && domain.contains(n) {
                solutions.push(vec![n; 1]);
            }
            let m = known_val * cage.target();
            if m <= puzzle.width() as i32 && domain.contains(m) {
                solutions.push(vec![m; 1]);
            }
        } else {
            let domains = cell_variables.iter().map(|variable| variable.unwrap_unsolved()).collect::<Vec<_>>();
            for n in domains[0] {
                let m = n / cage.target();
                if m > 0 && n % cage.target() == 0 && domains[1].contains(m) {
                    solutions.push(vec![n, m]);
                }
                let m = n * cage.target();
                if m <= puzzle.width() as i32 && domains[1].contains(m) {
                    solutions.push(vec![n, m]);
                }
            }
        }
        solutions
    }

    fn init_add_next(
        i: usize,
        puzzle: &Puzzle,
        remain_sum: i32,
        cell_ids: &[CellId],
        cell_domains: &[&ValueSet],
        solution: &mut [i32],
        solutions: &mut Vec<Vec<i32>>)
    {
        let collides = |n: i32, vals: &[i32]| {
            (0..i)
                .filter(|&j| vals[j] == n)
                .any(|j| puzzle.shared_vector(cell_ids[i], cell_ids[j]).is_some())
        };
        if remain_sum <= 0 { return }
        if solution.is_empty() {
            todo!();
        }
        if i == solution.len() - 1 {
            if remain_sum > puzzle.width() as i32 { return }
            if !cell_domains[i].contains(remain_sum) { return }
            if collides(remain_sum, &solution[..i]) { return }
            solution[i] = remain_sum;
            solutions.push(solution.to_vec());
        } else {
            for n in cell_domains[i] {
                if n >= remain_sum { break }
                if collides(n, &solution[..i]) { continue }
                solution[i] = n;
                Self::init_add_next(i + 1, puzzle, remain_sum - n, cell_ids, cell_domains, solution, solutions);
            }
        }
    }

    fn init_multiply_next(
        i: usize,
        puzzle: &Puzzle,
        remain_product: i32,
        cell_ids: &[CellId],
        cell_domains: &[&ValueSet],
        solution: &mut [i32],
        solutions: &mut Vec<Vec<i32>>)
    {
        let collides = |n: i32, vals: &[i32]| {
            (0..i).filter(|&j| vals[j] == n)
                .any(|j| puzzle.shared_vector(cell_ids[i], cell_ids[j]).is_some())
        };
        if remain_product <= 0 { return }
        if i == solution.len() - 1 {
            if remain_product > puzzle.width() as i32 { return }
            if !cell_domains[i].contains(remain_product) { return }
            if collides(remain_product, &solution[..i]) { return }
            solution[i] = remain_product;
            solutions.push(solution.to_vec());
        } else {
            for n in cell_domains[i] {
                if n > remain_product { break }
                if remain_product % n != 0 { continue }
                if collides(n, &solution[..i]) { continue }
                solution[i] = n;
                Self::init_multiply_next(i + 1, puzzle, remain_product / n, cell_ids, cell_domains, solution, solutions);
            }
        }
    }
}
