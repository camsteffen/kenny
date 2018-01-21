use collections::FnvLinkedHashSet;
use collections::GetIndicesCloned;
use collections::square::SquareIndex;
use collections::square::VectorId;
use fnv::FnvHashMap;
use fnv::FnvHashSet;
use puzzle::Puzzle;
use puzzle::solve::PuzzleMarkup;
use puzzle::solve::PuzzleMarkupChanges;
use super::Constraint;

pub struct CageSolutionVectorDomainConstraint {
    cage_vector_cells: FnvHashMap<(u32, VectorId), FnvHashSet<SquareIndex>>,
    dirty_cage_vectors: FnvLinkedHashSet<(u32, VectorId)>,
}

impl CageSolutionVectorDomainConstraint {
    pub fn new(puzzle: &Puzzle) -> Self {
        let mut cage_vector_cells = FnvHashMap::default();
        for (cage_index, cage) in puzzle.cages.iter().enumerate() {
            for &index in &cage.cells {
                for v in index.intersecting_vectors(puzzle.width as usize).to_vec() {
                    cage_vector_cells.entry((cage_index as u32, v))
                        .or_insert_with(FnvHashSet::default)
                        .insert(index);
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
    fn enforce_partial(&mut self, puzzle: &Puzzle, markup: &PuzzleMarkup, changes: &mut PuzzleMarkupChanges) -> bool {
        while let Some((cage_index, vector_id)) = self.dirty_cage_vectors.pop_front() {
            let count = enforce_cage_vector(puzzle, markup, cage_index, vector_id, changes);
            if count > 0 { return true }
        }
        false
    }

    fn notify_changes(&mut self, puzzle: &Puzzle, changes: &PuzzleMarkupChanges) {
        for &(index, _) in &changes.cell_solutions {
            let cage_index = puzzle.cage_map[index];
            for v in index.intersecting_vectors(puzzle.width as usize).to_vec() {
                let key = (cage_index, v);
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
            let cage_index = puzzle.cage_map[index];
            for v in index.intersecting_vectors(puzzle.width as usize).to_vec() {
                let key = (cage_index, v);
                if self.cage_vector_cells.contains_key(&key) {
                    self.dirty_cage_vectors.insert(key);
                }
            }
        }
    }
}

fn enforce_cage_vector(puzzle: &Puzzle, markup: &PuzzleMarkup, cage_index: u32, vector_id: VectorId, changes: &mut PuzzleMarkupChanges) -> u32 {
    let cage_solutions = &markup.cage_solutions_set[cage_index as usize];
    let soln_indices = cage_solutions.indices.iter().cloned().enumerate()
        .filter_map(|(i, sq_idx)| {
            if vector_id.contains_sq_index(sq_idx, puzzle.width as usize) {
                Some(i)
            } else { None }
        })
        .collect::<Vec<_>>();
    let cage = &puzzle.get_cage(cage_index);
    let cage_cells = cage.cells.iter().filter(|&&i| vector_id.contains_sq_index(i, puzzle.width as usize)).collect::<FnvHashSet<_>>();
    let outide_domains = vector_id.iter_sq_pos(puzzle.width as usize)
        .filter_map(|i| {
            if cage_cells.contains(&i) { return None }
            markup.cell_variables[i].unsolved().map(|d| (i, d))
        })
        .collect::<Vec<_>>();
    let mut count = 0;
    'solutions: for (i, solution) in cage_solutions.solutions.iter().enumerate() {
        let values = solution.get_indices_cloned(&soln_indices).into_iter().collect::<FnvHashSet<_>>();
        for &(j, domain) in &outide_domains {
            if domain.len() > values.len() { continue }
            if domain.iter().all(|v| values.contains(&v)) {
                debug!("solution {:?} for cage at {:?} conflicts with cell domain at {:?}",
                       solution, cage.cells[0].as_coord(puzzle.width as usize), j.as_coord(puzzle.width as usize));
                changes.remove_cage_solution(cage_index, i);
                count += 1;
                continue 'solutions
            }
        }
    }
    count
}
