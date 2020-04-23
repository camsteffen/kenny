use crate::collections::LinkedAHashSet;
use crate::collections::square::SquareIndex;
use crate::collections::square::VectorId;
use crate::puzzle::{Puzzle, CellRef, CageRef};
use crate::puzzle::solve::PuzzleMarkup;
use crate::puzzle::solve::PuzzleMarkupChanges;
use super::Constraint;
use crate::puzzle::solve::constraint::State;
use ahash::{AHashMap, AHashSet};

pub struct CageSolutionVectorDomainConstraint {
    /// Key: Cage ID and Vector ID pair
    /// Value:
    cage_vector_cells: AHashMap<(usize, VectorId), AHashSet<SquareIndex>>,
    dirty_cage_vectors: LinkedAHashSet<(usize, VectorId)>,
}

impl CageSolutionVectorDomainConstraint {
    pub fn new(puzzle: &Puzzle) -> Self {
        let mut cage_vector_cells = AHashMap::default();
        for cage in puzzle.cages().iter() {
            for cell in cage.cells() {
                for &v in &cell.vectors() {
                    cage_vector_cells.entry((cage.index(), v))
                        .or_insert_with(AHashSet::default)
                        .insert(cell.index());
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

impl Constraint for CageSolutionVectorDomainConstraint {
    fn notify_changes(&mut self, puzzle: &Puzzle, changes: &PuzzleMarkupChanges) {
        for &(index, _) in &changes.cell_solutions {
            let cell = puzzle.cell(index);
            for &v in &cell.vectors() {
                let key = (cell.cage().index(), v);
                let remove = self.cage_vector_cells.get_mut(&key).map_or(false, |cells| {
                    if cells.len() == 2 { true }
                    else {
                        let removed = cells.remove(&index);
                        debug_assert!(removed);
                        false
                    }
                });
                if remove {
                    self.cage_vector_cells.remove(&key);
                }
            }
        }
        for &index in changes.cell_domain_value_removals.keys() {
            let cage_index = puzzle.cell(index).cage().index();
            for &v in &puzzle.cell(index).vectors() {
                let key = (cage_index, v);
                if self.cage_vector_cells.contains_key(&key) {
                    self.dirty_cage_vectors.insert(key);
                }
            }
        }
    }

    fn enforce_partial(&mut self, puzzle: &Puzzle, markup: &PuzzleMarkup, changes: &mut PuzzleMarkupChanges) -> bool {
        while let Some((cage_index, vector_id)) = self.dirty_cage_vectors.pop_front() {
            let count = enforce_cage_vector(puzzle, markup, puzzle.cage(cage_index), vector_id, changes);
            if count > 0 { return true }
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
    cage: CageRef,
    vector_id: VectorId,
    changes: &mut PuzzleMarkupChanges
) -> u32 {
    let cage_solutions = &markup.cage_solutions()[cage.index()];
    // indices of each solution where the cell is part of the vector
    let soln_indices = cage_solutions.indices.iter().copied().enumerate()
        .filter_map(|(i, sq_idx)| {
            if puzzle.cell(sq_idx).intersects_vector(vector_id) {
                Some(i)
            } else { None }
        })
        .collect::<Vec<_>>();
    let cage_cells: AHashSet<_> = cage.cells()
        .filter(|cell| cell.intersects_vector(vector_id))
        .map(CellRef::index)
        .collect();
    let outside_domains = puzzle.vector_cells(vector_id)
        .filter_map(|cell| {
            if cage_cells.contains(&cell.index()) { return None }
            markup.cells()[cell.index()].unsolved().map(|d| (cell.index(), d))
        })
        .collect::<Vec<_>>();
    let mut count = 0;
    'solutions: for (i, solution) in cage_solutions.solutions.iter().enumerate() {
        let values: AHashSet<_> = soln_indices.iter().map(|&i| solution[i]).collect();
        for &(j, domain) in &outside_domains {
            if domain.len() > values.len() { continue }
            if domain.iter().all(|v| values.contains(&v)) {
                debug!("solution {:?} for cage at {:?} conflicts with cell domain at {:?}",
                       solution, cage.cell(0).coord(), puzzle.cell(j).coord());
                changes.remove_cage_solution(cage.index(), i);
                count += 1;
                continue 'solutions
            }
        }
    }
    count
}
