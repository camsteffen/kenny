use collections::square::{SquareIndex, VectorId};
use fnv::FnvHashMap;
use fnv::FnvHashSet;
use puzzle::solve::markup::PuzzleMarkupChanges;
use puzzle::Puzzle;
use super::Constraint;
use puzzle::solve::PuzzleMarkup;
use collections::FnvLinkedHashSet;
use collections::CloneIndices;

/// A record of values known to be in a certain cage, in a certain vector
pub struct CageVectorValueConstraint {
    cell_cage_vector_map: Vec<(u32, Vec<VectorId>)>,
    dirty_cage_vectors: FnvLinkedHashSet<(u32, VectorId)>,
    known_vector_vals: FnvHashMap<VectorId, FnvHashSet<i32>>,
}

impl CageVectorValueConstraint {
    pub fn new(puzzle: &Puzzle) -> Self {
        Self {
            cell_cage_vector_map: create_cell_cage_vector_map(puzzle),
            dirty_cage_vectors: FnvLinkedHashSet::default(),
            known_vector_vals: FnvHashMap::default(),
        }
    }

    pub fn enforce_cage_vector(&mut self, puzzle: &Puzzle, markup: &PuzzleMarkup, change: &mut PuzzleMarkupChanges, cage_index: u32, vector_id: VectorId) -> u32 {
        let solution_indices = markup.cage_solutions_set[cage_index as usize].indices.iter().cloned().enumerate()
            .filter_map(|(i, sq_idx)| if vector_id.contains_sq_index(sq_idx, puzzle.width as usize) { Some(i) } else { None })
            .collect::<Vec<_>>();
        if solution_indices.len() < 2 { return 0 }

        let mut values: FnvHashSet<i32>;
        let mut solutions_iter = markup.cage_solutions_set[cage_index as usize].solutions.iter()
            .map(|s| s.clone_indices(&solution_indices).into_iter());
        let solution = solutions_iter.next().unwrap();

        // start with values in the first solution that are not already a known vector value
        values = solution
            .filter(|n| {
                self.known_vector_vals.get(&vector_id)
                    .map_or(true, |vals| !vals.contains(n))
            })
            .collect();

        for solution in solutions_iter {
            if values.is_empty() { break }

            // remove values that are not in the current solution
            values = solution
                .filter(|n| values.contains(n))
                .collect();
        }
        
        if values.is_empty() { return 0 }

        self.known_vector_vals.entry(vector_id)
            .or_insert_with(FnvHashSet::default)
            .extend(&values);

        // find cells that are in this vector but not part of this cage
        let remove_from = vector_id.iter_sq_pos(puzzle.width as usize)
            .filter(|&pos| puzzle.cage_map[pos] != cage_index)
            .collect::<Vec<_>>();

        let mut count = 0;
        
        // mark domain values for removal
        debug!("values {:?} exists in cage at {:?}, in {:?}", values,
            puzzle.get_cage(cage_index).cells[0].as_coord(puzzle.width as usize), vector_id);
        for n in values {
            for &pos in &remove_from {
                if markup.cell_variables[pos].unsolved_and_contains(n) {
                    change.remove_value_from_cell(pos, n);
                    count += 1;
                }
            }
        }
        count
    }

    fn notify_change_cell_domain(&mut self, index: SquareIndex) {
        let (cage_index, vector_ids) = &self.cell_cage_vector_map[index.0];
        for vector_id in vector_ids {
            self.dirty_cage_vectors.insert((*cage_index, *vector_id));
        }
    }
}

impl Constraint for CageVectorValueConstraint {
    fn enforce_partial(&mut self, puzzle: &Puzzle, markup: &PuzzleMarkup, changes: &mut PuzzleMarkupChanges) -> bool {
        while let Some((cage_index, vector_id)) = self.dirty_cage_vectors.pop_front() {
            let count = self.enforce_cage_vector(puzzle, markup, changes, cage_index, vector_id);
            if count > 0 { return true }
        }
        false
    }

    fn notify_changes(&mut self, puzzle: &Puzzle, changes: &PuzzleMarkupChanges) {
        for &index in changes.cell_domain_value_removals.keys() {
            self.notify_change_cell_domain(index);
        }
        for &cage_index in changes.cage_solution_removals.keys() {
            for &index in &puzzle.get_cage(cage_index).cells {
                self.notify_change_cell_domain(index);
            }
        }
    }
}

// TODO commonize metadata
fn create_cell_cage_vector_map(puzzle: &Puzzle) -> Vec<(u32, Vec<VectorId>)> {
    (0..puzzle.width.pow(2) as usize).map(|i| {
        let index = SquareIndex(i);
        let cage_index = puzzle.cage_map[index];
        let vector_ids = index.intersecting_vectors(puzzle.width as usize).into_iter().cloned()
            .filter(|v| {
                puzzle.get_cage(cage_index).cells.iter()
                    .filter(|&&i| v.contains_sq_index(i, puzzle.width as usize))
                    .take(2).count() == 2
            })
            .collect();
        (cage_index, vector_ids)
    }).collect()
}
