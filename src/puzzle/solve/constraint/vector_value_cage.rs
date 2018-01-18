//! If all possible solutions for a given value in a given vector are in a given cage, then the cage solution must
//! contain the given value in the given vector

use collections::FnvLinkedHashMap;
use collections::FnvLinkedHashSet;
use collections::square::VectorId;
use fnv::FnvHashMap;
use puzzle::Puzzle;
use puzzle::solve::cage_solutions::CageSolutions;
use puzzle::solve::constraint::Constraint;
use puzzle::solve::markup::PuzzleMarkup;
use puzzle::solve::markup::PuzzleMarkupChanges;

pub struct VectorValueCageConstraint {
    known_vector_value_cages: FnvLinkedHashMap<(VectorId, i32), usize>,
    vector_value_cages: FnvHashMap<(VectorId, i32), FnvLinkedHashMap<usize, u32>>,
}

impl VectorValueCageConstraint {
    pub fn new(puzzle: &Puzzle) -> Self {
        let vector_value_cages = (0..puzzle.width * 2).map(|i| VectorId(i)).flat_map(|v| {
            let cages = v.iter_sq_pos(puzzle.width).map(|i| puzzle.cage_map[i])
                .fold(FnvLinkedHashMap::default(), |mut cages, cage| {
                    *cages.entry(cage).or_insert(0) += 1;
                    cages
                });
            (1..=puzzle.width as i32).map(move |i| ((v, i), cages.clone()))
        }).collect();
        Self {
            known_vector_value_cages: FnvLinkedHashMap::default(),
            vector_value_cages,
        }
    }
}

impl Constraint for VectorValueCageConstraint {

    fn enforce_partial(&mut self, puzzle: &Puzzle, markup: &PuzzleMarkup, changes: &mut PuzzleMarkupChanges) -> bool {
        while let Some(((vector, value), cage)) = self.known_vector_value_cages.pop_front() {
            let CageSolutions { indices, solutions, .. } = &markup.cage_solutions_set[cage];
            let indices_in_vector = indices.iter().enumerate().filter_map(|(i, &sq_idx)| {
                if vector.contains_sq_index(sq_idx, puzzle.width) { Some(i) } else { None }
            }).collect::<Vec<_>>();
            //if indices_in_vector.is_empty() { continue } // TODO rewrite
            let mut count = 0;
            for (soln_idx, solution) in solutions.iter().enumerate() {
                if !indices_in_vector.iter().any(|&i| solution[i] == value) {
                    changes.remove_cage_solution(cage, soln_idx);
                    count += 1;
                }
            }
            if count > 0 { return true }
        }
        false
    }

    fn notify_changes(&mut self, puzzle: &Puzzle, changes: &PuzzleMarkupChanges) {
        for (&i, values) in &changes.cell_domain_value_removals {
            let cage = puzzle.cage_map[i];
            for vector_id in i.intersecting_vectors(puzzle.width).to_vec() {
                for &value in values {
                    let key = (vector_id, value);
                    let cages = self.vector_value_cages.get_mut(&key).unwrap();
                    if cages.is_empty() { continue }
                    let cage_empty = {
                        let cage_count = cages.get_mut(&cage).unwrap();
                        *cage_count -= 1;
                        *cage_count == 0
                    };
                    if cage_empty {
                        cages.remove(&cage);
                        if cages.len() == 1 {
                            let cage = cages.pop_front().unwrap().0;
                            debug!("The {} in {:?} must be in cage at {:?}", value, vector_id, puzzle.cages[cage].cells[0].as_coord(puzzle.width));
                            if vector_id == VectorId::row(0) && value == 4 {
                                debug!("der");
                            }
                            self.known_vector_value_cages.insert((vector_id, value), cage);
                        }
                    }
                }
            }
        }

        for &(sq_idx, value) in &changes.cell_solutions {
            for vector_id in sq_idx.intersecting_vectors(puzzle.width).to_vec() {
                if sq_idx.0 == 1 {
                    debug!("der");
                }
                self.known_vector_value_cages.remove(&(vector_id, value));
            }
        }
    }
}