use collections::square::SquareIndex;
use puzzle::Cage;
use puzzle::Operator;
use puzzle::Puzzle;
use puzzle::solve::CellDomain;
use puzzle::solve::CellVariable::*;
use collections::Square;
use std::ops::Deref;
use std::ops::DerefMut;
use fnv::FnvHashMap;
use fnv::FnvHashSet;
use std::mem;
use collections::GetIndiciesCloned;
use super::CellVariable;
use super::markup::PuzzleMarkupChanges;

pub struct CageSolutionsSet {
    cage_map: Square<usize>,
    data: Vec<CageSolutions>,
}

impl CageSolutionsSet {
    pub fn new(puzzle: &Puzzle) -> Self {
        Self {
            cage_map: puzzle.cage_map.clone(),
            data: puzzle.cages.iter().map(|cage| CageSolutions::new(&cage.cells)).collect(),
        }
    }

    pub fn init(&mut self, puzzle: &Puzzle, cell_variables: &Square<CellVariable>) {
        let mut cages_iter = puzzle.cages.iter();
        for cage_solutions in &mut self.data {
            let cage = cages_iter.next().unwrap();
            if cage.operator != Operator::Nop {
                let cell_variables = cage.cells.iter().cloned().map(|i| &cell_variables[i]).collect::<Vec<_>>();
                cage_solutions.init(puzzle.width, cage, &cell_variables);
            }
        }
    }

