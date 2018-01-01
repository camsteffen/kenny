mod changes;

pub use self::changes::PuzzleMarkupChanges;

use collections::Square;
use puzzle::solve::CellVariable;
use puzzle::Puzzle;
use puzzle::solve::CageSolutionsSet;

pub struct PuzzleMarkup {
    pub cell_variables: Square<CellVariable>,
    pub cage_solutions_set: CageSolutionsSet,
    puzzle_width: usize,
}

impl PuzzleMarkup {
    pub fn new(puzzle: &Puzzle) -> Self {
        Self {
            cell_variables: Square::with_width_and_value(puzzle.width, CellVariable::unsolved_with_all(puzzle.width)),
            cage_solutions_set: CageSolutionsSet::new(puzzle),
            puzzle_width: puzzle.width,
        }
    }

    pub fn init_cage_solutions(&mut self, puzzle: &Puzzle) {
        self.cage_solutions_set.init(puzzle, &self.cell_variables);
    }

    pub fn sync_changes(&mut self, changes: &mut PuzzleMarkupChanges) {

        // apply cell solutions and collect cell domain removals
        let mut new_cell_domain_removals = Vec::new();
        for &(index, value) in &changes.cell_solutions {
            let cell_variable = &mut self.cell_variables[index];
            {
                let cell_domain = cell_variable.unwrap_unsolved();
                for i in (1..value).chain(value + 1..=self.puzzle_width as i32) {
                    if cell_domain.contains(value) {
                        new_cell_domain_removals.push((index, i));
                    }
                }
            }
            *cell_variable = CellVariable::Solved(value);
        }

        // apply cell domain removals and add any resulting cell solutions to changes
        for &(index, value) in &changes.cell_domain_value_removals {
            if let Some(solution) = self.cell_variables[index].remove_from_domain(value) {
                changes.cell_solutions.push((index, solution));
            }
        }

        // add previosly collected cell domain removals to changes
        changes.cell_domain_value_removals.extend(new_cell_domain_removals);

        self.cage_solutions_set.apply_changes(changes);
    }

/*
    pub fn remove_cell_domain_value(&mut self, index: SquareIndex, cage_index: usize, value: i32) {
        let cage_solutions = &mut self.cage_solutions_set[cage_index];
        let solved = self.cell_variables[index].remove_from_domain(value).is_some();
        cage_solutions.remove_cell_domain_value(index, value);
        if solved {
            cage_solutions.remove_indices(&[index]); // TODO collect list
        }
    }
    */
}
