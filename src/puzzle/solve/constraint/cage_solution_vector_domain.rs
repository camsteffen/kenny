use crate::collections::FnvLinkedHashSet;
use crate::collections::square::SquareIndex;
use crate::collections::square::VectorId;
use fnv::FnvHashMap;
use fnv::FnvHashSet;
use crate::puzzle::{Puzzle, CellRef};
use crate::puzzle::solve::PuzzleMarkup;
use crate::puzzle::solve::PuzzleMarkupChanges;
use super::Constraint;

pub struct CageSolutionVectorDomainConstraint {
    cage_vector_cells: FnvHashMap<(usize, VectorId), FnvHashSet<SquareIndex>>,
    dirty_cage_vectors: FnvLinkedHashSet<(usize, VectorId)>,
}

impl CageSolutionVectorDomainConstraint {
    pub fn new(puzzle: &Puzzle) -> Self {
        let mut cage_vector_cells = FnvHashMap::default();
        for cage in puzzle.cages().iter() {
            for cell in cage.cells() {
                for v in cell.index().intersecting_vectors(puzzle.width()).to_vec() {
                    cage_vector_cells.entry((cage.index(), v))
                        .or_insert_with(FnvHashSet::default)
                        .insert(cell.index());
                }
            }
        }
        cage_vector_cells.retain(|_, cells| cells.len() > 1);
        cage_vector_cells.shrink_to_fit();
        let dirty_cage_vectors = cage_vector_cells.keys().cloned().collect();
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
            let cage = cell.cage();
            for v in index.intersecting_vectors(puzzle.width()).to_vec() {
                let key = (cage.index(), v);
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
            for v in index.intersecting_vectors(puzzle.width()).to_vec() {
                let key = (cage_index, v);
                if self.cage_vector_cells.contains_key(&key) {
                    self.dirty_cage_vectors.insert(key);
                }
            }
        }
    }

    fn enforce_partial(&mut self, puzzle: &Puzzle, markup: &PuzzleMarkup, changes: &mut PuzzleMarkupChanges) -> bool {
        while let Some((cage_index, vector_id)) = self.dirty_cage_vectors.pop_front() {
            let count = enforce_cage_vector(puzzle, markup, cage_index, vector_id, changes);
            if count > 0 { return true }
        }
        false
    }
}

fn enforce_cage_vector(puzzle: &Puzzle, markup: &PuzzleMarkup, cage_index: usize, vector_id: VectorId, changes: &mut PuzzleMarkupChanges) -> u32 {
    let cage_solutions = &markup.cage_solutions()[cage_index];
    let soln_indices = cage_solutions.indices.iter().cloned().enumerate()
        .filter_map(|(i, sq_idx)| {
            if vector_id.contains_sq_index(sq_idx, puzzle.width()) {
                Some(i)
            } else { None }
        })
        .collect::<Vec<_>>();
    let cage = puzzle.cage(cage_index);
    let cage_cells = cage.cells()
        .filter(|cell| vector_id.contains_sq_index(cell.index(), puzzle.width()))
        .map(CellRef::index)
        .collect::<FnvHashSet<_>>();
    let outside_domains = vector_id.iter_sq_pos(puzzle.width())
        .filter_map(|i| {
            if cage_cells.contains(&i) { return None }
            markup.cells()[i].unsolved().map(|d| (i, d))
        })
        .collect::<Vec<_>>();
    let mut count = 0;
    'solutions: for (i, solution) in cage_solutions.solutions.iter().enumerate() {
        let values = soln_indices.iter().map(|&i| solution[i]).collect::<FnvHashSet<_>>();
        for &(j, domain) in &outside_domains {
            if domain.len() > values.len() { continue }
            if domain.iter().all(|v| values.contains(&v)) {
                debug!("solution {:?} for cage at {:?} conflicts with cell domain at {:?}",
                       solution, cage.cell(0).coord(), puzzle.cell(j).coord());
                changes.remove_cage_solution(cage_index, i);
                count += 1;
                continue 'solutions
            }
        }
    }
    count
}
