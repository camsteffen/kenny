use super::Constraint;
use crate::collections::square::VectorId;
use crate::collections::LinkedAHashSet;
use crate::puzzle::solve::cage_solutions::CageSolutions;
use crate::puzzle::solve::constraint::State;
use crate::puzzle::solve::PuzzleMarkupChanges;
use crate::puzzle::solve::{PuzzleMarkup, ValueSet};
use crate::puzzle::{CageId, CageRef, CellId, Puzzle};
use ahash::{AHashMap, AHashSet};
use std::collections::hash_map::Entry;

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
pub struct CageSolutionOuterCellDomainConstraint {
    cage_vector_cells: AHashMap<(CageId, VectorId), AHashSet<CellId>>,
    dirty_cage_vectors: LinkedAHashSet<(CageId, VectorId)>,
}

impl CageSolutionOuterCellDomainConstraint {
    pub fn new(puzzle: &Puzzle) -> Self {
        let mut cage_vector_cells = AHashMap::default();
        for cage in puzzle.cages() {
            for cell in cage.cells() {
                for &v in &cell.vectors() {
                    cage_vector_cells
                        .entry((cage.id(), v))
                        .or_insert_with(AHashSet::new)
                        .insert(cell.id());
                }
            }
        }
        cage_vector_cells.retain(|_, cells| cells.len() > 1);
        cage_vector_cells.shrink_to_fit();

        let dirty_cage_vectors: LinkedAHashSet<_> = cage_vector_cells.keys().copied().collect();
        Self {
            cage_vector_cells,
            dirty_cage_vectors,
        }
    }
}

impl Constraint for CageSolutionOuterCellDomainConstraint {
    fn notify_changes(&mut self, puzzle: &Puzzle, changes: &PuzzleMarkupChanges) {
        for &(id, _) in &changes.cell_solutions {
            let cell = puzzle.cell(id);
            for &v in &cell.vectors() {
                let key = (cell.cage_id(), v);
                if let Entry::Occupied(mut entry) = self.cage_vector_cells.entry(key) {
                    let cells = entry.get_mut();
                    if cells.len() == 2 {
                        entry.remove();
                    } else {
                        let removed = cells.remove(&id);
                        debug_assert!(removed);
                    }
                }
            }
        }
        for &id in changes.cell_domain_value_removals.keys() {
            let cell = puzzle.cell(id);
            let cage_id = cell.cage_id();
            for &v in &cell.vectors() {
                let key = (cage_id, v);
                if self.cage_vector_cells.contains_key(&key) {
                    self.dirty_cage_vectors.insert(key);
                }
            }
        }
    }

    fn enforce_partial(
        &mut self,
        puzzle: &Puzzle,
        markup: &PuzzleMarkup,
        changes: &mut PuzzleMarkupChanges,
    ) -> bool {
        while let Some((cage_id, vector_id)) = self.dirty_cage_vectors.pop_front() {
            let count =
                enforce_cage_vector(puzzle, markup, puzzle.cage(cage_id), vector_id, changes);
            if count > 0 {
                return true;
            }
        }
        false
    }

    fn state(&self) -> State {
        // TODO inconsistent state?
        if self.dirty_cage_vectors.is_empty() {
            State::SATISFIED
        } else {
            State::PENDING
        }
    }
}

fn enforce_cage_vector(
    puzzle: &Puzzle,
    markup: &PuzzleMarkup,
    cage: CageRef<'_>,
    vector_id: VectorId,
    changes: &mut PuzzleMarkupChanges,
) -> u32 {
    let CageSolutions {
        cell_ids,
        solutions,
        ..
    } = &markup.cage_solutions()[cage.id()];

    if cell_ids.is_empty() {
        // the cage is solved
        return 0;
    }

    // indices within each solution where the cell is part of the vector
    let soln_indices = cell_ids
        .iter()
        .copied()
        .enumerate()
        .filter(|&(_, sq_idx)| puzzle.cell(sq_idx).is_in_vector(vector_id))
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    if soln_indices.is_empty() {
        return 0;
    }

    // cell domains in the vector, outside the cage, where domain size <= solution size
    let outside_domains: Vec<(CellId, &ValueSet)> = puzzle
        .vector_cells(vector_id)
        .filter(|cell| cell.cage_id() != cage.id())
        .filter_map(|cell| {
            if let Some(domain) = markup.cells()[cell.id()].unsolved() {
                if domain.len() <= soln_indices.len() {
                    return Some((cell.id(), domain));
                }
            }
            None
        })
        .collect();
    if outside_domains.is_empty() {
        return 0;
    }

    let mut count = 0;
    for (solution_index, solution) in solutions.iter().enumerate() {
        // solution values for cells in cage and vector
        let mut solution_values = ValueSet::new(puzzle.width());
        solution_values.extend(soln_indices.iter().map(|&i| solution[i]));
        for &(cell_id, cell_domain) in &outside_domains {
            if cell_domain
                .iter()
                .all(|value| solution_values.contains(value))
            {
                debug!(
                    "solution {:?} for cage at {:?} conflicts with cell domain at {:?}",
                    solution,
                    cage.coord(),
                    puzzle.cell(cell_id).coord()
                );
                changes.remove_cage_solution(cage.id(), solution_index);
                count += 1;
                break;
            }
        }
    }
    count
}
