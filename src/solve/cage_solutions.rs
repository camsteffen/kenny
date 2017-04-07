use std::ops::Deref;
use std::ops::DerefMut;

use vec_map::VecMap;

use super::markup::PuzzleMarkupChanges;
use super::CellVariable;
use crate::collections::iterator_ext::IteratorExt;
use crate::collections::square::{IsSquare, Square, SquareVector};
use crate::collections::vec_ext::VecExt;
use crate::puzzle::Puzzle;
use crate::puzzle::{CageId, Operator};
use crate::puzzle::{CellId, Value};
use crate::solve::markup::CellChange;
use crate::solve::CellVariable::{Solved, Unsolved};
use crate::solve::ValueSet;
use crate::{HashMap, HashSet};

#[derive(Clone)]
pub(crate) struct CageSolutionsSet {
    data: Vec<CageSolutions>,
}

impl CageSolutionsSet {
    pub fn init(puzzle: &Puzzle, cell_variables: &Square<CellVariable>) -> Self {
        let data = puzzle
            .cages()
            .map(|cage| {
                let cell_variables: Vec<_> = cage
                    .cells()
                    .map(|cell| &cell_variables[cell.id()])
                    .collect();
                CageSolutions::init(puzzle, cage.id(), &cell_variables)
            })
            .collect();
        Self { data }
    }

    /// Returns false if a cage is left unsolvable
    #[must_use]
    pub fn sync_changes(&self, puzzle: &Puzzle, changes: &mut PuzzleMarkupChanges) -> bool {
        for cage_id in Self::changed_cage_ids(puzzle, changes) {
            let solutions = &self.data[cage_id];
            let valid_count = solutions
                .solutions
                .iter()
                .enumerate()
                .filter(|&(i, solution)| {
                    if changes.is_cage_solution_removed(cage_id, i) {
                        return false;
                    }
                    let valid = solution.iter().enumerate().all(|(i, value)| {
                        match changes.cells.get(solutions.cell_ids[i]) {
                            Some(CellChange::Solution(cell_solution)) => value == cell_solution,
                            Some(CellChange::DomainRemovals(removals)) => !removals.contains(value),
                            None => true,
                        }
                    });
                    if !valid {
                        changes.remove_cage_solution(cage_id, i);
                    }
                    valid
                })
                .count();
            if valid_count == 0 {
                debug!(
                    "No solutions left for cage at {:?}",
                    puzzle.cage(cage_id).coord()
                );
                return false;
            }
        }
        true
    }

    fn changed_cage_ids(puzzle: &Puzzle, changes: &mut PuzzleMarkupChanges) -> HashSet<usize> {
        changes
            .cells
            .keys()
            .map(|&id| puzzle.cell(id).cage_id())
            .chain(changes.cage_solution_removals.keys().copied())
            .collect()
    }

