use super::Constraint;
use crate::collections::square::Square;
use crate::collections::LinkedAHashSet;
use crate::puzzle::solve::cage_solutions::CageSolutions;
use crate::puzzle::solve::markup::{PuzzleMarkup, PuzzleMarkupChanges};
use crate::puzzle::solve::CellVariable;
use crate::puzzle::solve::ValueSet;
use crate::puzzle::Puzzle;
use crate::puzzle::{CageId, CageRef, Operator};

/// A cell domain value must have at least one corresponding cage solution value
#[derive(Clone)]
pub struct CellCageSolutionConstraint {
    dirty_cages: LinkedAHashSet<CageId>,
}

impl CellCageSolutionConstraint {
    pub fn new(puzzle: &Puzzle) -> Self {
        Self {
            dirty_cages: puzzle
                .cages()
                .filter(|cage| cage.operator() != Operator::Nop)
                .map(CageRef::id)
                .collect(),
        }
    }
}

impl Constraint for CellCageSolutionConstraint {
    fn notify_changes(&mut self, puzzle: &Puzzle, changes: &PuzzleMarkupChanges) {
        for (id, _) in changes.cell_domain_removals() {
            self.dirty_cages.insert(puzzle.cell(id).cage_id());
        }
        for &cage_id in changes.cage_solution_removals.keys() {
            self.dirty_cages.insert(cage_id);
        }
    }

    fn enforce_partial(
        &mut self,
        puzzle: &Puzzle,
        markup: &PuzzleMarkup,
        changes: &mut PuzzleMarkupChanges,
    ) -> bool {
        while let Some(cage_id) = self.dirty_cages.pop_front() {
            let cage_solutions = &markup.cage_solutions().unwrap()[cage_id];
            let count = enforce_cage(puzzle, &markup.cells(), cage_solutions, changes);
            if count > 0 {
                return true;
            }
        }
        false
    }
}

fn enforce_cage(
    puzzle: &Puzzle,
    cell_variables: &Square<CellVariable>,
    cage_solutions: &CageSolutions,
    changes: &mut PuzzleMarkupChanges,
) -> u32 {
    // assemble domain for each unsolved cell from cell solutions
    let mut soln_domain = vec![ValueSet::new(puzzle.width()); cage_solutions.num_cells()];
    for solution in &cage_solutions.solutions {
        for i in 0..cage_solutions.num_cells() {
            soln_domain[i].insert(solution[i]);
        }
    }

    let mut count = 0;

    // remove values from cell domains that are not in a cage solution
    for (i, id) in cage_solutions.cell_ids.iter().copied().enumerate() {
        let domain = cell_variables[id].unsolved().unwrap();
        let no_solutions = domain
            .iter()
            .filter(|&n| !soln_domain[i].contains(n))
            .collect::<Vec<_>>();
        if no_solutions.is_empty() {
            continue;
        }
        count += no_solutions.len() as u32;
        debug!(
            "values {:?} in cell {:?} are not part of a cage solution",
            no_solutions,
            puzzle.cell(id).coord()
        );
        for n in no_solutions {
            changes.remove_value_from_cell(id, n);
        }
    }
    count
}
