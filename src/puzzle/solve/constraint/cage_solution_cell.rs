use crate::puzzle::solve::constraint::Constraint;
use crate::puzzle::solve::markup::{CellChange, PuzzleMarkup, PuzzleMarkupChanges};
use crate::puzzle::{CellId, Puzzle};
use ahash::AHashMap;
use std::collections::hash_map::Entry;
use vec_map::VecMap;

// Every cage solution must have corresponding values in cell domains
#[derive(Clone)]
pub struct CageSolutionCellConstraint {
    // cage ID -> cell ID -> cell change
    cage_cell_changes: VecMap<AHashMap<CellId, CellChange>>,
}

impl CageSolutionCellConstraint {
    pub fn new(puzzle: &Puzzle) -> Self {
        Self {
            cage_cell_changes: VecMap::with_capacity(puzzle.cage_count()),
        }
    }
}

impl Constraint for CageSolutionCellConstraint {
    fn notify_changes(&mut self, puzzle: &Puzzle, changes: &PuzzleMarkupChanges) {
        for (&id, change) in changes.cells.iter() {
            let cell_map = self
                .cage_cell_changes
                .entry(puzzle.cell(id).cage_id())
                .or_default();
            match change {
                CellChange::DomainRemovals(values) => match cell_map.entry(id) {
                    Entry::Occupied(mut entry) => {
                        if let CellChange::DomainRemovals(existing_values) = entry.get_mut() {
                            existing_values.extend(values);
                        }
                    }
                    Entry::Vacant(entry) => {
                        entry.insert((*change).clone());
                    }
                },
                CellChange::Solution(_) => {
                    cell_map.insert(id, (*change).clone());
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
        let mut changed = false;
        let mut processed_cages = Vec::new();
        for (cage_id, cell_map) in self.cage_cell_changes.iter() {
            let mut removed_count = 0;
            let cage_solutions = &markup.cage_solutions().unwrap()[cage_id];
            for (i, values) in cage_solutions.solutions.iter().enumerate() {
                for (cell_id, change) in cell_map.iter() {
                    let idx = match cage_solutions.index_map.get(&cell_id) {
                        Some(&idx) => idx,
                        None => continue,
                    };
                    let solution_value = values[idx];
                    let keep = match change {
                        CellChange::DomainRemovals(values) => !values.contains(&solution_value),
                        &CellChange::Solution(value) => value == solution_value,
                    };
                    if !keep {
                        debug!(
                            "Removing cage solution {:?} for cage at {:?} because of {:?} at {}",
                            &values,
                            puzzle.cage(cage_id).coord(),
                            change,
                            cell_id
                        );
                        changes.remove_cage_solution(cage_id, i);
                        removed_count += 1;
                        changed = true;
                    }
                }
            }
            processed_cages.push(cage_id);
            if removed_count > 0 {
                debug!(
                    "Removed {} cage solutions from cage at {:?}",
                    removed_count,
                    puzzle.cage(cage_id).coord()
                );
            }
        }
        for cage_id in processed_cages {
            self.cage_cell_changes.remove(cage_id);
        }
        changed
    }
}
