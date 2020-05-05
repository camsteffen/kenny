use super::Constraint;
use crate::collections::square::{IsSquare, Vector};
use crate::collections::{LinkedAHashSet, RangeSet};
use crate::puzzle::solve::markup::PuzzleMarkupChanges;
use crate::puzzle::solve::PuzzleMarkup;
use crate::puzzle::{CellRef, Puzzle, Value};
use std::ops::{Index, IndexMut};

/// If only one cell in a vector has a given value in its domain, then the cell has that value.
#[derive(Clone)]
pub struct VectorValueDomainConstraint {
    data: VectorValueIndexSet,
    dirty_vec_vals: LinkedAHashSet<(Vector, i32)>,
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
        let sq_pos = puzzle.vector_point(vector, vec_val_pos);
        debug!(
            "the only possible position for {} in {:?} is {:?}",
            n,
            vector,
            puzzle.coord_at(sq_pos)
        );
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

    fn enforce_partial(
        &mut self,
        puzzle: &Puzzle,
        _: &PuzzleMarkup,
        changes: &mut PuzzleMarkupChanges,
    ) -> bool {
        while let Some((vector, value)) = self.dirty_vec_vals.pop_front() {
            let solved = self.enforce_vector_value(puzzle, vector, value, changes);
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

    pub fn remove_cell_value(&mut self, cell: CellRef<'_>, value: Value) {
        for &vector in &cell.vectors() {
            self.remove_vector_value(vector, value);
        }
    }

    pub fn remove_vector_value(&mut self, vector: Vector, value: Value) {
        self[vector][value as usize - 1] = None;
    }
}

impl Index<Vector> for VectorValueIndexSet {
    type Output = Vec<Option<RangeSet>>;

    fn index(&self, vector: Vector) -> &Self::Output {
        &self.0[usize::from(vector)]
    }
}

impl IndexMut<Vector> for VectorValueIndexSet {
    fn index_mut(&mut self, vector: Vector) -> &mut Self::Output {
        &mut self.0[usize::from(vector)]
    }
}
