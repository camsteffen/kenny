use crate::collections::FnvLinkedHashSet;
use crate::collections::Square;
use crate::puzzle::Operator;
use crate::puzzle::Puzzle;
use crate::puzzle::solve::CellDomain;
use crate::puzzle::solve::CellVariable;
use crate::puzzle::solve::markup::PuzzleMarkupChanges;
use crate::puzzle::solve::PuzzleMarkup;
use super::Constraint;
use crate::puzzle::solve::cage_solutions::CageSolutions;

/// Ensures that for every value in a cell domain, there is a possible solution of the corresponding cage
/// with the value in that cell
pub struct CageSolutionsConstraint {
    dirty_cages: FnvLinkedHashSet<usize>,
}

impl CageSolutionsConstraint {
    pub fn new(puzzle: &Puzzle) -> Self {
        Self {
            dirty_cages: puzzle.cages().iter()
                .filter_map(|cage| if cage.operator() == Operator::Nop { None } else { Some(cage.index()) })
                .collect(),
        }
    }

    fn enforce_cage(puzzle: &Puzzle, cell_variables: &Square<CellVariable>, cage_solutions: &CageSolutions, changes: &mut PuzzleMarkupChanges) -> u32 {
        // assemble domain for each unsolved cell from cell solutions
        let mut soln_domain = vec![CellDomain::new(puzzle.width()); cage_solutions.num_cells()];
        for solution in &cage_solutions.solutions {
            for i in 0..cage_solutions.num_cells() {
                soln_domain[i].insert(solution[i]);
            }
        }

        let mut count = 0;

        // remove values from cell domains that are not in a cage solution
        for (i, sq_index) in cage_solutions.indices.iter().cloned().enumerate() {
            let domain = cell_variables[sq_index].unwrap_unsolved();
            let no_solutions = domain.iter()
                    .filter(|&n| !soln_domain[i].contains(n))
                    .collect::<Vec<_>>();
            if no_solutions.is_empty() { continue }
            count += no_solutions.len() as u32;
            debug!("values {:?} in cell {:?} are not part of a solution",
                   no_solutions, puzzle.cell(sq_index).coord());
            for n in no_solutions {
                changes.remove_value_from_cell(sq_index, n);
            }
        }
        count
    }
}

impl Constraint for CageSolutionsConstraint {
    fn notify_changes(&mut self, puzzle: &Puzzle, changes: &PuzzleMarkupChanges) {
        for &index in changes.cell_domain_value_removals.keys() {
            self.dirty_cages.insert(puzzle.cell(index).cage().index());
        }
        for &cage_index in changes.cage_solution_removals.keys() {
            self.dirty_cages.insert(cage_index);
        }
    }

    fn enforce_partial(&mut self, puzzle: &Puzzle, markup: &PuzzleMarkup, changes: &mut PuzzleMarkupChanges) -> bool {
        while let Some(cage_index) = self.dirty_cages.pop_front() {
            let cage_solutions = &markup.cage_solutions()[cage_index];
            let count = Self::enforce_cage(puzzle, &markup.cells(), cage_solutions, changes);
            if count > 0 { return true }
        }
        false
    }
}
