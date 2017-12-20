use collections::square::{SquareIndex, VectorId};
use fnv::FnvHashMap;
use fnv::FnvHashSet;
use solve::DomainChangeSet;
use std::collections::HashSet;

/// A record of values known to be in a certain cage, in a certain vector
pub struct CageVectorValuesConstraint {
    data: FnvHashMap<VectorId, FnvHashSet<i32>>,
}

impl CageVectorValuesConstraint {
    pub fn new() -> Self {
        Self {
            data: FnvHashMap::default(),
        }
    }

    pub fn notify_cage_domain_value_removed(&mut self, index: SquareIndex) {

    }

    pub fn enforce(&self, change: DomainChangeSet) {
        let cage_index = cage_index as usize;
        let cage = &self.puzzle.cages[cage_index];
        let mut unsolved = cage.cells.iter()
            .cloned()
            .filter(|&pos| self.cells[pos].is_unsolved())
            .collect_vec();
        
        let mut vectors = BTreeMap::new();
        for ((i1, &p1), (i2, &p2)) in unsolved.iter().enumerate().tuple_combinations() {
            if let Some(vector_id) = shared_vector(p1, p2, self.size()) {
                let vector_positions = vectors.entry(vector_id).or_insert_with(BTreeSet::new);
                vector_positions.insert(i1);
                vector_positions.insert(i2);
            }
        }
        let vectors = vectors.into_iter()
            .map(|(vector_id, unsolved_indices)| {
                (vector_id, unsolved_indices.into_iter().collect_vec())
            });

        for (vector_id, unsolved_indices) in vectors {
            self.find_vector_values(cage_index, vector_id, &unsolved_indices, &solutions);
        }
    }

    fn find_vector_values(&mut self,
                          cage_index: usize,
                          vector_id: VectorId,
                          unsolved_indices: &[usize],
                          solutions: &[Vec<i32>])
    {
        let mut values: HashSet<i32>;
        {
            let cage = &mut self.cages[cage_index];
            let mut solutions_iter = solutions.iter();
            let solution = solutions_iter.next().unwrap();
            values = unsolved_indices.iter()
                .map(|&unsolved_id| solution[unsolved_id])
                .filter(|n| {
                    cage.vector_vals.get(&vector_id)
                        .map_or(true, |vals| !vals.contains(n))
                })
                .collect();
            for solution in solutions_iter {
                if values.is_empty() { break }
                values = unsolved_indices.iter()
                    .map(|&unsolved_id| solution[unsolved_id])
                    .filter(|n| values.contains(n))
                    .collect();
            }

            cage.vector_vals.entry(vector_id)
                .or_insert_with(HashSet::new)
                .extend(&values);
        }

        let remove_from = vector_id.iter_sq_pos(self.size())
            .filter(|&pos| self.cage_map[pos] != cage_index)
            .collect_vec();
        for n in values {
            debug!("value {} exists in cage at {}, in {}", n, self.cage_first_coord(cage_index), vector_id);

            for &pos in &remove_from {
                self.remove_from_cell_domain_and_solve(pos, n, true);
            }
        }
    }
}