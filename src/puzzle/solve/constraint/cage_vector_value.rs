use std::convert::TryInto;

use ahash::{AHashMap, AHashSet};

use crate::collections::{LinkedAHashSet, Square};
use crate::collections::square::VectorId;
use crate::puzzle::{CageId, CellRef, Puzzle, Value};
use crate::puzzle::solve::markup::PuzzleMarkupChanges;
use crate::puzzle::solve::PuzzleMarkup;

use super::Constraint;

/// If a value exists in every cage solution for a cage, in a given vector, that value must be in that cage-vector.
/// It must not be in any other cell in the vector, not in that cage.
#[derive(Clone)]
pub struct CageVectorValueConstraint {
    /// A map of vectors per cell where there are multiple cells in the cage and vector.
    /// This is used to determine which cage-vector's to check after puzzle markup changes.
    cell_cage_vectors: Square<Vec<VectorId>>,
    /// Cage-vectors to be checked due to puzzle markup changes
    dirty_cage_vectors: LinkedAHashSet<(CageId, VectorId)>,
    /// A record of values known to be in a certain cage, in a certain vector
    /// This is used to avoid duplicate work
    known_vector_vals: AHashMap<VectorId, AHashSet<Value>>,
}

impl CageVectorValueConstraint {
    pub fn new(puzzle: &Puzzle) -> Self {
        Self {
            cell_cage_vectors: create_cell_cage_vector_map(puzzle),
            dirty_cage_vectors: Default::default(),
            known_vector_vals: Default::default(),
        }
    }

    pub fn enforce_cage_vector(
        &mut self,
        puzzle: &Puzzle,
        markup: &PuzzleMarkup,
        change: &mut PuzzleMarkupChanges,
        cage_id: CageId,
        vector_id: VectorId,
    ) -> u32 {
        let values = self.find_cage_vector_values(puzzle, markup, cage_id, vector_id);

        if values.is_empty() { return 0; }

        debug!("values {:?} exists in cage at {:?}, in {:?}", values,
               puzzle.cage(cage_id).cell(0).coord(), vector_id);

        // record known vector values
        self.known_vector_vals.entry(vector_id)
            .or_insert_with(Default::default)
            .extend(&values);

        // cells that are in the vector but not in the cage
        let remove_from = puzzle.vector_cells(vector_id)
            .filter(|cell| cell.cage_id() != cage_id)
            .map(CellRef::id)
            .collect::<Vec<_>>();

        let mut count = 0;

        // mark domain values for removal
        for n in values {
            for &pos in &remove_from {
                if markup.cells()[pos].unsolved_and_contains(n) {
                    change.remove_value_from_cell(pos, n);
                    count += 1;
                }
            }
        }
        count
    }

    /// find values that exist in every cage solution in the vector
    fn find_cage_vector_values(
        &self,
        puzzle: &Puzzle,
        markup: &PuzzleMarkup,
        cage_id: CageId,
        vector_id: VectorId,
    ) -> AHashSet<i32> {
        // indices within each solution where the cell is in the vector
        let solution_indices: Vec<usize> = markup.cage_solutions()[cage_id].cell_ids.iter().copied().enumerate()
            .filter(|&(_, cell_id)| puzzle.cell(cell_id).is_in_vector(vector_id))
            .map(|(i, _)| i)
            .collect();
        if solution_indices.len() < 2 { return AHashSet::new(); }

        // iterator of solutions with only cells in the vector
        let mut solutions_iter = markup.cage_solutions()[cage_id].solutions.iter()
            .map(|solution| solution_indices.iter()
                .map(move |&i| solution[i]));
        let solution = solutions_iter.next().unwrap_or_else(|| todo!());

        // values in the first solution that are not already a known vector value
        let mut values: AHashSet<i32> = solution
            .filter(|n| {
                self.known_vector_vals.get(&vector_id)
                    .map_or(true, |values| !values.contains(n))
            })
            .collect();

        if values.is_empty() { return values; }

        for solution in solutions_iter {
            // remove values that are not in the current solution
            values = solution
                .filter(|n| values.contains(n))
                .collect();

            if values.is_empty() { break; }
        }

        values
    }

    fn notify_change_cell_domain(&mut self, cell: CellRef<'_>) {
        for vector_id in &self.cell_cage_vectors[usize::from(cell.id())] {
            self.dirty_cage_vectors.insert((cell.cage_id(), *vector_id));
        }
    }
}

impl Constraint for CageVectorValueConstraint {
    fn notify_changes(&mut self, puzzle: &Puzzle, changes: &PuzzleMarkupChanges) {
        for &id in changes.cell_domain_value_removals.keys() {
            self.notify_change_cell_domain(puzzle.cell(id));
        }
        for &cage_id in changes.cage_solution_removals.keys() {
            for cell in puzzle.cage(cage_id).cells() {
                self.notify_change_cell_domain(cell);
            }
        }
    }

    fn enforce_partial(&mut self, puzzle: &Puzzle, markup: &PuzzleMarkup, changes: &mut PuzzleMarkupChanges) -> bool {
        while let Some((cage_id, vector_id)) = self.dirty_cage_vectors.pop_front() {
            let count = self.enforce_cage_vector(puzzle, markup, changes, cage_id, vector_id);
            if count > 0 { return true; }
        }
        false
    }
}

fn create_cell_cage_vector_map(puzzle: &Puzzle) -> Square<Vec<VectorId>> {
    puzzle.cells()
        .map(|cell| {
            cell.vectors().iter().copied()
                // include vector if there are other cells in the same cage in the same vector
                .filter(|&vector| cell.cage().cells().any(|cage_cell|
                    cage_cell.id() != cell.id()
                        && cage_cell.is_in_vector(vector)))
                .collect()
        })
        .collect::<Vec<_>>()
        .try_into().unwrap()
}
