use crate::collections::{RangeSet, LinkedAHashSet};
use std::ops::{Index, IndexMut};
use crate::collections::square::{VectorId, IsSquare};
use crate::puzzle::solve::markup::PuzzleMarkupChanges;
use crate::puzzle::{Puzzle, CellRef, Value};
use crate::puzzle::solve::PuzzleMarkup;
use super::Constraint;

/// If only one cell in a vector has a given value in its domain, then the cell has that value.
#[derive(Clone)]
pub struct VectorValueDomainConstraint {
    data: VectorValueIndexSet,
    dirty_vec_vals: LinkedAHashSet<(VectorId, i32)>,
}

impl VectorValueDomainConstraint {
    pub fn new(puzzle_width: usize) -> Self {
        Self {
            data: VectorValueIndexSet::new(puzzle_width),
            dirty_vec_vals: Default::default(),
        }
    }

    fn enforce_vector_value(
        &mut self,
        puzzle: &Puzzle,
        vector_id: VectorId,
        n: i32,
        change: &mut PuzzleMarkupChanges
    ) -> bool {
        let vec_val_pos = match self.data[vector_id][n as usize - 1].as_ref().and_then(RangeSet::single_value) {
            Some(v) => v,
            None => return false,
        };
        let sq_pos = puzzle.vector_point(vector_id, vec_val_pos);
        debug!("the only possible position for {} in {:?} is {:?}", n, vector_id, puzzle.coord_at(sq_pos));
        change.solve_cell(sq_pos, n);
        self.data.remove_cell_value(puzzle.cell(sq_pos), n);
        true
    }

}

impl Constraint for VectorValueDomainConstraint {
    fn notify_changes(&mut self, puzzle: &Puzzle, changes: &PuzzleMarkupChanges) {
        for &(index, value) in &changes.cell_solutions {
            self.data.remove_cell_value(puzzle.cell(index), value);
        }
        for (&index, values) in &changes.cell_domain_value_removals {
            for &vector in &puzzle.cell(index).vectors() {
                let vector_data = &mut self.data[vector];
                for &value in values {
                    if let Some(dom) = vector_data[value as usize - 1].as_mut() {
                        let vec_pos = puzzle.index_to_vector_point(index, vector);
                        if dom.remove(vec_pos) {
                            self.dirty_vec_vals.insert((vector, value));
                        }
                    };
                }
            }
        }
    }

    fn enforce_partial(&mut self, puzzle: &Puzzle, _: &PuzzleMarkup, changes: &mut PuzzleMarkupChanges) -> bool {
        while let Some((vector_id, value)) = self.dirty_vec_vals.pop_front() {
            let solved = self.enforce_vector_value(puzzle, vector_id, value, changes);
            if solved { return true }
        }
        false
    }
}

/// VectorId -> value -> vector indices (where the value could be)
#[derive(Clone)]
struct VectorValueIndexSet(Vec<Vec<Option<RangeSet>>>);

impl VectorValueIndexSet {
    pub fn new(puzzle_width: usize) -> VectorValueIndexSet {
        let width = puzzle_width;
        VectorValueIndexSet(vec![vec![Some(RangeSet::with_all(width)); width]; 2 * width])
    }

    pub fn remove_cell_value(&mut self, cell: CellRef<'_>, value: Value) {
        for &vector_id in &cell.vectors() {
            self.remove_vector_value(vector_id, value);
        }
    }

    pub fn remove_vector_value(&mut self, vector_id: VectorId, value: Value) {
        self[vector_id][value as usize - 1] = None;
    }
}

impl Index<VectorId> for VectorValueIndexSet {
    type Output = Vec<Option<RangeSet>>;

    fn index(&self, vector_id: VectorId) -> &Self::Output {
        &self.0[usize::from(vector_id)]
    }
}

impl IndexMut<VectorId> for VectorValueIndexSet {
    fn index_mut(&mut self, vector_id: VectorId) -> &mut Self::Output {
        &mut self.0[usize::from(vector_id)]
    }
}
