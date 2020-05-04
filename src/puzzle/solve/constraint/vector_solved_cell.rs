use crate::collections::Square;
use crate::puzzle::{Puzzle, CellRef, CellId};
use crate::puzzle::solve::CellVariable;
use crate::puzzle::solve::markup::PuzzleMarkupChanges;
use crate::puzzle::solve::PuzzleMarkup;
use super::Constraint;

/// If a cell is solved in a vector, other cells in that vector must not have the same value.
#[derive(Clone)]
pub struct VectorSolvedCellConstraint {
    /// Solved cells that have not been checked
    solved_cells: Vec<CellId>,
}

impl VectorSolvedCellConstraint {

    pub fn new() -> Self {
        Self {
            solved_cells: Vec::new(),
        }
    }

    pub fn enforce_solved_cell(
        puzzle: &Puzzle,
        cell_variables: &Square<CellVariable>,
        cell: CellRef<'_>,
        value: i32,
        changes: &mut PuzzleMarkupChanges
    ) -> u32 {
        let count = cell.vectors().iter().copied()
            .flat_map(|v| puzzle.vector_cells(v))
            .filter(|cell| cell_variables[cell.id()].unsolved_and_contains(value))
            .map(|cell| changes.remove_value_from_cell(cell.id(), value))
            .count() as u32;
        debug!("Removed {} instances of the value {} surrounding solved cell at {:?}", count, value, cell.coord());
        count
    }
}

impl Constraint for VectorSolvedCellConstraint {
    fn notify_changes(&mut self, _: &Puzzle, changes: &PuzzleMarkupChanges) {
        for &(index, _) in &changes.cell_solutions {
            self.solved_cells.push(index);
        }
    }

    fn enforce_partial(&mut self, puzzle: &Puzzle, markup: &PuzzleMarkup, changes: &mut PuzzleMarkupChanges) -> bool {
        while let Some(index) = self.solved_cells.pop() {
            let cell = puzzle.cell(index);
            let value = markup.cells()[index].unwrap_solved();
            let count = Self::enforce_solved_cell(puzzle, &markup.cells(), cell, value, changes);
            if count > 0 { return true }
        }
        false
    }
}
