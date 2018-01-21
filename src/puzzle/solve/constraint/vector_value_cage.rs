//! If all possible solutions for a given value in a given vector are in a given cage, then the cage solution must
//! contain the given value in the given vector

use collections::FnvLinkedHashSet;
use collections::square::VectorId;
use puzzle::Puzzle;
use puzzle::solve::cage_solutions::CageSolutions;
use puzzle::solve::constraint::Constraint;
use puzzle::solve::markup::PuzzleMarkup;
use puzzle::solve::markup::PuzzleMarkupChanges;

pub struct VectorValueCageConstraint {
    dirty_vector_values: FnvLinkedHashSet<(VectorId, i32)>,
}

impl VectorValueCageConstraint {
    pub fn new(puzzle: &Puzzle) -> Self {
        let dirty_vector_values = (0..puzzle.width as usize * 2).map(|i| VectorId(i)).flat_map(|v| {
            (1..=puzzle.width as i32).map(move |i| ((v, i)))
        }).collect();
        Self {
            dirty_vector_values,
        }
    }
}

impl Constraint for VectorValueCageConstraint {

    fn enforce_partial(&mut self, puzzle: &Puzzle, markup: &PuzzleMarkup, changes: &mut PuzzleMarkupChanges) -> bool {
        while let Some((vector, value)) = self.dirty_vector_values.pop_front() {
            let count = enforce_vector_value(vector, value, puzzle, markup, changes);
            if count > 0 { return true }
        }
        false
    }

    fn notify_changes(&mut self, puzzle: &Puzzle, changes: &PuzzleMarkupChanges) {
        for (&i, values) in &changes.cell_domain_value_removals {
            for vector_id in i.intersecting_vectors(puzzle.width as usize).to_vec() {
                for &value in values {
                    self.dirty_vector_values.insert((vector_id, value));
                }
            }
        }

        for &(sq_idx, value) in &changes.cell_solutions {
            for vector_id in sq_idx.intersecting_vectors(puzzle.width as usize).to_vec() {
                self.dirty_vector_values.remove(&(vector_id, value));
            }
        }
    }
}

fn enforce_vector_value(vector: VectorId, value: i32, puzzle: &Puzzle, markup: &PuzzleMarkup, changes: &mut PuzzleMarkupChanges) -> u32 {
    let mut cages = vector.iter_sq_pos(puzzle.width as usize).filter_map(|i| {
        let has_value = markup.cell_variables[i].unsolved().map_or(false, |d| d.contains(value));
        if has_value {
            Some(puzzle.cage_map[i])
        } else {
            None
        }
    });
    let cage = match cages.next() {
        Some(cage) => cage,
        None => return 0,
    };
    if cages.any(|c| c != cage) {
        return 0
    }
    let CageSolutions { indices, solutions, .. } = &markup.cage_solutions_set[cage as usize];
    let indices_in_vector = indices.iter().enumerate().filter_map(|(i, &sq_idx)| {
        if vector.contains_sq_index(sq_idx, puzzle.width as usize) { Some(i) } else { None }
    }).collect::<Vec<_>>();
    debug_assert!(!indices_in_vector.is_empty());
    let mut count = 0;
    for (soln_idx, solution) in solutions.iter().enumerate() {
        if !indices_in_vector.iter().any(|&i| solution[i] == value) {
            changes.remove_cage_solution(cage, soln_idx);
            count += 1;
        }
    }
    count
}