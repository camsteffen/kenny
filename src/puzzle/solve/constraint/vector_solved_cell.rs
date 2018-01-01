use collections::Square;
use collections::square::SquareIndex;
use puzzle::Puzzle;
use puzzle::solve::CellVariable;
use puzzle::solve::markup::PuzzleMarkupChanges;
use puzzle::solve::PuzzleMarkup;
use super::Constraint;

pub struct VectorSolvedCellConstraint {
    solved_cells: Vec<SquareIndex>,
}

impl VectorSolvedCellConstraint {

    pub fn new() -> Self {
        Self {
            solved_cells: Vec::new(),
        }
    }

    pub fn enforce_solved_cell(&mut self, cell_variables: &Square<CellVariable>, index: SquareIndex, value: i32, changes: &mut PuzzleMarkupChanges) -> u32 {
        let puzzle_width = cell_variables.width();
        let surrounding_indices = index.intersecting_vectors(puzzle_width).to_vec().into_iter()
            .flat_map(|v| v.iter_sq_pos(puzzle_width))
            .filter(|&i| cell_variables[i].unsolved_and_contains(value));
        let mut count = 0;
        for i in surrounding_indices {
            changes.remove_value_from_cell(i, value);
            count += 1;
        }
        debug!("removed {} instances of {} surrounding solved cell at {:?}", count, value,
            index.as_coord(puzzle_width));
        count
    }
}

impl Constraint for VectorSolvedCellConstraint {
    
    fn enforce_partial(&mut self, _: &Puzzle, markup: &PuzzleMarkup, changes: &mut PuzzleMarkupChanges) -> bool {
        while let Some(index) = self.solved_cells.pop() {
            let value = markup.cell_variables[index].unwrap_solved();
            let count = self.enforce_solved_cell(&markup.cell_variables, index, value, changes);
            if count > 0 { return true }
        }
        false
    }

    fn notify_changes(&mut self, changes: &PuzzleMarkupChanges) {
        for &(index, _) in &changes.cell_solutions {
            self.solved_cells.push(index);
        }
    }
}
