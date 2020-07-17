use std::hash::BuildHasherDefault;

use ahash::AHasher;

use super::Constraint;
use crate::collections::iterator_ext::IteratorExt;
use crate::collections::square::{IsSquare, Square, Vector};
use crate::collections::LinkedAHashSet;
use crate::puzzle::solve::markup::{CellChange, PuzzleMarkup, PuzzleMarkupChanges};
use crate::puzzle::solve::CellVariable;
use crate::puzzle::{CageId, CellId, Puzzle, Value};
use crate::HashSet;

/// Summary: A cage solution must not conflict with a cell's domain outside of the cage
///
/// Given:
/// * A vector (V)
/// * A cage (G) having a potential solution (S), only including cells in V
/// * A cell (C) with domain (D); C is in V but not in G
///
/// Constraint: S must be a proper subset of D
///
/// Unsatisfied action: Remove S from possible cage solutions
#[derive(Clone)]
pub(crate) struct CageSolutionOuterCellDomainConstraint<'a> {
    puzzle: &'a Puzzle,
    dirty_cells: LinkedAHashSet<CellId>,
}

impl<'a> CageSolutionOuterCellDomainConstraint<'a> {
    pub fn new(puzzle: &'a Puzzle) -> Self {
        Self {
            puzzle,
            dirty_cells: LinkedAHashSet::default(),
        }
    }
}

impl<'a> Constraint for CageSolutionOuterCellDomainConstraint<'a> {
    fn notify_changes(
        &mut self,
        changes: &PuzzleMarkupChanges,
        _cell_variables: &Square<CellVariable>,
    ) {
        for (&id, change) in &changes.cells {
            match change {
                CellChange::DomainRemovals(_) => {
                    self.dirty_cells.insert(id);
                }
                CellChange::Solution(_) => {
                    self.dirty_cells.remove(&id);
                }
            }
        }
    }

    fn enforce_partial(
        &mut self,
        markup: &PuzzleMarkup<'_>,
        changes: &mut PuzzleMarkupChanges,
    ) -> bool {
        while let Some(cell_id) = self.dirty_cells.pop_front() {
            let count = self.enforce_cell(cell_id, markup, changes);
            if count > 0 {
                return true;
            }
        }
        false
    }
}

impl CageSolutionOuterCellDomainConstraint<'_> {
    fn enforce_cell(
        &self,
        cell_id: CellId,
        markup: &PuzzleMarkup<'_>,
        changes: &mut PuzzleMarkupChanges,
    ) -> u32 {
        if markup.cells()[cell_id].is_solved() {
            return 0;
        }
        let mut count = 0;
        let cell = self.puzzle.cell(cell_id);
        for &vector in &cell.vectors() {
            let cage_ids = self
                .puzzle
                .vector(vector)
                .iter()
                .map(|cell| cell.cage_id())
                .filter(|&cage_id| cage_id != cell.cage_id())
                .unique_default::<BuildHasherDefault<AHasher>>();
            for cage_id in cage_ids {
                count += self.enforce_cell_cage_vector(cell_id, cage_id, vector, markup, changes);
            }
        }
        count
    }

    fn enforce_cell_cage_vector(
        &self,
        cell_id: CellId,
        cage_id: CageId,
        vector: Vector,
        markup: &PuzzleMarkup<'_>,
        changes: &mut PuzzleMarkupChanges,
    ) -> u32 {
        let cage_solutions = &markup.cage_solutions().unwrap()[cage_id];
        if cage_solutions.cell_ids.is_empty() {
            // cage is solved
            return 0;
        }
        let cage_solutions_view = cage_solutions.vector_view(self.puzzle.vector(vector));
        let domain = markup.cells()[cell_id].unsolved().unwrap();
        if cage_solutions_view.len() < domain.len() {
            return 0;
        }

        let cell = self.puzzle.cell(cell_id);
        let cage = self.puzzle.cage(cage_id);
        let mut count = 0;
        for (solution_index, solution) in cage_solutions_view.solutions().enumerate() {
            // solution values for cells in cage and vector
            let solution_values: HashSet<Value> = solution.iter().copied().collect();
            if domain.iter().all(|value| solution_values.contains(&value)) {
                debug!(
                    "solution {:?} for cage at {:?} conflicts with cell domain at {:?}",
                    solution,
                    cage.coord(),
                    cell.coord()
                );
                changes.remove_cage_solution(cage.id(), solution_index);
                count += 1;
                break;
            }
        }
        count
    }
}
