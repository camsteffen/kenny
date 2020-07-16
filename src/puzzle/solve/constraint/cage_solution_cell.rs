use std::collections::hash_map::Entry;

use vec_map::VecMap;

use crate::collections::square::{IsSquare, Square};
use crate::puzzle::solve::constraint::Constraint;
use crate::puzzle::solve::markup::{CellChange, PuzzleMarkup, PuzzleMarkupChanges};
use crate::puzzle::solve::CellVariable;
use crate::puzzle::{CellId, Puzzle};
use crate::HashMap;

// Every cage solution must have corresponding values in cell domains
#[derive(Clone)]
pub(crate) struct CageSolutionCellConstraint<'a> {
    puzzle: &'a Puzzle,
    // cage ID -> cell ID -> cell change
    cage_cell_changes: VecMap<HashMap<CellId, CellChange>>,
}

impl<'a> CageSolutionCellConstraint<'a> {
    pub fn new(puzzle: &'a Puzzle) -> Self {
        Self {
            puzzle,
            cage_cell_changes: VecMap::with_capacity(puzzle.cage_count()),
        }
    }
}

impl<'a> Constraint<'a> for CageSolutionCellConstraint<'a> {
    fn notify_changes(
        &mut self,
        changes: &PuzzleMarkupChanges,
        _cell_variables: &Square<CellVariable>,
    ) {
        for (&id, change) in &changes.cells {
            let cell_map = self
                .cage_cell_changes
                .entry(self.puzzle.cell(id).cage_id())
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
        markup: &PuzzleMarkup<'_>,
        changes: &mut PuzzleMarkupChanges,
    ) -> bool {
        let mut changed = false;
        let mut processed_cages = Vec::new();
        for (cage_id, cell_map) in &self.cage_cell_changes {
            let mut removed_count = 0;
            let cage_solutions = &markup.cage_solutions().unwrap()[cage_id];
            for (i, values) in cage_solutions.solutions.iter().enumerate() {
                for (cell_id, change) in cell_map {
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
                            self.puzzle.cage(cage_id).coord(),
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
                    self.puzzle.cage(cage_id).coord()
                );
            }
        }
        for cage_id in processed_cages {
            self.cage_cell_changes.remove(cage_id);
        }
        changed
    }
}
