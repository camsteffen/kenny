//! If all possible solutions for a given value in a given vector are in a given cage, then the cage solution must
//! contain the given value in the given vector

use crate::collections::square::{IsSquare, VectorId};
use crate::collections::LinkedAHashSet;
use crate::puzzle::solve::cage_solutions::CageSolutions;
use crate::puzzle::solve::constraint::Constraint;
use crate::puzzle::solve::markup::PuzzleMarkup;
use crate::puzzle::solve::markup::PuzzleMarkupChanges;
use crate::puzzle::{Puzzle, Value};
use itertools::Itertools;

/// If a value is known to be in a cage-vector it must not be in other cells in the vector
#[derive(Clone)]
pub struct VectorValueCageConstraint {
    dirty_vector_values: LinkedAHashSet<(VectorId, Value)>,
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
        for (&i, values) in &changes.cell_domain_value_removals {
            let cell = puzzle.cell(i);
            for vector_id in cell.vectors().iter().copied() {
                for &value in values {
                    self.dirty_vector_values.insert((vector_id, value));
                }
            }
        }

        for &(sq_idx, value) in &changes.cell_solutions {
            let cell = puzzle.cell(sq_idx);
            for vector_id in cell.vectors().iter().copied() {
                self.dirty_vector_values.remove(&(vector_id, value));
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
    vector: VectorId,
    value: Value,
    puzzle: &Puzzle,
    markup: &PuzzleMarkup,
    changes: &mut PuzzleMarkupChanges,
) -> u32 {
    if puzzle
        .vector_indices(vector)
        .any(|i| markup.cells()[i].solved() == Some(value))
    {
        return 0;
    }

    // cage containing all unsolved cells in the vector with the value in its domain
    let cage = puzzle
        .vector_indices(vector)
        .filter(|&i| markup.cells()[i].unsolved_and_contains(value))
        .map(|i| puzzle.cell(i).cage_id())
        .dedup()
        .exactly_one();
    let cage = match cage {
        Ok(cage) => cage,
        Err(_) => return 0,
    };

    let CageSolutions {
        cell_ids,
        solutions,
        ..
    } = &markup.cage_solutions()[cage];

    // indices of cage solution cell_ids where the cell is in the vector
    let indices_in_vector: Vec<_> = cell_ids
        .iter()
        .copied()
        .enumerate()
        .filter(|&(_, id)| puzzle.cell(id).is_in_vector(vector))
        .map(|(i, _)| i)
        .collect();
    debug_assert!(!indices_in_vector.is_empty());

    // find and remove solutions that do not include the value in the vector
    let mut count = 0;
    for (soln_idx, solution) in solutions.iter().enumerate() {
        if indices_in_vector.iter().all(|&i| solution[i] != value) {
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
