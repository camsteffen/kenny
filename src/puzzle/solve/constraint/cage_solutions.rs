use collections::FnvLinkedHashSet;
use collections::Square;
use puzzle::Operator;
use puzzle::Puzzle;
use puzzle::solve::CageSolutions;
use puzzle::solve::CellDomain;
use puzzle::solve::CellVariable;
use puzzle::solve::markup::PuzzleMarkupChanges;
use puzzle::solve::PuzzleMarkup;
use super::Constraint;

/// Ensures that for every value in a cell domain, there is a possible solution of the corresponding cage
/// with the value in that cell
pub struct CageSolutionsConstraint {
    puzzle_width: usize,
    cage_map: Square<usize>,
    dirty_cages: FnvLinkedHashSet<usize>,
}

impl CageSolutionsConstraint {
    pub fn new(puzzle: &Puzzle) -> Self {
        Self {
            puzzle_width: puzzle.width,
            cage_map: puzzle.cage_map.clone(),
            dirty_cages: puzzle.cages.iter().enumerate()
                .filter_map(|(i, cage)| if cage.operator != Operator::Nop { Some(i) } else { None })
                .collect(),
        }
    }

    fn enforce_cage(&self, cell_variables: &Square<CellVariable>, cage_solutions: &CageSolutions, changes: &mut PuzzleMarkupChanges) -> u32 {
        // assemble domain for each unsolved cell from cell solutions
        let mut soln_domain = vec![CellDomain::new(self.puzzle_width); cage_solutions.num_cells()];
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
            debug!("values {:?} in cell {:?} are not part of a solution", no_solutions,
                sq_index.as_coord(self.puzzle_width));
            for n in no_solutions {
                changes.remove_value_from_cell(sq_index, n);
            }
        }
        count
    }
}

impl Constraint for CageSolutionsConstraint {
    fn enforce_partial(&mut self, _: &Puzzle, markup: &PuzzleMarkup, changes: &mut PuzzleMarkupChanges) -> bool {
        while let Some(cage_index) = self.dirty_cages.pop_front() {
            let cage_solutions = &markup.cage_solutions_set[cage_index];
            let count = self.enforce_cage(&markup.cell_variables, cage_solutions, changes);
            if count > 0 { return true }
        }
        false
    }

    fn notify_changes(&mut self, changes: &PuzzleMarkupChanges) {
        for &(index, _) in &changes.cell_domain_value_removals {
            let cage_index = self.cage_map[index];
            self.dirty_cages.insert(cage_index);
        }
    }
}
