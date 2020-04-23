mod changes;

pub use self::changes::PuzzleMarkupChanges;

use crate::collections::Square;
use crate::puzzle::Puzzle;
use crate::puzzle::solve::CellVariable;
use crate::puzzle::solve::cage_solutions::CageSolutionsSet;
use std::convert::TryInto;
use ahash::AHashMap;

pub struct PuzzleMarkup {
    cell_variables: Square<CellVariable>,
    cage_solutions_set: CageSolutionsSet,
    unsolved_cell_count: u32,
}

impl PuzzleMarkup {
    pub fn new(puzzle: &Puzzle) -> Self {
        Self {
            cell_variables: Square::with_width_and_value(puzzle.width(), CellVariable::unsolved_with_all(puzzle.width())),
            cage_solutions_set: CageSolutionsSet::new(puzzle),
            unsolved_cell_count: puzzle.cell_count() as u32,
        }
    }

    pub fn init_cage_solutions(&mut self, puzzle: &Puzzle) {
        self.cage_solutions_set.init(puzzle, &self.cell_variables);
    }

    pub fn cage_solutions(&self) -> &CageSolutionsSet {
        &self.cage_solutions_set
    }

    pub fn cells(&self) -> &Square<CellVariable> {
        &self.cell_variables
    }

    pub fn is_solved(&self) -> bool {
        self.unsolved_cell_count == 0
    }

    pub fn solution(&self) -> Option<Square<i32>> {
        if !self.is_solved() {
            return None
        }
        let solution = self.cell_variables.iter()
            .map(|v| v.unwrap_solved())
            .collect::<Vec<_>>()
            .try_into().unwrap();
        Some(solution)
    }

    pub fn sync_changes(&mut self, puzzle: &Puzzle, changes: &mut PuzzleMarkupChanges) {

        // apply cell solutions and collect cell domain removals
        let mut new_cell_domain_removals: AHashMap<_, _> = AHashMap::new();
        for &(index, value) in &changes.cell_solutions {
            let cell_variable = &mut self.cell_variables[index];
            {
                let cell_domain = cell_variable.unwrap_unsolved();
                for i in (1..value).chain(value + 1..=puzzle.width() as i32) {
                    if cell_domain.contains(i) {
                        new_cell_domain_removals.entry(index).or_insert_with(Vec::new).push(i);
                    }
                }
            }
            *cell_variable = CellVariable::Solved(value);
        }

        // apply cell domain removals and add any resulting cell solutions to changes
        for (&index, values) in &changes.cell_domain_value_removals {
            let cell_variable = &mut self.cell_variables[index];
            let domain = cell_variable.unwrap_unsolved_mut();
            for &value in values {
                domain.remove(value);
            }
            if let Some(solution) = cell_variable.solve() {
                changes.cell_solutions.push((index, solution));
            }
        }

        // add previously collected cell domain removals to changes
        changes.cell_domain_value_removals.extend(new_cell_domain_removals);

        self.cage_solutions_set.apply_changes(puzzle, changes);
        self.unsolved_cell_count -= changes.cell_solutions.len() as u32;
    }
}
