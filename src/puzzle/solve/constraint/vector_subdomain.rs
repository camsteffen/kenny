use collections::square::VectorId;
use collections::square::Square;
use collections::square::SquareIndex;
use puzzle::solve::CellVariable;
use itertools::Itertools;
use puzzle::solve::CellDomain;
use puzzle::solve::markup::PuzzleMarkupChanges;
use puzzle::Puzzle;
use puzzle::solve::PuzzleMarkup;
use super::Constraint;
use collections::FnvLinkedHashSet;

pub struct VectorSubdomainConstraint {
    dirty_vecs: FnvLinkedHashSet<VectorId>,
}

impl VectorSubdomainConstraint {

    pub fn new() -> Self {
        Self {
            dirty_vecs: FnvLinkedHashSet::default(),
        }
    }

    fn enforce_vector(&self, cell_variables: &Square<CellVariable>, vector_id: VectorId, change: &mut PuzzleMarkupChanges) -> u32 {
        let size = cell_variables.width();

        // organize cells by domain size
        let mut cells_by_domain_size = vec![Vec::new(); size - 3];
        for i in 0..size {
            let sq_pos = vector_id.vec_pos_to_sq_pos(i, size);
            if let Some(domain) = cell_variables[sq_pos].unsolved() {
                let len = domain.len();
                if len < size - 1 {
                    cells_by_domain_size[len - 2].push(sq_pos);
                }
            }
        }

        let mut count = 0;

        // find a set of cells where the collective domain size < the size of the set
        let mut cells: Vec<SquareIndex> = Vec::with_capacity(size - 1);
        'domain_sizes: for (i, cells2) in cells_by_domain_size.into_iter().enumerate().filter(|(_, cells)| !cells.is_empty()) {
            cells = cells.into_iter().merge(cells2).collect();
            let max_size = i + 2;
            'combinations: for cells in cells.iter().cloned().combinations(max_size) {
                let mut domain = CellDomain::new(size as u32);
                for &cell in &cells {
                    for j in cell_variables[cell].unsolved().unwrap() {
                        if domain.insert(j) && domain.len() > max_size {
                            continue 'combinations
                        }
                    }
                }
                debug!("values {:?} are in cells {:?}", domain.iter().collect::<Vec<_>>(),
                    cells.iter().map(|i| i.as_coord(size)).collect::<Vec<_>>());
                let mut iter = cells.iter().cloned();
                let mut cell = iter.next();
                for i in 0..size {
                    let sq_pos = vector_id.vec_pos_to_sq_pos(i, size);
                    let in_group = cell.map_or(false, |i| sq_pos == i);
                    if in_group {
                        cell = iter.next();
                    } else {
                        for val in &domain {
                            if cell_variables[sq_pos].unsolved_and_contains(val) {
                                change.remove_value_from_cell(sq_pos, val);
                                count += 1;
                            }
                        }
                    }
                }
                break 'domain_sizes
            }
        }
        count
    }
}

impl Constraint for VectorSubdomainConstraint {
    fn enforce_partial(&mut self, _: &Puzzle, markup: &PuzzleMarkup, changes: &mut PuzzleMarkupChanges) -> bool {
        while let Some(vector_id) = self.dirty_vecs.front().cloned() {
            let count = self.enforce_vector(&markup.cell_variables, vector_id, changes);
            if count == 0 {
                self.dirty_vecs.pop_front();
            } else {
                return true
            }
        }
        false
    }

    fn notify_changes(&mut self, puzzle: &Puzzle, changes: &PuzzleMarkupChanges) {
        for &index in changes.cell_domain_value_removals.keys() {
            for &vector_id in &index.intersecting_vectors(puzzle.width as usize) {
                self.dirty_vecs.insert(vector_id);
            }
        }
    }
}
