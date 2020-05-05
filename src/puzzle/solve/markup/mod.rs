mod changes;

pub use self::changes::PuzzleMarkupChanges;

use crate::collections::Square;
use crate::puzzle::solve::cage_solutions::CageSolutionsSet;
use crate::puzzle::solve::CellVariable;
use crate::puzzle::Puzzle;
use ahash::AHashMap;
use std::convert::TryInto;
use std::mem;

/// Markup on a puzzle including possible cell values and cage solutions
#[derive(Clone)]
pub struct PuzzleMarkup {
    cell_variables: Square<CellVariable>,
    cage_solutions_set: CageSolutionsSet,
    blank_cell_count: u32,
}

impl PuzzleMarkup {
    pub fn new(puzzle: &Puzzle) -> Self {
        Self {
            cell_variables: Square::with_width_and_value(
                puzzle.width(),
                CellVariable::unsolved_with_all(puzzle.width()),
            ),
            cage_solutions_set: CageSolutionsSet::new(puzzle),
            blank_cell_count: puzzle.cell_count() as u32,
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

    pub fn is_completed(&self) -> bool {
        self.blank_cell_count == 0
    }

    pub fn completed_values(&self) -> Option<Square<i32>> {
        if !self.is_completed() {
            return None;
        }
        let values = self
            .cell_variables
            .iter()
            .map(|v| v.solved().unwrap())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        Some(values)
    }

    pub fn sync_changes(&mut self, puzzle: &Puzzle, changes: &mut PuzzleMarkupChanges) {
        // apply cell solutions and collect cell domain removals
        let mut new_cell_domain_removals: AHashMap<_, _> = AHashMap::new();
        for &(index, value) in &changes.cell_solutions {
            let cell_variable =
                mem::replace(&mut self.cell_variables[index], CellVariable::Solved(value));
            let cell_domain = cell_variable.unsolved().unwrap();
            for i in cell_domain {
                if i != value {
                    new_cell_domain_removals
                        .entry(index)
                        .or_insert_with(Vec::new)
                        .push(i);
                }
            }
        }

        // apply cell domain removals and add any resulting cell solutions to changes
        for (&index, values) in &changes.cell_domain_value_removals {
            let cell_variable = &mut self.cell_variables[index];
            let domain = cell_variable.unsolved_mut().unwrap();
            for &value in values {
                domain.remove(value);
            }
            if let Some(solution) = cell_variable.solve() {
                changes.cell_solutions.push((index, solution));
            }
        }

        // add previously collected cell domain removals to changes
        changes
            .cell_domain_value_removals
            .extend(new_cell_domain_removals);

        self.cage_solutions_set.apply_changes(puzzle, changes);
        self.blank_cell_count -= changes.cell_solutions.len() as u32;
    }
}
