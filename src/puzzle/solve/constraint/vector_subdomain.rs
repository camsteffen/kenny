use crate::collections::square::{VectorId, IsSquare};
use crate::collections::square::Square;
use crate::puzzle::solve::CellVariable;
use itertools::Itertools;
use crate::puzzle::solve::ValueSet;
use crate::puzzle::solve::markup::PuzzleMarkupChanges;
use crate::puzzle::{Puzzle, CellId};
use crate::puzzle::solve::PuzzleMarkup;
use super::Constraint;
use crate::collections::LinkedAHashSet;
use crate::collections::iterator_ext::IteratorExt;

/// If there is a set of cells within a vector where the size of the union of their domains is less than or equal to
/// the number of cells, then all of the values in the unified domain must be in that set of cells.
#[derive(Clone)]
pub struct VectorSubdomainConstraint {
    dirty_vecs: LinkedAHashSet<VectorId>,
}

impl VectorSubdomainConstraint {
    pub fn new() -> Self {
        Self {
            dirty_vecs: Default::default(),
        }
    }

    fn enforce_vector(
        cell_variables: &Square<CellVariable>,
        vector_id: VectorId,
        change: &mut PuzzleMarkupChanges
    ) -> u32 {
        let unsolved_count = cell_variables.vector(vector_id).filter(|v| v.is_unsolved()).count();
        if unsolved_count < 3 {
            return 0;
        }
        let max_domain = unsolved_count - 1;

        // list lists of unsolved cell IDs, outer list sorted by domain size ascending
        let mut cells_by_domain_size = vec![Vec::new(); max_domain - 2];
        for (index, variable) in cell_variables.vector_with_indices(vector_id) {
            if let Some(domain) = variable.unsolved() {
                if domain.len() < max_domain {
                    if domain.len().checked_sub(2).is_none() {
                        panic!("o no!") // todo
                    }
                    // domain is at least 2, so offset accordingly
                    cells_by_domain_size[domain.len() - 2]
                        .push(index);
                }
            }
        }

        let mut count = 0;

        // TODO optimize?

        // find a set of cells where the union of their domains is less than the number of cells
        let mut cells: Vec<CellId> = Vec::with_capacity(cell_variables.width() - 1);
        'domain_sizes: for (i, cells2) in cells_by_domain_size.into_iter().enumerate() {
            if cells2.is_empty() { continue }

            // merge the cells, maintaining order
            cells = cells.into_iter().merge(cells2).collect();

            let max_domain_size = i + 2;

            for cells in cells.iter().copied().combinations(max_domain_size) {
                if let Some(domain) = unify_domain(cell_variables, &cells, max_domain_size) {
                    count += found_vector_subdomain(cell_variables, change, vector_id, &cells, &domain);
                    break 'domain_sizes;
                }
            }
        }
        count
    }
}

impl Constraint for VectorSubdomainConstraint {
    fn notify_changes(&mut self, puzzle: &Puzzle, changes: &PuzzleMarkupChanges) {
        for &index in changes.cell_domain_value_removals.keys() {
            for vector_id in puzzle.cell(index).vectors().iter().copied() {
                self.dirty_vecs.insert(vector_id);
            }
        }
    }

    fn enforce_partial(&mut self, _: &Puzzle, markup: &PuzzleMarkup, changes: &mut PuzzleMarkupChanges) -> bool {
        while let Some(vector_id) = self.dirty_vecs.front().copied() {
            let count = Self::enforce_vector(&markup.cells(), vector_id, changes);
            if count == 0 {
                self.dirty_vecs.pop_front();
            } else {
                return true;
            }
        }
        false
    }
}

fn unify_domain(cell_variables: &Square<CellVariable>, cells: &[CellId], max_size: usize) -> Option<ValueSet> {
    let mut domain = ValueSet::new(cell_variables.width());
    for &cell in cells {
        for j in cell_variables[cell].unsolved().unwrap() {
            if domain.insert(j) && domain.len() > max_size {
                // the domain is too big
                return None;
            }
        }
    }
    Some(domain)
}

fn found_vector_subdomain(
    cell_variables: &Square<CellVariable>,
    changes: &mut PuzzleMarkupChanges,
    vector_id: VectorId,
    cells: &[CellId],
    domain: &ValueSet
) -> u32 {
    debug!("values {:?} are among cells {:?}", domain.iter().collect::<Vec<_>>(),
           cells.iter().map(|&i| cell_variables.coord_at(i)).collect::<Vec<_>>());

    let mut count = 0;

    let other_cells: Vec<CellId> = vector_id.indices(cell_variables)
        .left_merge(cells.iter().copied())
        .collect();
    for cell in other_cells {
        for value in domain {
            if cell_variables[cell].unsolved_and_contains(value) {
                changes.remove_value_from_cell(cell, value);
                count += 1;
            }
        }
    }
    count
}
