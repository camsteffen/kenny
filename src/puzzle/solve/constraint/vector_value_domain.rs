use collections::RangeSet;
use std::ops::{Index, IndexMut};
use collections::square::VectorId;
use puzzle::solve::markup::PuzzleMarkupChanges;
use puzzle::Puzzle;
use puzzle::solve::PuzzleMarkup;
use super::Constraint;
use collections::FnvLinkedHashSet;
use collections::square::SquareIndex;

pub struct VectorValueDomainConstraint {
    data: VectorValueSet,
    dirty_vec_vals: FnvLinkedHashSet<(VectorId, i32)>,
}

impl VectorValueDomainConstraint {
    pub fn new(puzzle_width: u32) -> Self {
        Self {
            data: VectorValueSet::new(puzzle_width),
            dirty_vec_vals: FnvLinkedHashSet::default(),
        }
    }

    fn enforce_vector_value(&mut self, puzzle_width: u32, vector_id: VectorId, n: i32, change: &mut PuzzleMarkupChanges) -> bool {
        let vec_val_pos = match self.data[vector_id][n as usize - 1].as_ref().and_then(|dom| dom.single_value()) {
            Some(v) => v,
            None => return false,
        };
        let sq_pos = vector_id.vec_pos_to_sq_pos(vec_val_pos as usize, puzzle_width as usize);
        debug!("the only possible position for {} in {:?} is {:?}", n, vector_id, sq_pos.as_coord(puzzle_width as usize));
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
            let solved = self.enforce_vector_value(puzzle.width, vector_id, value, changes);
            if solved { return true }
        }
        false
    }

    fn notify_changes(&mut self, puzzle: &Puzzle, changes: &PuzzleMarkupChanges) {
        for &(index, value) in &changes.cell_solutions {
            self.data.remove_index_value(puzzle.width, index, value);
        }
        for (&index, values) in &changes.cell_domain_value_removals {
            for &v in &index.intersecting_vectors(puzzle.width as usize) {
                for &value in values {
                    if let Some(dom) = self.data[v][value as usize - 1].as_mut() {
                        let vec_pos = v.sq_pos_to_vec_pos(index, puzzle.width as usize);
                        let removed = dom.remove(vec_pos);
                        debug_assert!(removed);
                        self.dirty_vec_vals.insert((v, value));
                    };
                }
            }
        }
    }
}

struct VectorValueSet(Vec<Vec<Option<RangeSet>>>);

impl VectorValueSet {
    pub fn new(puzzle_width: u32) -> VectorValueSet {
        let width = puzzle_width as usize;
        VectorValueSet(vec![vec![Some(RangeSet::with_all(width)); width]; 2 * width])
    }

    pub fn remove_index_value(&mut self, puzzle_width: u32, index: SquareIndex, value: i32) {
        for vector_id in index.intersecting_vectors(puzzle_width as usize).to_vec() {
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
