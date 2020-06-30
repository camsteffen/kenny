mod changes;

pub(crate) use self::changes::{CellChange, CellChanges, PuzzleMarkupChanges};

use crate::collections::square::{IsSquare, Square};
use crate::puzzle::solve::cage_solutions::CageSolutionsSet;
use crate::puzzle::solve::CellVariable;
use crate::puzzle::Puzzle;
use std::convert::TryInto;

/// Markup on a puzzle including possible cell values and cage solutions
#[derive(Clone)]
pub(crate) struct PuzzleMarkup<'a> {
    puzzle: &'a Puzzle,
    cell_variables: Square<CellVariable>,
    cage_solutions_set: Option<CageSolutionsSet>,
    blank_cell_count: u32,
}

impl<'a> PuzzleMarkup<'a> {
    pub fn new(puzzle: &'a Puzzle) -> Self {
        Self {
            puzzle,
            cell_variables: Square::with_width_and_value(
                puzzle.width(),
                CellVariable::unsolved_with_all(puzzle.width()),
            ),
            cage_solutions_set: None,
            blank_cell_count: puzzle.cell_count() as u32,
        }
    }

    pub fn init_cage_solutions(&mut self, puzzle: &Puzzle) {
        debug_assert!(self.cage_solutions_set.is_none());
        self.cage_solutions_set = Some(CageSolutionsSet::init(puzzle, &self.cell_variables));
    }

    pub fn cage_solutions(&self) -> Option<&CageSolutionsSet> {
        self.cage_solutions_set.as_ref()
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

    /// Returns true if the puzzle is solved or solvable
    pub fn sync_changes(&mut self, changes: &mut PuzzleMarkupChanges) -> bool {
        if !self.sync_cells(changes) {
            return false;
        }
        if let Some(ref mut cage_solutions_set) = self.cage_solutions_set {
            if !cage_solutions_set.apply_changes(self.puzzle, changes) {
                return false;
            }
        }
        self.blank_cell_count -= changes.cells.solutions().count() as u32;
        true
    }

    fn sync_cells(&mut self, changes: &mut PuzzleMarkupChanges) -> bool {
        for (&id, change) in &mut changes.cells {
            let cell_variable = &mut self.cell_variables[id];
            match change {
                CellChange::DomainRemovals(values) => {
                    let domain = cell_variable.unsolved_mut().unwrap();
                    for value in values {
                        domain.remove(*value);
                    }
                    if domain.is_empty() {
                        debug!(
                            "Cell domain at {:?} is empty",
                            self.cell_variables.cell(id).coord()
                        );
                        return false;
                    } else if let Some(solution) = cell_variable.solve() {
                        *change = CellChange::Solution(solution);
                    }
                }
                CellChange::Solution(value) => {
                    *cell_variable = CellVariable::Solved(*value);
                }
            }
        }
        true
    }
}