    pub fn apply_changes(&mut self, changes: &PuzzleMarkupChanges) {
        struct CageData {
            removed_indices: Vec<SquareIndex>,
            removed_values: Vec<(SquareIndex, i32)>,
        }
        impl CageData {
            fn new() -> Self {
                Self {
                    removed_indices: Vec::new(),
                    removed_values: Vec::new(),
                }
            }
        }

        let mut data = FnvHashMap::default();

        for &(index, _) in &changes.cell_solutions {
            let cage_index = self.cage_map[index];
            data.entry(cage_index).or_insert_with(CageData::new).removed_indices.push(index);
        }

        for &(index, value) in &changes.cell_domain_value_removals {
            let cage_index = self.cage_map[index];
            data.entry(cage_index).or_insert_with(CageData::new).removed_values.push((index, value));
        }

        for (cage_index, cage_data) in data {
            self[cage_index].apply_changes(&cage_data.removed_indices, &cage_data.removed_values);
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
pub struct CageSolutions {
    /// the indicies of the unsolved cells in the cage
    pub indices: Vec<SquareIndex>,
    /// a list of all possible solutions for a cage. the numbers correspond to the cells represented in indicies
    pub solutions: Vec<Vec<i32>>,
    index_map: FnvHashMap<SquareIndex, usize>,
}

impl CageSolutions {

    pub fn new(indices: &[SquareIndex]) -> Self {
        Self {
            indices: indices.to_vec(),
            solutions: Vec::new(),
            index_map: Self::build_index_map(indices),
        }
    }

    pub fn init(&mut self, puzzle_width: usize, cage: &Cage, cell_variables: &[&CellVariable]) {
        self.solutions = match cage.operator {
            Operator::Add => self.init_add(puzzle_width, cage, cell_variables),
            Operator::Multiply => self.init_multiply(puzzle_width, cage, cell_variables),
            Operator::Subtract => self.init_subtract(puzzle_width, cage, cell_variables),
            Operator::Divide => self.init_divide(puzzle_width, cage, cell_variables),
            Operator::Nop => panic!("cannot init CageSolutions with a Nop cage"),
        };

        debug!("solutions: {:?}", self.solutions);
    }

    pub fn num_cells(&self) -> usize {
        self.indices.len()
    }

    pub fn apply_changes(&mut self, removed_indices: &[SquareIndex], removed_values: &[(SquareIndex, i32)]) {

        // create hashmap of removed solution indices
        let removed_indices = removed_indices.iter().map(|i| self.index_map[i]).collect::<FnvHashSet<_>>();

        let len = self.indices.len() - removed_indices.len();
        let indices = mem::replace(&mut self.indices, Vec::with_capacity(len));
        let mut keep_indices = Vec::with_capacity(len);
        let indices = indices.into_iter().enumerate().filter(|&(i, _)| !removed_indices.contains(&i));
        for (i, sq_idx) in indices {
            self.indices.push(sq_idx);
            keep_indices.push(i);
        }
        let removed_values = removed_values.iter().map(|&(i, v)| (self.index_map[&i], v)).collect::<Vec<_>>();
        let solutions = mem::replace(&mut self.solutions, Vec::new());
        self.solutions = solutions.into_iter()
            .filter_map(|solution| {
                for &(index, value) in &removed_values {
                    if solution[index] == value {
                        return None
                    }
                }
                Some(solution.get_indices_cloned(&keep_indices))
            })
            .collect();
    }

    pub fn remove_indices(&mut self, indices: &[SquareIndex]) {
        let capacity = self.indices.len() - indices.len();
        if capacity == 0 {
            self.indices.clear();
            self.solutions.clear();
            self.index_map.clear();
            return
        }
        let indices_set = indices.iter().cloned().collect::<FnvHashSet<_>>();
        let solution_indices_set = indices.iter()
            .map(|&i| self.indices.iter().position(|&j| j == i).unwrap())
            .collect::<FnvHashSet<_>>();
        self.indices.retain(|i| !indices_set.contains(i));
        for solution in &mut self.solutions {
            *solution = solution.into_iter().enumerate()
                .filter_map(|(i, &mut j)| {
                    if solution_indices_set.contains(&i) { None } else { Some(j) }
                }).collect();
        }
        self.reset_index_map();
    }

    pub fn remove_cell_domain_value(&mut self, index: SquareIndex, value: i32) {
        let index = self.index_map[&index];
        self.solutions.retain(|solution| solution[index] != value);
    }

    fn build_index_map(indicies: &[SquareIndex]) -> FnvHashMap<SquareIndex, usize> {
        indicies.iter().cloned().enumerate().map(|(i, sq_i)| (sq_i, i)).collect()
    }

    fn reset_index_map(&mut self) {
        self.index_map = Self::build_index_map(&self.indices);
    }

    fn init_add(&mut self, puzzle_width: usize, cage: &Cage, cell_variables: &[&CellVariable]) -> Vec<Vec<i32>> {
        let mut indicies = Vec::new();
        let mut cell_domains = Vec::new();
        let mut solved_sum = 0;
        for (i, &cell_variable) in cell_variables.iter().enumerate() {
            match *cell_variable {
                Solved(v) => solved_sum += v,
                Unsolved(ref domain) => {
                    indicies.push(cage.cells[i]);
                    cell_domains.push(domain);
                },
            }
        }
        let remain_sum = cage.target - solved_sum;
        let mut solution = vec![0; indicies.len()];
        let mut solutions = Vec::new();
        Self::init_add_next(0, puzzle_width, remain_sum, &indicies, &cell_domains, &mut solution, &mut solutions);
        solutions
    }

    fn init_multiply(&mut self, puzzle_width: usize, cage: &Cage, cell_variables: &[&CellVariable]) -> Vec<Vec<i32>> {
        let mut indicies = Vec::new();
        let mut cell_domains = Vec::new();
        let mut solved_product = 1;
        for (i, &cell_variable) in cell_variables.iter().enumerate() {
            match *cell_variable {
                Solved(v) => solved_product *= v,
                Unsolved(ref domain) => {
                    indicies.push(cage.cells[i]);
                    cell_domains.push(domain);
                },
            }
        }
        let remain_product = cage.target / solved_product;
        let mut solution = vec![0; indicies.len()];
        let mut solutions = Vec::new();
        Self::init_multiply_next(0, puzzle_width, remain_product, &indicies, &cell_domains, &mut solution, &mut solutions);
        solutions
    }

    fn init_subtract(&mut self, puzzle_width: usize, cage: &Cage, cell_variables: &[&CellVariable]) -> Vec<Vec<i32>> {
        debug_assert_eq!(2, cage.cells.len());
        let mut solutions = Vec::new();
        if let Some(solved_pos) = cell_variables.iter().position(|v| v.is_solved()) {
            let known_val = cell_variables[solved_pos].unwrap_solved();
            let domain = cell_variables[(solved_pos + 1) % 2].unwrap_unsolved();
            let n = known_val - cage.target;
            if n > 0 && domain.contains(n) {
                solutions.push(vec![n; 1]);
            }
            let n = known_val + cage.target;
            if n <= puzzle_width as i32 && domain.contains(n) {
                solutions.push(vec![n; 1]);
            }
        } else {
            let domains = cell_variables.iter().map(|variable| variable.unwrap_unsolved()).collect::<Vec<_>>();
            for n in domains[0] {
                let m = n - cage.target;
                if m > 0 && domains[1].contains(m) {
                    solutions.push(vec![n, m]);
                }
                let m = n + cage.target;
                if m <= puzzle_width as i32 && domains[1].contains(m) {
                    solutions.push(vec![n, m]);
                }
            }
        }
        solutions
    }

    fn init_divide(&mut self, puzzle_width: usize, cage: &Cage, cell_variables: &[&CellVariable]) -> Vec<Vec<i32>> {
        debug_assert_eq!(2, cage.cells.len());
        let mut solutions = Vec::new();
        if let Some(solved_pos) = cell_variables.iter().position(|v| v.is_solved()) {
            let known_val = cell_variables[solved_pos].unwrap_solved();
            let domain = cell_variables[(solved_pos + 1) % 2].unwrap_unsolved();
            let n = known_val / cage.target;
            if n > 0 && domain.contains(n) {
                solutions.push(vec![n; 1]);
            }
            let n = known_val * cage.target;
            if n <= puzzle_width as i32 && domain.contains(n) {
                solutions.push(vec![n; 1]);
            }
        } else {
            let domains = cell_variables.iter().map(|variable| variable.unwrap_unsolved()).collect::<Vec<_>>();
            for n in domains[0] {
                let m = n / cage.target;
                if m > 0 && n % cage.target == 0 && domains[1].contains(m) {
                    solutions.push(vec![n, m]);
                }
                let m = n * cage.target;
                if m <= puzzle_width as i32 && domains[1].contains(m) {
                    solutions.push(vec![n, m]);
                }
            }
        }
        solutions
    }

    fn init_add_next(
        i: usize,
        puzzle_width: usize,
        remain_sum: i32,
        indicies: &[SquareIndex],
        cell_domains: &[&CellDomain],
        solution: &mut [i32],
        solutions: &mut Vec<Vec<i32>>)
    {
        let collides = |n: i32, vals: &[i32]| {
            (0..i).filter(|&j| vals[j] == n)
                .any(|j| indicies[i].shared_vector(indicies[j], puzzle_width).is_some())
        };
        if remain_sum <= 0 { return }
        if i == solution.len() - 1 {
            if remain_sum > puzzle_width as i32 { return }
            if !cell_domains[i].contains(remain_sum) { return }
            if collides(remain_sum, &solution[..i]) { return }
            solution[i] = remain_sum;
            solutions.push(solution.to_vec());
        } else {
            for n in cell_domains[i] {
                if n >= remain_sum { break }
                if collides(n, &solution[..i]) { continue }
                solution[i] = n;
                Self::init_add_next(i + 1, puzzle_width, remain_sum - n, indicies, cell_domains, solution, solutions);
            }
        }
    }

    fn init_multiply_next(
        i: usize,
        puzzle_width: usize,
        remain_product: i32,
        indicies: &[SquareIndex],
        cell_domains: &[&CellDomain],
        solution: &mut [i32],
        solutions: &mut Vec<Vec<i32>>)
    {
        let collides = |n: i32, vals: &[i32]| {
            (0..i).filter(|&j| vals[j] == n)
                .any(|j| indicies[i].shared_vector(indicies[j], puzzle_width).is_some())
        };
        if remain_product <= 0 { return }
        if i == solution.len() - 1 {
            if remain_product > puzzle_width as i32 { return }
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
                Self::init_multiply_next(i + 1, puzzle_width, remain_product / n, indicies, cell_domains, solution, solutions);
            }
        }
    }
}
