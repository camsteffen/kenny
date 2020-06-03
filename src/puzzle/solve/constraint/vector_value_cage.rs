//! If all possible solutions for a given value in a given vector are in a given cage, then the cage solution must
//! contain the given value in the given vector

use crate::collections::square::{IsSquare, Vector};
use crate::collections::LinkedAHashSet;
use crate::puzzle::solve::constraint::Constraint;
use crate::puzzle::solve::markup::{CellChange, PuzzleMarkup, PuzzleMarkupChanges};
use crate::puzzle::{Puzzle, Value};
use itertools::Itertools;

/// If a value is known to be in a cage-vector, cage solutions must include the value in the vector.
#[derive(Clone)]
pub struct VectorValueCageConstraint {
    dirty_vector_values: LinkedAHashSet<(Vector, Value)>,
}

impl VectorValueCageConstraint {
    pub fn new(puzzle: &Puzzle) -> Self {
        let dirty_vector_values = puzzle
            .vectors()
            .flat_map(|v| (1..=puzzle.width() as i32).map(move |i| (v, i)))
            .collect();
        Self {
            dirty_vector_values,
        }
    }
}

impl Constraint for VectorValueCageConstraint {
    fn notify_changes(&mut self, puzzle: &Puzzle, changes: &PuzzleMarkupChanges) {
        for (&id, change) in changes.cells.iter() {
            let cell = puzzle.cell(id);
            match change {
                CellChange::DomainRemovals(values) => {
                    self.dirty_vector_values.extend(
                        cell.vectors()
                            .iter()
                            .flat_map(|&vector| values.iter().map(move |&value| (vector, value))),
                    );
                }
                &CellChange::Solution(value) => {
                    for vector in cell.vectors().iter().copied() {
                        self.dirty_vector_values.remove(&(vector, value));
                    }
                }
            }
        }
    }

    fn enforce_partial(
        &mut self,
        puzzle: &Puzzle,
        markup: &PuzzleMarkup,
        changes: &mut PuzzleMarkupChanges,
    ) -> bool {
        while let Some((vector, value)) = self.dirty_vector_values.pop_front() {
            let count = enforce_vector_value(vector, value, puzzle, markup, changes);
            if count > 0 {
                return true;
            }
        }
        false
    }
}

fn enforce_vector_value(
    vector: Vector,
    value: Value,
    puzzle: &Puzzle,
    markup: &PuzzleMarkup,
    changes: &mut PuzzleMarkupChanges,
) -> u32 {
    let solved = puzzle
        .vector(vector)
        .indices()
        .any(|i| markup.cells()[i].solved() == Some(value));
    if solved {
        return 0;
    }

    // todo only consider cages with multiple vectors to avoid redundancy with preemptive sets? If yes, use `take_while`.
    // cage containing all unsolved cells in the vector with the value in its domain
    let cage = puzzle
        .vector(vector)
        .indices()
        .filter(|&i| markup.cells()[i].unsolved_and_contains(value))
        .map(|i| puzzle.cell(i).cage_id())
        .dedup()
        .exactly_one();
    let cage = match cage {
        Ok(cage) => cage,
        Err(_) => return 0,
    };

    let view = markup.cage_solutions().unwrap()[cage].vector_view(puzzle.vector(vector));
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
        debug!("Removed {} cage solutions for cage at {:?} where cage does not have {} in the vector {:?}",
               count, puzzle.cage(cage).coord(), value, vector)
    }
    count
}
