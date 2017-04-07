use std::ops::{Index, IndexMut};

use super::Constraint;
use crate::collections::range_set::RangeSet;
use crate::collections::square::{AsVector, EmptySquare, IsSquare, Square, Vector};
use crate::collections::LinkedAHashSet;
use crate::puzzle::{CellId, Puzzle, Value};
use crate::solve::markup::{CellChange, PuzzleMarkup, PuzzleMarkupChanges};
use crate::solve::CellVariable;

/// If only one cell in a vector has a given value in its domain, then the cell has that value.
#[derive(Clone)]
pub(crate) struct VectorValueDomainConstraint<'a> {
    puzzle: &'a Puzzle,
    data: VectorValueIndexSet,
    dirty_vec_vals: LinkedAHashSet<(Vector, i32)>,
}

impl<'a> VectorValueDomainConstraint<'a> {
    pub fn new(puzzle: &'a Puzzle) -> Self {
        Self {
            puzzle,
            data: VectorValueIndexSet::new(puzzle.width()),
            dirty_vec_vals: LinkedAHashSet::default(),
        }
    }

    fn enforce_vector_value(
        &mut self,
        puzzle: &Puzzle,
        vector: Vector,
        n: i32,
        change: &mut PuzzleMarkupChanges,
    ) -> bool {
        let vec_val_pos = match self.data[vector][n as usize - 1]
            .as_ref()
            .and_then(RangeSet::single_value)
        {
            Some(v) => v,
            None => return false,
        };
        let cell_id = puzzle.vector(vector).square_index_at(vec_val_pos);
        debug!(
            "the only possible position for {} in {:?} is {:?}",
            n,
            vector,
            puzzle.cell(cell_id).coord()
        );
        change.cells.solve(cell_id, n);
        self.data.remove_cell_value(cell_id, n);
        true
    }
}

impl<'a> Constraint for VectorValueDomainConstraint<'a> {
    fn notify_changes(
        &mut self,
        changes: &PuzzleMarkupChanges,
        _cell_variables: &Square<CellVariable>,
    ) {
        for (&id, change) in &changes.cells {
            let cell = self.puzzle.cell(id);
            match change {
                CellChange::DomainRemovals(values) => {
                    for &vector in &cell.vectors() {
                        let vector_data = &mut self.data[vector];
                        for &value in values {
                            if let Some(dom) = vector_data[value as usize - 1].as_mut() {
                                let vec_pos = cell.dimension_index(vector.dimension());
                                if dom.remove(vec_pos) {
                                    self.dirty_vec_vals.insert((vector, value));
                                }
                            };
                        }
                    }
                }
                &CellChange::Solution(value) => {
                    self.data.remove_cell_value(id, value);
                }
            }
        }
    }

    fn enforce_partial(&mut self, _: &PuzzleMarkup<'_>, changes: &mut PuzzleMarkupChanges) -> bool {
        while let Some((vector, value)) = self.dirty_vec_vals.pop_front() {
            let solved = self.enforce_vector_value(self.puzzle, vector, value, changes);
            if solved {
                return true;
            }
        }
        false
    }
}

/// Vector -> Value -> vector indices (where the value could be)
#[derive(Clone)]
struct VectorValueIndexSet(Vec<Vec<Option<RangeSet>>>);

impl VectorValueIndexSet {
    pub fn new(puzzle_width: usize) -> VectorValueIndexSet {
        let width = puzzle_width;
        VectorValueIndexSet(vec![
            vec![Some(RangeSet::with_all(width)); width];
            2 * width
        ])
    }

    pub fn remove_cell_value(&mut self, cell_id: CellId, value: Value) {
        for &vector in &self.square().cell(cell_id).vectors() {
            self.remove_vector_value(vector, value);
        }
    }

    pub fn remove_vector_value(&mut self, vector: Vector, value: Value) {
        self[vector][value as usize - 1] = None;
    }

    fn square(&self) -> EmptySquare {
        EmptySquare::new(self.0.len() / 2)
    }
}

impl Index<Vector> for VectorValueIndexSet {
    type Output = Vec<Option<RangeSet>>;

    fn index(&self, vector: Vector) -> &Self::Output {
        &self.0[vector.id()]
    }
}

impl IndexMut<Vector> for VectorValueIndexSet {
    fn index_mut(&mut self, vector: Vector) -> &mut Self::Output {
        &mut self.0[vector.id()]
    }
}
