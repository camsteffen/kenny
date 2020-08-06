use linked_hash_set::LinkedHashSet;

use super::Constraint;
use crate::puzzle::{CageId, Puzzle};
use crate::solve::cage_solutions::CageSolutions;
use crate::solve::markup::{PuzzleMarkup, PuzzleMarkupChanges};
use crate::solve::CellVariable;
use crate::square::Square;

/// If a cage has one remaining cage solution, solve the cage
#[derive(Clone)]
pub(crate) struct CageSolutionConstraint<'a> {
    puzzle: &'a Puzzle,
    /// Cages that have not been checked
    dirty_cages: LinkedHashSet<CageId>,
}

impl<'a> CageSolutionConstraint<'a> {
    pub fn new(puzzle: &'a Puzzle) -> Self {
        Self {
            puzzle,
            dirty_cages: LinkedHashSet::new(),
        }
    }
}

impl<'a> Constraint for CageSolutionConstraint<'a> {
    fn notify_changes(
        &mut self,
        changes: &PuzzleMarkupChanges,
        _cell_variables: &Square<CellVariable>,
    ) {
        for &id in changes.cage_solution_removals.keys() {
            self.dirty_cages.insert(id);
        }
    }

    fn enforce_partial(
        &mut self,
        markup: &PuzzleMarkup<'_>,
        changes: &mut PuzzleMarkupChanges,
    ) -> bool {
        while let Some(cage_id) = self.dirty_cages.pop_front() {
            let CageSolutions {
                cell_ids,
                solutions,
                ..
            } = &markup.cage_solutions().unwrap()[cage_id];
            if let [solution] = solutions.as_slice() {
                debug!(
                    "One cage solution remains for cage at {:?}",
                    self.puzzle.cage(cage_id).coord()
                );
                debug_assert!(!solution.is_empty());
                for (i, &value) in solution.iter().enumerate() {
                    changes.cells.solve(cell_ids[i], value);
                }
                return true;
            }
        }
        false
    }
}
