use crate::collections::square::{IsSquare, Square};
use crate::puzzle::{CellId, Puzzle};
use crate::solve::cage_solutions::CageSolutionsSet;
use crate::solve::CellVariable;
use itertools::Itertools;
use std::convert::TryInto;

pub(crate) use self::changes::{CellChange, CellChanges, PuzzleMarkupChanges};

mod changes;

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
    #[must_use]
    pub fn sync_changes(&self, changes: &mut PuzzleMarkupChanges) -> bool {
        if !self.sync_cells(changes) {
            return false;
        }
        if let Some(ref cage_solutions_set) = self.cage_solutions_set {
            if !cage_solutions_set.sync_changes(self.puzzle, changes) {
                return false;
            }
        }
        true
    }

    #[must_use]
    fn sync_cells(&self, changes: &mut PuzzleMarkupChanges) -> bool {
        for (&id, change) in &mut changes.cells {
            let cell_variable = &self.cell_variables[id];
            if let CellChange::DomainRemovals(values) = change {
                let domain = cell_variable.unsolved().unwrap();
                match domain.len() - values.len() {
                    0 => {
                        debug!(
                            "Cell domain at {:?} is empty",
                            self.cell_variables.cell(id).coord()
                        );
                        return false;
                    }
                    1 => {
                        let solution = domain
                            .iter()
                            .filter(|i| !values.contains(i))
                            .exactly_one()
                            .ok()
                            .unwrap();
                        *change = CellChange::Solution(solution);
                    }
                    _ => (),
                }
            }
        }
        true
    }

    /// Returns true if the puzzle is solved or solvable
    pub fn apply_changes(&mut self, changes: &PuzzleMarkupChanges) {
        self.apply_cell_changes(&changes.cells);
        if let Some(ref mut cage_solutions_set) = self.cage_solutions_set {
            cage_solutions_set.apply_changes(self.puzzle, changes);
        }
        self.blank_cell_count -= changes.cells.solutions().count() as u32;
    }

    pub fn apply_cell_changes(&mut self, changes: &CellChanges) {
        for (&id, change) in changes {
            self.apply_cell_change(id, change)
        }
    }

    pub fn apply_cell_change(&mut self, id: CellId, change: &CellChange) {
        let cell_variable = &mut self.cell_variables[id];
        match change {
            CellChange::DomainRemovals(removals) => {
                let domain = cell_variable.unsolved_mut().unwrap();
                for value in removals {
                    domain.remove(*value);
                }
            }
            CellChange::Solution(value) => {
                *cell_variable = CellVariable::Solved(*value);
            }
        }
    }
}
