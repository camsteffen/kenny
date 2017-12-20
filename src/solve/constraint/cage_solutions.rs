use collections::square::SquareIndex;
use puzzle::Cage;
use puzzle::Operator;
use puzzle::Puzzle;
use solve::CellDomain;
use solve::Solver;
use solve::CellVariable;
use solve::CellVariable::{Solved, Unsolved};
use std::collections::{BTreeMap, BTreeSet};
use collections::GetByIndicies;
use collections::Square;
use solve::DomainChangeSet;

/// This struct is used to keep track of all possible solutions for all cages in the puzzle.
/// When this constraint is enforced, all cell domain values must be part of a possible cage solution.
pub struct CageSolutionsConstraint {
    size: usize,
    dirty: BTreeSet<usize>,
}

impl CageSolutionsConstraint {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            dirty: BTreeSet::new(),
        }
    }

    fn notify_cage_solutions_changed(&mut self, cage_id: usize) {
        self.dirty.insert(cage_id);
    }

    fn enforce(&self, puzzle: &Puzzle, cell_variables: &Square<CellVariable>, cage_solutions: &PuzzleCageSolutions, changes: &mut DomainChangeSet) {
        let cage_id = self.dirty.iter().next().cloned().unwrap();
        let cage_solutions = cage_solutions.cage_solutions(cage_id);
        self.enforce_cage(puzzle, cell_variables, cage_id, cage_solutions, changes);
        self.dirty.remove(&cage_id);
    }

    fn enforce_cage(&self, puzzle: &Puzzle, cell_variables: &Square<CellVariable>, cage_id: usize, cage_solutions: &CageSolutions, changes: &mut DomainChangeSet) {
        // assemble domain for each unsolved cell from cell solutions
        let mut soln_domain = vec![CellDomain::new(self.size); cage_solutions.num_cells()];
        for solution in &cage_solutions.solutions {
            for i in 0..cage_solutions.num_cells() {
                soln_domain[i].insert(solution[i]);
            }
        }

        let cell_domains = cell_variables.get_indicies(&cage_solutions.indicies).into_iter()
                .map(|variable| variable.unwrap_unsolved()).collect::<Vec<_>>();

        // remove values from cell domains that are not in a cage solution
        let mut to_remove = Vec::new();
        for (i, sq_index) in cage_solutions.indicies.iter().cloned().enumerate() {
            let domain = cell_variables[sq_index].unwrap_unsolved();
            let no_solutions;
            {
                no_solutions = domain.iter()
                    .filter(|&n| !soln_domain[i].contains(n))
                    .collect::<Vec<_>>();
            }
            for n in no_solutions {
                changes.remove_value_from_cell(sq_index, n);
            }
        }
    }
}

pub struct PuzzleCageSolutions {
    data: Vec<CageSolutions>,
    cell_domain_value_removals: Vec<Vec<(usize, i32)>>,
}

impl PuzzleCageSolutions {

    fn new(cages: &[Cage]) -> Self {
        Self {
            data: cages.iter().map(|cage| CageSolutions::new(&cage.cells)).collect(),
            cell_domain_value_removals: vec![Vec::new(); cages.len()],
        }
    }

    fn notify_cell_domain_value_removed(&self, cage_index: usize, cage_cell_index: usize, value: i32) {
        self.cell_domain_value_removals[cage_index].push((cage_cell_index, value));
    }

    fn cage_solutions(&self, cage_index: usize) -> &CageSolutions {
        self.sync_cage(cage_index);
        &self.data[cage_index]
    }

    fn sync_cage(&mut self, cage_index: usize) {
        let cage_solutions = self.data[cage_index];
        let domain_values = self.cell_domain_value_removals[cage_index];
        cage_solutions.remove_cell_domain_values(&domain_values);
        domain_values.clear();
    }
}

pub struct CageSolutions {
    indicies: Vec<SquareIndex>,
    solutions: Vec<Vec<i32>>,
}

impl CageSolutions {

    pub fn new(indicies: &[SquareIndex]) -> Self {
        Self {
            indicies: indicies.to_vec(),
            solutions: Vec::new(),
        }
    }

    fn init(&mut self, puzzle_width: usize, cage: &Cage, cell_variables: &[&CellVariable]) {
        self.solutions = match cage.operator {
            Operator::Add => self.init_add(puzzle_width, cage, cell_variables),
            Operator::Multiply => self.init_multiply(puzzle_width, cage, cell_variables),
            Operator::Subtract => self.init_subtract(puzzle_width, cage, cell_variables),
            Operator::Divide => self.init_divide(puzzle_width, cage, cell_variables),
            Operator::Nop => unreachable!(),
        };

        debug!("solutions: {:?}", self.solutions);
    }

    fn num_cells(&self) -> usize {
        self.indicies.len()
    }

    fn remove_indicies(&mut self, indicies: &[SquareIndex]) {
        let capacity = self.indicies.len() - indicies.len();
        if capacity == 0 {
            self.indicies.clear();
            self.solutions.clear();
            return
        }
        let steps = indicies.iter().scan((0, &self.indicies[..]), |&mut (start, indicies), source_index| {
            let index = indicies.binary_search(source_index).unwrap();
            indicies = &indicies[index..];
            Some(index)
        }).collect::<Vec<_>>();
        let indicies_iter = self.indicies.into_iter();
        let solutions_iter = self.solutions.into_iter();
        self.indicies = Vec::with_capacity(capacity);
        self.solutions = Vec::with_capacity(capacity);
        for step in steps {
            for i in 0..step {
                self.indicies.push(indicies_iter.next().unwrap());
                self.solutions.push(solutions_iter.next().unwrap());
            }
            indicies_iter.next();
            solutions_iter.next();
        }
        self.indicies.extend(indicies_iter);
        self.solutions.extend(solutions_iter);
    }

    fn remove_cell_domain_values(&mut self, domain_values: &[(usize, i32)]) {
        self.solutions.retain(|solution| {
            domain_values.iter().all(|&(index, value)| solution[index] != value)
        });
    }

    fn init_add(&mut self, puzzle_width: usize, cage: &Cage, cell_variables: &[&CellVariable]) -> Vec<Vec<i32>> {
        let mut indicies = Vec::new();
        let mut cell_domains = Vec::new();
        let mut solved_sum = 0;
        for (i, &&cell_variable) in cell_variables.iter().enumerate() {
            match cell_variable {
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
        for (i, &&cell_variable) in cell_variables.iter().enumerate() {
            match cell_variable {
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