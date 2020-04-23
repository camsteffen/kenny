use crate::collections::square::VectorId;
use crate::puzzle::solve::markup::PuzzleMarkupChanges;
use crate::puzzle::{Puzzle, CellRef};
use super::Constraint;
use crate::puzzle::solve::PuzzleMarkup;
use crate::collections::{Square, LinkedAHashSet};
use crate::puzzle::solve::constraint::State;
use std::convert::TryInto;
use ahash::{AHashMap, AHashSet};

/// A record of values known to be in a certain cage, in a certain vector
pub struct CageVectorValueConstraint {
    cell_cage_vector_map: Square<Vec<VectorId>>,
    dirty_cage_vectors: LinkedAHashSet<(usize, VectorId)>,
    known_vector_vals: AHashMap<VectorId, AHashSet<i32>>,
}

impl CageVectorValueConstraint {
    pub fn new(puzzle: &Puzzle) -> Self {
        Self {
            cell_cage_vector_map: create_cell_cage_vector_map(puzzle),
            dirty_cage_vectors: Default::default(),
            known_vector_vals: Default::default(),
        }
    }

    pub fn enforce_cage_vector(&mut self, puzzle: &Puzzle, markup: &PuzzleMarkup, change: &mut PuzzleMarkupChanges, cage_index: usize, vector_id: VectorId) -> u32 {
        let solution_indices = markup.cage_solutions()[cage_index].indices.iter().copied().enumerate()
            .filter_map(|(i, sq_idx)| if puzzle.cell(sq_idx).intersects_vector(vector_id) { Some(i) } else { None })
            .collect::<Vec<_>>();
        if solution_indices.len() < 2 { return 0; }

        let mut solutions_iter = markup.cage_solutions()[cage_index].solutions.iter()
            .map(|s| solution_indices.iter().map(|&i| s[i]).collect::<Vec<_>>().into_iter());
        let solution = solutions_iter.next().unwrap();

        // start with values in the first solution that are not already a known vector value
        let mut values: AHashSet<i32> = solution
            .filter(|n| {
                self.known_vector_vals.get(&vector_id)
                    .map_or(true, |vals| !vals.contains(n))
            })
            .collect();

        for solution in solutions_iter {
            if values.is_empty() { break; }

            // remove values that are not in the current solution
            values = solution
                .filter(|n| values.contains(n))
                .collect();
        }

        if values.is_empty() { return 0; }

        self.known_vector_vals.entry(vector_id)
            .or_insert_with(Default::default)
            .extend(&values);

        // find cells that are in this vector but not part of this cage
        let remove_from = puzzle.vector_cells(vector_id)
            .filter(|cell| cell.cage().index() != cage_index)
            .map(CellRef::index)
            .collect::<Vec<_>>();

        let mut count = 0;

        // mark domain values for removal
        debug!("values {:?} exists in cage at {:?}, in {:?}", values,
               puzzle.cage(cage_index).cell(0).coord(), vector_id);
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

    fn notify_change_cell_domain(&mut self, cell: CellRef) {
        for vector_id in &self.cell_cage_vector_map[usize::from(cell.index())] {
            self.dirty_cage_vectors.insert((cell.cage().index(), *vector_id));
        }
    }
}

impl Constraint for CageVectorValueConstraint {
    fn enforce_partial(&mut self, puzzle: &Puzzle, markup: &PuzzleMarkup, changes: &mut PuzzleMarkupChanges) -> bool {
        while let Some((cage_index, vector_id)) = self.dirty_cage_vectors.pop_front() {
            let count = self.enforce_cage_vector(puzzle, markup, changes, cage_index, vector_id);
            if count > 0 { return true; }
        }
        false
    }

    fn notify_changes(&mut self, puzzle: &Puzzle, changes: &PuzzleMarkupChanges) {
        for &index in changes.cell_domain_value_removals.keys() {
            self.notify_change_cell_domain(puzzle.cell(index));
        }
        for &cage_index in changes.cage_solution_removals.keys() {
            for cell in puzzle.cage(cage_index).cells() {
                self.notify_change_cell_domain(cell);
            }
        }
    }

    fn state(&self) -> State {
        unimplemented!()
    }
}

fn create_cell_cage_vector_map(puzzle: &Puzzle) -> Square<Vec<VectorId>> {
    puzzle.cells()
        .map(|cell| {
            cell.vectors().iter().copied()
                // find other cells in the same cage on the same vector
                .filter(|&vector| cell.cage().cells().any(|cage_cell|
                    cage_cell.index() != cell.index()
                        && cage_cell.intersects_vector(vector)))
                .collect()
        })
        .collect::<Vec<_>>()
        .try_into().unwrap()
}