    /// Returns a list of new solutions if a cage is solved
    /// Returns Err if a cage is left with no solutions
    pub fn apply_changes(&mut self, puzzle: &Puzzle, changes: &PuzzleMarkupChanges) {
        #[derive(Default)]
        struct CageData {
            removed_solution_ids: HashSet<usize>,
            solved_cells: HashSet<CellId>,
        }

        let mut data: VecMap<CageData> = VecMap::default();

        for (&cage_id, values) in &changes.cage_solution_removals {
            data.entry(cage_id)
                .or_default()
                .removed_solution_ids
                .extend(values);
        }

        for (id, _value) in changes.cells.solutions() {
            let cage_id = puzzle.cell(id).cage_id();
            let cage_data = data.entry(cage_id).or_default();
            cage_data.solved_cells.insert(id);
        }

        for (cage_id, cage_data) in data {
            self[cage_id].apply_changes(&cage_data.removed_solution_ids, &cage_data.solved_cells);
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
pub(crate) struct CageSolutions {
    /// the indices of the unsolved cells in the cage (not every cell in the cage)
    pub cell_ids: Vec<CellId>,
    /// a list of all possible solutions for a cage. the numbers correspond to the cells represented in cell_ids
    pub solutions: Vec<Vec<i32>>,
    /// Cage cell index -> Cage unsolved cell index
    pub index_map: HashMap<CellId, usize>,
}

impl CageSolutions {
    pub fn init(puzzle: &Puzzle, cage_id: CageId, cell_variables: &[&CellVariable]) -> Self {
        let cage = puzzle.cage(cage_id);
        let cell_ids: Vec<_> = cage
            .cell_ids()
            .iter()
            .zip(cell_variables)
            .filter(|(_, v)| v.is_unsolved())
            .map(|(&id, _)| id)
            .collect();
        let index_map = Self::build_index_map(&cell_ids);

        let solutions = match cage.operator() {
            Operator::Add => Self::init_add(puzzle, cage_id, cell_variables),
            Operator::Multiply => Self::init_multiply(puzzle, cage_id, cell_variables),
            Operator::Subtract => Self::init_subtract(puzzle, cage_id, cell_variables),
            Operator::Divide => Self::init_divide(puzzle, cage_id, cell_variables),
            Operator::Nop => Vec::new(),
        };

        debug!("cage at {:?} solutions: {:?}", cage.coord(), &solutions);

        Self {
            cell_ids,
            solutions,
            index_map,
        }
    }

    fn clear(&mut self) {
        self.cell_ids.clear();
        self.index_map.clear();
        self.solutions.clear();
    }

    pub fn num_cells(&self) -> usize {
        self.cell_ids.len()
    }

    /// Returns Ok if the cage is still solvable with a list of newly solved cells if the
    /// cage is now solved. Returns Err if the cage is left unsolvable.
    pub fn apply_changes(
        &mut self,
        removed_solution_indices: &HashSet<usize>,
        solved_cells: &HashSet<CellId>,
    ) {
        debug_assert!(!removed_solution_indices.is_empty() || !solved_cells.is_empty());

        if solved_cells.len() == self.cell_ids.len() {
            // all the cells in the cage have been solved
            self.clear();
            return;
        }
        if !removed_solution_indices.is_empty() {
            self.solutions
                .retain_indexed(|i, _| !removed_solution_indices.contains(&i));
        }
        if !solved_cells.is_empty() {
            let remove_indices = self
                .cell_ids
                .iter()
                .enumerate()
                .filter(|&(_, id)| solved_cells.contains(id))
                .map(|(i, _)| i)
                .collect_into(Vec::with_capacity(solved_cells.len()));

            for solution in &mut self.solutions {
                solution.remove_indices_copy(&remove_indices);
            }

            self.cell_ids.remove_indices_copy(&remove_indices);

            for id in solved_cells {
                self.index_map.remove(id);
            }
        }
    }

    pub fn vector_view<T: IsSquare>(&self, vector: SquareVector<'_, T>) -> CageSolutionsView<'_> {
        let indices = self
            .cell_ids
            .iter()
            .copied()
            .enumerate()
            .filter(|&(_, id)| vector.contains_square_index(id))
            .map(|(i, _)| i)
            .collect();
        CageSolutionsView {
            cage_solutions: self,
            indices,
        }
    }

    fn build_index_map(indices: &[CellId]) -> HashMap<CellId, usize> {
        indices.iter().enumerate().map(|(i, &id)| (id, i)).collect()
    }

    fn init_add(
        puzzle: &Puzzle,
        cage_id: CageId,
        cell_variables: &[&CellVariable],
    ) -> Vec<Vec<i32>> {
        let cage = puzzle.cage(cage_id);
        let mut indices = Vec::new();
        let mut cell_domains = Vec::new();
        let mut solved_sum = 0_i32;
        for (i, &cell_variable) in cell_variables.iter().enumerate() {
            match cell_variable {
                &Solved(v) => solved_sum += v,
                Unsolved(domain) => {
                    indices.push(cage.cell(i).id());
                    cell_domains.push(domain);
                }
            }
        }
        let remain_sum = cage.target() - solved_sum;
        let mut solution = vec![0; indices.len()];
        let mut solutions = Vec::new();
        Self::init_add_next(
            0,
            puzzle,
            remain_sum,
            &indices,
            &cell_domains,
            &mut solution,
            &mut solutions,
        );
        solutions
    }

    fn init_multiply(
        puzzle: &Puzzle,
        cage_id: CageId,
        cell_variables: &[&CellVariable],
    ) -> Vec<Vec<i32>> {
        let cage = puzzle.cage(cage_id);
        let mut indices = Vec::new();
        let mut cell_domains = Vec::new();
        let mut solved_product = 1;
        for (i, &cell_variable) in cell_variables.iter().enumerate() {
            match cell_variable {
                &Solved(v) => solved_product *= v,
                Unsolved(domain) => {
                    indices.push(cage.cell(i).id());
                    cell_domains.push(domain);
                }
            }
        }
        let remain_product = cage.target() / solved_product;
        let mut solution = vec![0; indices.len()];
        let mut solutions = Vec::new();
        Self::init_multiply_next(
            0,
            puzzle,
            remain_product,
            &indices,
            &cell_domains,
            &mut solution,
            &mut solutions,
        );
        solutions
    }

    fn init_subtract(
        puzzle: &Puzzle,
        cage_id: CageId,
        cell_variables: &[&CellVariable],
    ) -> Vec<Vec<i32>> {
        let cage = puzzle.cage(cage_id);
        debug_assert_eq!(cage.cell_count(), 2);
        let mut solutions = Vec::new();
        if let Some(solved_pos) = cell_variables.iter().position(|v| v.is_solved()) {
            let known_val = cell_variables[solved_pos].solved().unwrap();
            let domain = cell_variables[(solved_pos + 1) % 2].unsolved().unwrap();
            let n = known_val - cage.target();
            if n > 0 && domain.contains(n) {
                solutions.push(vec![n; 1]);
            }
            let m = known_val + cage.target();
            if m <= puzzle.width() as i32 && domain.contains(m) {
                solutions.push(vec![m; 1]);
            }
        } else {
            let domains = cell_variables
                .iter()
                .map(|variable| variable.unsolved().unwrap())
                .collect::<Vec<_>>();
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

    fn init_divide(
        puzzle: &Puzzle,
        cage_id: CageId,
        cell_variables: &[&CellVariable],
    ) -> Vec<Vec<i32>> {
        let cage = puzzle.cage(cage_id);
        debug_assert_eq!(cage.cell_count(), 2);
        let mut solutions = Vec::new();
        if let Some(solved_pos) = cell_variables.iter().position(|v| v.is_solved()) {
            let known_val = cell_variables[solved_pos].solved().unwrap();
            let domain = cell_variables[(solved_pos + 1) % 2].unsolved().unwrap();
            let n = known_val / cage.target();
            if n > 0 && domain.contains(n) {
                solutions.push(vec![n; 1]);
            }
            let m = known_val * cage.target();
            if m <= puzzle.width() as i32 && domain.contains(m) {
                solutions.push(vec![m; 1]);
            }
        } else {
            let domains = cell_variables
                .iter()
                .map(|variable| variable.unsolved().unwrap())
                .collect::<Vec<_>>();
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
        solutions: &mut Vec<Vec<i32>>,
    ) {
        let collides = |n: i32, vals: &[i32]| -> bool {
            (0..i)
                .filter(|&j| vals[j] == n)
                .any(|j| puzzle.shared_vector(cell_ids[i], cell_ids[j]).is_some())
        };
        if remain_sum <= 0 {
            return;
        }
        if i == solution.len() - 1 {
            if remain_sum > puzzle.width() as i32 {
                return;
            }
            if !cell_domains[i].contains(remain_sum) {
                return;
            }
            if collides(remain_sum, &solution[..i]) {
                return;
            }
            solution[i] = remain_sum;
            solutions.push(solution.to_vec());
        } else {
            for n in cell_domains[i] {
                if n >= remain_sum {
                    break;
                }
                if collides(n, &solution[..i]) {
                    continue;
                }
                solution[i] = n;
                Self::init_add_next(
                    i + 1,
                    puzzle,
                    remain_sum - n,
                    cell_ids,
                    cell_domains,
                    solution,
                    solutions,
                );
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
        solutions: &mut Vec<Vec<i32>>,
    ) {
        let collides = |n: i32, vals: &[i32]| {
            (0..i)
                .filter(|&j| vals[j] == n)
                .any(|j| puzzle.shared_vector(cell_ids[i], cell_ids[j]).is_some())
        };
        if remain_product <= 0 {
            return;
        }
        if i == solution.len() - 1 {
            if remain_product > puzzle.width() as i32 {
                return;
            }
            if !cell_domains[i].contains(remain_product) {
                return;
            }
            if collides(remain_product, &solution[..i]) {
                return;
            }
            solution[i] = remain_product;
            solutions.push(solution.to_vec());
        } else {
            for n in cell_domains[i] {
                if n > remain_product {
                    break;
                }
                if remain_product % n != 0 {
                    continue;
                }
                if collides(n, &solution[..i]) {
                    continue;
                }
                solution[i] = n;
                Self::init_multiply_next(
                    i + 1,
                    puzzle,
                    remain_product / n,
                    cell_ids,
                    cell_domains,
                    solution,
                    solutions,
                );
            }
        }
    }
}

pub(crate) struct CageSolutionsView<'a> {
    cage_solutions: &'a CageSolutions,
    indices: Vec<usize>,
}

impl<'a> CageSolutionsView<'a> {
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    pub fn len(&self) -> usize {
        self.indices.len()
    }

    pub fn solutions<'b>(&'b self) -> impl Iterator<Item = Vec<Value>> + 'b {
        self.cage_solutions
            .solutions
            .iter()
            .map(move |solution| self.indices.iter().map(|&i| solution[i]).collect())
    }
}
