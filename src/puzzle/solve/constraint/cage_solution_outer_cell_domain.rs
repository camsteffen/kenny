use std::collections::hash_map::Entry;

use super::Constraint;
use crate::collections::iterator_ext::IteratorExt;
use crate::collections::square::{IsSquare, Square, Vector};
use crate::collections::LinkedAHashSet;
use crate::puzzle::solve::markup::{CellChange, PuzzleMarkup, PuzzleMarkupChanges};
use crate::puzzle::solve::{CellVariable, ValueSet};
use crate::puzzle::{CageId, CellId, Puzzle};
use crate::{HashMap, HashSet};
use ahash::AHasher;
use std::hash::BuildHasherDefault;

/// Summary: A cage solution must not conflict with a cell's domain outside of the cage
///
/// Given:
/// * A vector (V)
/// * A cage (G) having a potential solution (S), only including cells in V
/// * A cell (C) with domain (D); C is in V but not in G
///
/// Constraint: S must be a proper subset of D
///
/// Unsatisfied action: Remove S from possible cage solutions
#[derive(Clone)]
pub(crate) struct CageSolutionOuterCellDomainConstraint<'a> {
    puzzle: &'a Puzzle,
    cage_vector_cells: HashMap<(CageId, Vector), HashSet<CellId>>,
    dirty_cage_vectors: LinkedAHashSet<(CageId, Vector)>,
}

impl<'a> CageSolutionOuterCellDomainConstraint<'a> {
    pub fn new(puzzle: &'a Puzzle) -> Self {
        let mut cage_vector_cells: HashMap<(CageId, Vector), HashSet<CellId>> = HashMap::default();
        for cage in puzzle.cages() {
            for cell in cage.cells() {
                for &v in &cell.vectors() {
                    cage_vector_cells
                        .entry((cage.id(), v))
                        .or_default()
                        .insert(cell.id());
                }
            }
        }
        cage_vector_cells.retain(|_, cells| cells.len() > 1);
        cage_vector_cells.shrink_to_fit();

        let dirty_cage_vectors: LinkedAHashSet<_> = cage_vector_cells.keys().copied().collect();
        Self {
            puzzle,
            cage_vector_cells,
            dirty_cage_vectors,
        }
    }
}

impl<'a> Constraint<'a> for CageSolutionOuterCellDomainConstraint<'a> {
    fn notify_changes(
        &mut self,
        changes: &PuzzleMarkupChanges,
        _cell_variables: &Square<CellVariable>,
    ) {
        for (&id, change) in &changes.cells {
            match change {
                CellChange::DomainRemovals(_) => self.notify_cell_domain_removal(id),
                CellChange::Solution(_) => self.notify_cell_solved(id),
            }
        }
    }

    fn enforce_partial(
        &mut self,
        markup: &PuzzleMarkup<'_>,
        changes: &mut PuzzleMarkupChanges,
    ) -> bool {
        while let Some((cage_id, vector)) = self.dirty_cage_vectors.pop_front() {
            let count = self.enforce_cage_vector(markup, cage_id, vector, changes);
            if count > 0 {
                return true;
            }
        }
        false
    }
}

impl CageSolutionOuterCellDomainConstraint<'_> {
    fn notify_cell_domain_removal(&mut self, cell_id: CellId) {
        let cell = self.puzzle.cell(cell_id);
        for &vector in &cell.vectors() {
            let cage_ids = self
                .puzzle
                .vector(vector)
                .iter()
                .map(|cell| cell.cage_id())
                .unique_default::<BuildHasherDefault<AHasher>>();
            for cage_id in cage_ids {
                let key = (cage_id, vector);
                if self.cage_vector_cells.contains_key(&key) {
                    self.dirty_cage_vectors.insert(key);
                }
            }
        }
    }

    fn notify_cell_solved(&mut self, cell_id: CellId) {
        let cell = self.puzzle.cell(cell_id);
        for &v in &cell.vectors() {
            let key = (cell.cage_id(), v);
            if let Entry::Occupied(mut entry) = self.cage_vector_cells.entry(key) {
                let cells = entry.get_mut();
                if cells.len() == 2 {
                    entry.remove();
                } else {
                    let removed = cells.remove(&cell.id());
                    debug_assert!(removed);
                }
            }
        }
    }

    fn enforce_cage_vector(
        &self,
        markup: &PuzzleMarkup<'_>,
        cage_id: CageId,
        vector: Vector,
        changes: &mut PuzzleMarkupChanges,
    ) -> u32 {
        let cage = self.puzzle.cage(cage_id);
        let cage_solutions = &markup.cage_solutions().unwrap()[cage_id];
        if cage_solutions.cell_ids.is_empty() {
            return 0;
        }
        let view = cage_solutions.vector_view(self.puzzle.vector(vector));
        if view.is_empty() {
            return 0;
        }

        // cell domains in the vector, outside the cage, where domain size <= solution size
        let outside_domains: Vec<(CellId, &ValueSet)> = self
            .puzzle
            .vector(vector)
            .iter()
            .filter(|cell| cell.cage_id() != cage_id)
            .filter_map(|cell| {
                markup.cells()[cell.id()]
                    .unsolved()
                    .filter(|domain| domain.len() <= view.len())
                    .map(|domain| (cell.id(), domain))
            })
            .collect();
        if outside_domains.is_empty() {
            return 0;
        }

        let mut count = 0;
        for (solution_index, solution) in view.solutions().enumerate() {
            // solution values for cells in cage and vector
            let mut solution_values = ValueSet::new(self.puzzle.width());
            solution_values.extend(solution.iter().copied());
            for &(cell_id, cell_domain) in &outside_domains {
                if cell_domain
                    .iter()
                    .all(|value| solution_values.contains(value))
                {
                    debug!(
                        "solution {:?} for cage at {:?} conflicts with cell domain at {:?}",
                        solution,
                        cage.coord(),
                        self.puzzle.cell(cell_id).coord()
                    );
                    changes.remove_cage_solution(cage.id(), solution_index);
                    count += 1;
                    break;
                }
            }
        }
        count
    }
}
