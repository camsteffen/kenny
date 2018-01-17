//! If all possible solutions for a given value in a given vector are in a given cage, then the cage solution must
//! contain the given value in the given vector

use collections::FnvLinkedHashMap;
use collections::FnvLinkedHashSet;
use collections::square::VectorId;
use fnv::FnvHashMap;
use fnv::FnvHashSet;
use puzzle::Puzzle;
use puzzle::solve::cage_solutions::CageSolutions;
use puzzle::solve::constraint::Constraint;
use puzzle::solve::markup::PuzzleMarkup;
use puzzle::solve::markup::PuzzleMarkupChanges;

pub struct VectorValueCageConstraint {
    dirty_vector_values: FnvLinkedHashSet<(VectorId, i32)>,
    known_vector_value_cages: Vec<(VectorId, i32, usize)>,
    vector_value_cages: FnvHashMap<(VectorId, i32), FnvLinkedHashMap<usize, u32>>,
    vector_value_domain_sizes: FnvHashMap<(VectorId, i32), u32>,
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
        let vector_value_domain_sizes = (0..puzzle.width * 2).map(|i| VectorId(i)).flat_map(|vector_id|
            (1..puzzle.width as i32).map(move |value| ((vector_id, value), puzzle.width as u32))).collect();
        Self {
            dirty_vector_values: FnvLinkedHashSet::default(),
            known_vector_value_cages: Vec::new(),
            vector_value_cages,
            vector_value_domain_sizes,
        }
    }
}

impl Constraint for VectorValueCageConstraint {

    fn enforce_partial(&mut self, puzzle: &Puzzle, markup: &PuzzleMarkup, changes: &mut PuzzleMarkupChanges) -> bool {
        while let Some((vector, value, cage)) = self.known_vector_value_cages.pop() {
            let CageSolutions { indices, solutions, .. } = &markup.cage_solutions_set[cage];
            let indices_in_vector = indices.iter().enumerate().filter_map(|(i, &sq_idx)| {
                if vector.contains_sq_index(sq_idx, puzzle.width) { Some(i) } else { None }
            }).collect::<Vec<_>>();
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
                    /*
                    let domain_size = self.vector_value_domain_sizes.get_mut(&key).unwrap();
                    *domain_size -= 1;
                    let domain_size = *domain_size;
                    if domain_size > 1 {
                        self.dirty_vector_values.insert(key);
                    } else if domain_size == 1 {
                        self.dirty_vector_values.remove(&key);
                    }
                    */
                    let cages = self.vector_value_cages.get_mut(&key).unwrap();
                    let cage_empty = {
                        let cage_count = cages.get_mut(&cage).unwrap();
                        *cage_count -= 1;
                        *cage_count == 0
                    };
                    if cage_empty {
                        cages.remove(&cage);
                        if cages.len() == 1 {
                            self.known_vector_value_cages.push((vector_id, value, cage));
                        }
                    }

                }
            }
        }
    }
}