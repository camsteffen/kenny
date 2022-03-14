use itertools::Itertools;

use super::Constraint;
use crate::collections::iterator_ext::IteratorExt;
use crate::collections::square::Square;
use crate::collections::square::{IsSquare, Vector};
use crate::puzzle::{CellId, Puzzle};
use crate::solve::markup::{PuzzleMarkup, PuzzleMarkupChanges};
use crate::solve::CellVariable;
use crate::solve::ValueSet;
use crate::LinkedHashSet;

/// If there is a set of cells within a vector where the size of the union of their domains is equal to
/// the number of cells, then all of the values in the unified domain must be in that set of cells.
#[derive(Clone)]
pub(crate) struct VectorPreemptiveSetConstraint<'a> {
    puzzle: &'a Puzzle,
    dirty_vecs: LinkedHashSet<Vector>,
}

impl<'a> VectorPreemptiveSetConstraint<'a> {
    pub fn new(puzzle: &'a Puzzle) -> Self {
        Self {
            puzzle,
            dirty_vecs: LinkedHashSet::default(),
        }
    }
}

impl<'a> Constraint for VectorPreemptiveSetConstraint<'a> {
    fn notify_changes(
        &mut self,
        changes: &PuzzleMarkupChanges,
        _cell_variables: &Square<CellVariable>,
    ) {
        for (id, _) in changes.cells.domain_removals() {
            for vector in self.puzzle.cell(id).vectors().iter().copied() {
                self.dirty_vecs.insert(vector);
            }
        }
    }

    fn enforce_partial(
        &mut self,
        markup: &PuzzleMarkup<'_>,
        changes: &mut PuzzleMarkupChanges,
    ) -> bool {
        while let Some(vector) = self.dirty_vecs.front().copied() {
            let count = enforce_vector(&markup.cells(), vector, changes);
            if count == 0 {
                self.dirty_vecs.pop_front();
            } else {
                return true;
            }
        }
        false
    }
}

fn enforce_vector(
    cell_variables: &Square<CellVariable>,
    vector: Vector,
    change: &mut PuzzleMarkupChanges,
) -> u32 {
    let unsolved_count = cell_variables
        .vector(vector)
        .iter()
        .filter(|v| v.is_unsolved())
        .count();
    if unsolved_count < 3 {
        return 0;
    }
    let max_domain = unsolved_count - 1;

    // list lists of unsolved cell IDs, outer list sorted by domain size ascending
    let mut cells_by_domain_size = vec![Vec::new(); max_domain - 2];
    for (index, variable) in cell_variables.vector(vector).indexed() {
        if let Some(domain) = variable.unsolved() {
            if domain.len() < max_domain {
                // domain is at least 2, so offset accordingly
                cells_by_domain_size[domain.len() - 2].push(index);
            }
        }
    }

    let mut count = 0;

    // TODO can this be optimized?

    // find a set of cells where the size of the union of their domains is equal to the number of cells
    let mut cells: Vec<CellId> = Vec::with_capacity(cell_variables.width() as usize - 1);
    'domain_sizes: for (i, cells2) in cells_by_domain_size.into_iter().enumerate() {
        if cells2.is_empty() {
            continue;
        }

        // merge the cells, maintaining order
        cells = cells.into_iter().merge(cells2).collect();

        let max_domain_size = i + 2;

        for cells in cells.iter().copied().combinations(max_domain_size) {
            if let Some(domain) = unify_domain(cell_variables, &cells, max_domain_size) {
                count += found_preemptive_set(cell_variables, change, vector, &cells, &domain);
                break 'domain_sizes;
            }
        }
    }
    count
}

fn unify_domain(
    cell_variables: &Square<CellVariable>,
    cells: &[CellId],
    target_size: usize,
) -> Option<ValueSet> {
    let mut domain = ValueSet::new(cell_variables.width() as usize);
    for &cell in cells {
        for j in cell_variables[cell].unsolved().unwrap() {
            if domain.insert(j) && domain.len() > target_size {
                // the domain is too big
                return None;
            }
        }
    }
    debug_assert_eq!(domain.len(), target_size);
    Some(domain)
}

fn found_preemptive_set(
    cell_variables: &Square<CellVariable>,
    changes: &mut PuzzleMarkupChanges,
    vector: Vector,
    cells: &[CellId],
    values: &ValueSet,
) -> u32 {
    debug!(
        "values {:?} are among cells {:?}",
        values.iter().collect::<Vec<_>>(),
        cells
            .iter()
            .map(|&i| cell_variables.cell(i).coord())
            .collect::<Vec<_>>()
    );

    let mut count = 0;

    let other_cells: Vec<CellId> = cell_variables
        .vector(vector)
        .indices()
        .left_merge(cells.iter().copied())
        .collect();
    for cell in other_cells {
        for value in values {
            if cell_variables[cell].unsolved_and_contains(value) {
                changes.cells.remove_domain_value(cell, value);
                count += 1;
            }
        }
    }
    count
}
