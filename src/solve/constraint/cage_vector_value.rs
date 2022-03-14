use std::convert::TryInto;

use super::Constraint;
use crate::collections::square::IsSquare;
use crate::collections::square::{Square, Vector};
use crate::puzzle::{CageId, CellId, CellRef, Puzzle, Value};
use crate::solve::markup::{PuzzleMarkup, PuzzleMarkupChanges};
use crate::solve::CellVariable;
use crate::{HashMap, HashSet, LinkedHashSet};

/// If a value exists in every cage solution for a cage, in a given vector, that value must be in that cage-vector.
/// It must not be in any other cell in the vector, not in that cage.
#[derive(Clone)]
pub(crate) struct CageVectorValueConstraint<'a> {
    puzzle: &'a Puzzle,
    /// A map of vectors per cell where there are multiple cells in the cage and vector.
    /// This is used to determine which cage-vector's to check after puzzle markup changes.
    cell_cage_vectors: Square<Vec<Vector>>,
    /// Cage-vectors to be checked due to puzzle markup changes
    dirty_cage_vectors: LinkedHashSet<(CageId, Vector)>,
    /// A record of values known to be in a certain cage, in a certain vector
    /// This is used to avoid duplicate work
    known_vector_vals: HashMap<Vector, HashSet<Value>>,
}

impl<'a> CageVectorValueConstraint<'a> {
    pub fn new(puzzle: &'a Puzzle) -> Self {
        Self {
            puzzle,
            cell_cage_vectors: create_cell_cage_vector_map(puzzle),
            dirty_cage_vectors: LinkedHashSet::default(),
            known_vector_vals: HashMap::default(),
        }
    }
}

impl<'a> Constraint for CageVectorValueConstraint<'a> {
    fn notify_changes(
        &mut self,
        changes: &PuzzleMarkupChanges,
        _cell_variables: &Square<CellVariable>,
    ) {
        for (id, _) in changes.cells.domain_removals() {
            self.notify_change_cell_domain(id);
        }
        for &cage_id in changes.cage_solution_removals.keys() {
            for &cell_id in self.puzzle.cage(cage_id).cell_ids() {
                self.notify_change_cell_domain(cell_id);
            }
        }
    }

    fn enforce_partial(
        &mut self,
        markup: &PuzzleMarkup<'_>,
        changes: &mut PuzzleMarkupChanges,
    ) -> bool {
        while let Some((cage_id, vector)) = self.dirty_cage_vectors.pop_front() {
            let count = self.enforce_cage_vector(markup, changes, cage_id, vector);
            if count > 0 {
                return true;
            }
        }
        false
    }
}

impl CageVectorValueConstraint<'_> {
    fn notify_change_cell_domain(&mut self, cell_id: CellId) {
        let cage_id = self.puzzle.cell(cell_id).cage_id();
        for &vector in &self.cell_cage_vectors[cell_id] {
            self.dirty_cage_vectors.insert((cage_id, vector));
        }
    }

    pub fn enforce_cage_vector(
        &mut self,
        markup: &PuzzleMarkup<'_>,
        change: &mut PuzzleMarkupChanges,
        cage_id: CageId,
        vector: Vector,
    ) -> u32 {
        let values = self.find_cage_vector_values(markup, cage_id, vector);

        if values.is_empty() {
            return 0;
        }

        debug!(
            "values {:?} exists in cage at {:?}, in {:?}",
            values,
            self.puzzle.cage(cage_id).cell(0).coord(),
            vector
        );

        // record known vector values
        self.known_vector_vals
            .entry(vector)
            .or_default()
            .extend(&values);

        // cells that are in the vector but not in the cage
        let remove_from = self
            .puzzle
            .vector(vector)
            .iter()
            .filter(|cell| cell.cage_id() != cage_id)
            .map(CellRef::id)
            .collect::<Vec<_>>();

        let mut count = 0;

        // mark domain values for removal
        for n in values {
            for &pos in &remove_from {
                if markup.cells()[pos].unsolved_and_contains(n) {
                    change.cells.remove_domain_value(pos, n);
                    count += 1;
                }
            }
        }
        count
    }

    /// find values that exist in every cage solution in the vector
    fn find_cage_vector_values(
        &self,
        markup: &PuzzleMarkup<'_>,
        cage_id: CageId,
        vector: Vector,
    ) -> HashSet<i32> {
        // indices within each solution where the cell is in the vector
        let solution_indices: Vec<usize> = markup.cage_solutions().unwrap()[cage_id]
            .cell_ids
            .iter()
            .copied()
            .enumerate()
            .filter(|&(_, cell_id)| self.puzzle.cell(cell_id).is_in_vector(vector))
            .map(|(i, _)| i)
            .collect();
        if solution_indices.len() < 2 {
            return HashSet::default();
        }

        // iterator of solutions with only cells in the vector
        let mut solutions_iter = markup.cage_solutions().unwrap()[cage_id]
            .solutions
            .iter()
            .map(|solution| solution_indices.iter().map(move |&i| solution[i]));
        let solution = solutions_iter.next().unwrap();

        // values in the first solution that are not already a known vector value
        let mut values: HashSet<i32> = solution
            .filter(|n| {
                self.known_vector_vals
                    .get(&vector)
                    .map_or(true, |values| !values.contains(n))
            })
            .collect();

        if values.is_empty() {
            return values;
        }

        for solution in solutions_iter {
            // remove values that are not in the current solution
            values = solution.filter(|n| values.contains(n)).collect();

            if values.is_empty() {
                break;
            }
        }

        values
    }
}

fn create_cell_cage_vector_map(puzzle: &Puzzle) -> Square<Vec<Vector>> {
    puzzle
        .cells()
        .map(|cell| {
            cell.vectors()
                .iter()
                .copied()
                // include vector if there are other cells in the same cage in the same vector
                .filter(|&vector| {
                    cell.cage().cells().any(|cage_cell| {
                        cage_cell.id() != cell.id() && cage_cell.is_in_vector(vector)
                    })
                })
                .collect()
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}
