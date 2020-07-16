use super::Constraint;
use crate::collections::square::{IsSquare, Square};
use crate::puzzle::solve::markup::{PuzzleMarkup, PuzzleMarkupChanges};
use crate::puzzle::solve::CellVariable;
use crate::puzzle::{CellId, Puzzle};

/// If a cell is solved in a vector, other cells in that vector must not have the same value.
#[derive(Clone)]
pub(crate) struct VectorSolvedCellConstraint<'a> {
    puzzle: &'a Puzzle,
    /// Solved cells that have not been checked
    solved_cells: Vec<CellId>,
}

impl<'a> VectorSolvedCellConstraint<'a> {
    pub fn new(puzzle: &'a Puzzle) -> Self {
        Self {
            puzzle,
            solved_cells: Vec::new(),
        }
    }
}

impl<'a> Constraint<'a> for VectorSolvedCellConstraint<'a> {
    fn notify_changes(
        &mut self,
        changes: &PuzzleMarkupChanges,
        _cell_variables: &Square<CellVariable>,
    ) {
        for (id, _) in changes.cells.solutions() {
            self.solved_cells.push(id);
        }
    }

    fn enforce_partial(
        &mut self,
        markup: &PuzzleMarkup<'_>,
        changes: &mut PuzzleMarkupChanges,
    ) -> bool {
        while let Some(cell_id) = self.solved_cells.pop() {
            let value = markup.cells()[cell_id].solved().unwrap();
            let count = self.enforce_solved_cell(&markup.cells(), cell_id, value, changes);
            if count > 0 {
                return true;
            }
        }
        false
    }
}

impl VectorSolvedCellConstraint<'_> {
    fn enforce_solved_cell(
        &self,
        cell_variables: &Square<CellVariable>,
        cell_id: CellId,
        value: i32,
        changes: &mut PuzzleMarkupChanges,
    ) -> u32 {
        let cell = self.puzzle.cell(cell_id);
        let count = cell
            .vectors()
            .iter()
            .flat_map(|&v| self.puzzle.vector(v).iter())
            .filter(|cell| cell_variables[cell.id()].unsolved_and_contains(value))
            .inspect(|cell| changes.cells.remove_domain_value(cell.id(), value))
            .count() as u32;
        debug!(
            "Removed {} instances of the value {} surrounding solved cell at {:?}",
            count,
            value,
            cell.coord()
        );
        count
    }
}
