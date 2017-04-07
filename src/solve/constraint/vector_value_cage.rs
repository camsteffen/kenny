//! If all possible solutions for a given value in a given vector are in a given cage, then the cage solution must
//! contain the given value in the given vector

use itertools::Itertools;

use crate::collections::square::{IsSquare, Square, Vector};
use crate::collections::LinkedAHashSet;
use crate::puzzle::{CellRef, Puzzle, Value};
use crate::solve::constraint::Constraint;
use crate::solve::markup::{CellChange, PuzzleMarkup, PuzzleMarkupChanges};
use crate::solve::CellVariable;

/// If a value is known to be in a cage-vector, cage solutions must include the value in the vector.
#[derive(Clone)]
pub(crate) struct VectorValueCageConstraint<'a> {
    puzzle: &'a Puzzle,
    dirty_vector_values: LinkedAHashSet<(Vector, Value)>,
}

impl<'a> VectorValueCageConstraint<'a> {
    pub fn new(puzzle: &'a Puzzle) -> Self {
        let dirty_vector_values = puzzle
            .vectors()
            .flat_map(|v| (1..=puzzle.width() as i32).map(move |i| (v, i)))
            .collect();
        Self {
            puzzle,
            dirty_vector_values,
        }
    }
}

impl<'a> Constraint for VectorValueCageConstraint<'a> {
    fn notify_changes(
        &mut self,
        changes: &PuzzleMarkupChanges,
        cell_variables: &Square<CellVariable>,
    ) {
        for (&cell_id, change) in &changes.cells {
            let cell = self.puzzle.cell(cell_id);
            match change {
                CellChange::DomainRemovals(values) => {
                    self.dirty_vector_values.extend(
                        cell.vectors()
                            .iter()
                            .flat_map(|&vector| values.iter().map(move |&value| (vector, value))),
                    );
                }
                &CellChange::Solution(value) => {
                    let vectors = cell.vectors();
                    for &vector in &vectors {
                        self.dirty_vector_values.remove(&(vector, value));
                    }
                    let domain = cell_variables[cell_id].unsolved().unwrap();
                    self.dirty_vector_values.extend(
                        domain
                            .iter()
                            .filter(|&v| v != value)
                            .flat_map(|v| vectors.iter().map(move |&vector| (vector, v))),
                    );
                }
            }
        }
    }

    fn enforce_partial(
        &mut self,
        markup: &PuzzleMarkup<'_>,
        changes: &mut PuzzleMarkupChanges,
    ) -> bool {
        while let Some((vector, value)) = self.dirty_vector_values.pop_front() {
            let count = self.enforce_vector_value(vector, value, markup, changes);
            if count > 0 {
                return true;
            }
        }
        false
    }
}

impl VectorValueCageConstraint<'_> {
    fn enforce_vector_value(
        &self,
        vector: Vector,
        value: Value,
        markup: &PuzzleMarkup<'_>,
        changes: &mut PuzzleMarkupChanges,
    ) -> u32 {
        let solved = self
            .puzzle
            .vector(vector)
            .indices()
            .any(|i| markup.cells()[i].solved() == Some(value));
        if solved {
            return 0;
        }

        // cage containing all unsolved cells in the vector with the value in its domain
        let cage = self
            .puzzle
            .vector(vector)
            .iter()
            .filter(|&cell| markup.cells()[cell.id()].unsolved_and_contains(value))
            .map(CellRef::cage_id)
            .dedup()
            .exactly_one();
        let cage = match cage {
            Ok(cage) => cage,
            Err(_) => return 0,
        };

        let view = markup.cage_solutions().unwrap()[cage].vector_view(self.puzzle.vector(vector));
        debug_assert!(!view.is_empty());

        // find and remove solutions that do not include the value in the vector
        let mut count = 0;
        for (soln_idx, solution) in view.solutions().enumerate() {
            if solution.iter().all(|&v| v != value) {
                changes.remove_cage_solution(cage, soln_idx);
                count += 1;
            }
        }
        if count > 0 {
            debug!(
                "Removed {} cage solutions for cage at {:?} where cage does not have {} in {:?}",
                count,
                self.puzzle.cage(cage).coord(),
                value,
                vector
            )
        }
        count
    }
}
