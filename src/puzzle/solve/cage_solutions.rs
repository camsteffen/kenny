use std::mem;
use std::ops::Deref;
use std::ops::DerefMut;

use vec_map::VecMap;

use super::markup::PuzzleMarkupChanges;
use super::CellVariable;
use crate::collections::iterator_ext::IteratorExt;
use crate::collections::square::{IsSquare, Square, SquareVector};
use crate::puzzle::solve::markup::CellChange;
use crate::puzzle::solve::CellVariable::{Solved, Unsolved};
use crate::puzzle::solve::ValueSet;
use crate::puzzle::Puzzle;
use crate::puzzle::{CageId, Operator};
use crate::puzzle::{CageRef, CellId, Value};
use crate::{HashMap, HashSet};
use itertools::Itertools;
use std::iter::FromIterator;

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

    /// Returns a list of new solutions if a cage is solved
    /// Returns Err if a cage is left with no solutions
    pub fn apply_changes(
        &mut self,
        puzzle: &Puzzle,
        changes: &PuzzleMarkupChanges,
    ) -> Result<Vec<(CellId, Value)>, ()> {
        #[derive(Default)]
        struct CageData {
            removed_solution_ids: HashSet<usize>,
            solved_cells: HashMap<CellId, Value>,
        }

        let mut data: VecMap<CageData> = VecMap::default();

        for (&cage_id, values) in &changes.cage_solution_removals {
            data.entry(cage_id)
                .or_default()
                .removed_solution_ids
                .extend(values);
        }

        for (id, value) in changes.cells.solutions() {
            let cage_id = puzzle.cell(id).cage_id();
            let cage_data = data.entry(cage_id).or_default();
            cage_data.solved_cells.insert(id, value);
        }

        data.into_iter()
            .map(|(cage_id, cage_data)| {
                self[cage_id].apply_changes(
                    changes,
                    &cage_data.removed_solution_ids,
                    &cage_data.solved_cells,
                    puzzle.cage(cage_id),
                )
            })
            .collect::<Result<Concat<_>, _>>()
            .map(|concat| concat.0)
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
        changes: &PuzzleMarkupChanges,
        // todo just use changes?
        removed_solution_indices: &HashSet<usize>,
        solved_cells: &HashMap<CellId, Value>,
        cage: CageRef<'_>,
    ) -> Result<Vec<(CellId, Value)>, ()> {
        let cell_ids_len = self.cell_ids.len() - solved_cells.len();
        if cell_ids_len == 0 {
            // all the cells in the cage have been solved
            self.clear();
            return Ok(Vec::new());
        }
        let solutions_len = self.solutions.len() - removed_solution_indices.len();
        if solutions_len == 0 {
            // all solutions removed, bail!
            debug!("No solutions left for cage at {:?}", cage.coord());
            return Err(());
        }

        let keep_indices: Vec<usize> = self
            .cell_ids
            .iter()
            .enumerate()
            .filter(|&(_, id)| !solved_cells.contains_key(id))
            .map(|(i, _)| i)
            .collect_into(Vec::with_capacity(cell_ids_len));
        self.solutions = mem::take(&mut self.solutions)
            .into_iter()
            .enumerate()
            .filter(|(i, _)| !removed_solution_indices.contains(i))
            // todo remove redundant constraint
            .filter(|(_, solution)| {
                solved_cells
                    .iter()
                    .all(|(cell_id, &v)| solution[self.index_map[cell_id]] == v)
            })
            .filter_map(|(_, solution)| {
                keep_indices
                    .iter()
                    .map(|&i| {
                        let value = solution[i];
                        match changes.cells.get(self.cell_ids[i]) {
                            // todo change to set
                            // todo remove redundant constraint
                            Some(&CellChange::DomainRemovals(ref values))
                                if values.contains(&value) =>
                            {
                                Err(())
                            }
                            _ => Ok(value),
                        }
                    })
                    .collect::<Result<_, _>>()
                    .ok()
            })
            .dedup()
            .collect_into(Vec::with_capacity(solutions_len));

        self.cell_ids = keep_indices.iter().map(|&i| self.cell_ids[i]).collect();
        debug_assert_eq!(self.cell_ids.len(), cell_ids_len);

        if self.solutions.len() == 1 {
            self.index_map.clear();
            let solution = mem::take(&mut self.solutions).into_iter().next().unwrap();
            let cell_solutions = mem::take(&mut self.cell_ids)
                .into_iter()
                .zip(solution)
                .collect();
            return Ok(cell_solutions);
        }

        self.reset_index_map();
        Ok(Vec::new())
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

    fn reset_index_map(&mut self) {
        self.index_map = Self::build_index_map(&self.cell_ids);
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

struct Concat<T>(Vec<T>);

impl<T> FromIterator<Vec<T>> for Concat<T> {
    fn from_iter<I: IntoIterator<Item = Vec<T>>>(iter: I) -> Self {
        Concat(iter.into_iter().flatten().collect())
    }
}
