use crate::collections::{RangeSet, LinkedAHashSet};
use std::ops::{Index, IndexMut};
use crate::collections::square::{VectorId, IsSquare};
use crate::puzzle::solve::markup::PuzzleMarkupChanges;
use crate::puzzle::{Puzzle, CellRef};
use crate::puzzle::solve::PuzzleMarkup;
use super::Constraint;

/// Constraint: For each vector V, for each value N, only one instance of N may occur.
pub struct VectorValueDomainConstraint {
    data: VectorValueSet,
    dirty_vec_vals: LinkedAHashSet<(VectorId, i32)>,
}

impl VectorValueDomainConstraint {
    pub fn new(puzzle_width: usize) -> Self {
        Self {
            data: VectorValueSet::new(puzzle_width),
            dirty_vec_vals: Default::default(),
        }
    }

    fn enforce_vector_value(&mut self, puzzle: &Puzzle, vector_id: VectorId, n: i32, change: &mut PuzzleMarkupChanges) -> bool {
        let vec_val_pos = match self.data[vector_id][n as usize - 1].as_ref().and_then(RangeSet::single_value) {
            Some(v) => v,
            None => return false,
        };
        let sq_pos = puzzle.vector_point(vector_id, vec_val_pos);
        debug!("the only possible position for {} in {:?} is {:?}", n, vector_id, puzzle.coord_at(sq_pos));
        let v2 = vector_id.intersect_at(vec_val_pos);
        self.data.remove_vector_value(vector_id, n);
        self.data.remove_vector_value(v2, n);
        change.solve_cell(sq_pos, n);
        true
    }

}

impl Constraint for VectorValueDomainConstraint {
    fn enforce_partial(&mut self, puzzle: &Puzzle, _: &PuzzleMarkup, changes: &mut PuzzleMarkupChanges) -> bool {
        while let Some((vector_id, value)) = self.dirty_vec_vals.pop_front() {
            let solved = self.enforce_vector_value(puzzle, vector_id, value, changes);
            if solved { return true }
        }
        false
    }

    fn notify_changes(&mut self, puzzle: &Puzzle, changes: &PuzzleMarkupChanges) {
        for &(index, value) in &changes.cell_solutions {
            self.data.remove_cell_value(puzzle.cell(index), value);
        }
        for (&index, values) in &changes.cell_domain_value_removals {
            for v in puzzle.cell(index).vectors().iter().copied() {
                let vector_data = &mut self.data[v];
                for &value in values {
                    if let Some(dom) = vector_data[value as usize - 1].as_mut() {
                        let vec_pos = puzzle.index_to_vector_point(index, v);
                        let removed = dom.remove(vec_pos);
                        debug_assert!(removed);
                        self.dirty_vec_vals.insert((v, value));
                    };
                }
            }
        }
    }
}

/// VectorId -> value -> positions
struct VectorValueSet(Vec<Vec<Option<RangeSet>>>);

impl VectorValueSet {
    pub fn new(puzzle_width: usize) -> VectorValueSet {
        let width = puzzle_width;
        VectorValueSet(vec![vec![Some(RangeSet::with_all(width)); width]; 2 * width])
    }

    pub fn remove_cell_value(&mut self, cell: CellRef, value: i32) {
        for vector_id in cell.vectors().iter().copied() {
            self.remove_vector_value(vector_id, value);
        }
    }

    pub fn remove_vector_value(&mut self, vector_id: VectorId, value: i32) {
        self[vector_id][value as usize - 1] = None;
    }
}

impl Index<VectorId> for VectorValueSet {
    type Output = Vec<Option<RangeSet>>;

    fn index(&self, vector_id: VectorId) -> &Self::Output {
        &self.0[usize::from(vector_id)]
    }
}

impl IndexMut<VectorId> for VectorValueSet {
    fn index_mut(&mut self, vector_id: VectorId) -> &mut Self::Output {
        &mut self.0[usize::from(vector_id)]
    }
}
