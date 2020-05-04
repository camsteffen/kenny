//! solve KenKen puzzles

use std::path::Path;
use std::path::PathBuf;

use failure::Fallible;

use crate::puzzle::{Puzzle, Solution};
use crate::puzzle::solve::step_writer::{StepWriterBuilder, StepWriter};

pub use self::value_set::ValueSet;
pub use self::cell_variable::CellVariable;
use self::constraint::apply_unary_constraints;
pub use self::markup::PuzzleMarkup;
pub use self::markup::PuzzleMarkupChanges;
use crate::puzzle::solve::search::{search_solution, SearchResult};
use crate::puzzle::solve::constraint::{ConstraintSet, PropagateResult};

mod cage_solutions;
mod search;
mod value_set;
mod cell_variable;
mod constraint;
mod markup;
mod step_writer;

pub enum SolveResult {
    /// The puzzle cannot be solved - there may be an error in the puzzle
    Unsolvable,
    /// The puzzle was solved and has exactly one solution, as it should
    Solved(Solution),
    /// Multiple solutions were found for the puzzle - this is not a proper puzzle
    MultipleSolutions,
}

// todo refactor, rename with "context"? Add more fields?
pub struct PuzzleSolver<'a> {
    puzzle: &'a Puzzle,
    steps: Option<StepsContext>,
}

impl<'a> PuzzleSolver<'a> {
    pub fn new(puzzle: &'a Puzzle) -> Self {
        Self {
            puzzle,
            steps: None,
        }
    }

    pub fn save_steps(&mut self, path: &Path) -> &mut Self {
        let steps_context = StepsContext {
            image_width: None,
            path: path.to_path_buf(),
        };
        self.steps = Some(steps_context);
        self
    }

    pub fn steps_image_width(&mut self, image_width: u32) -> &mut Self {
        self.steps.as_mut().unwrap().image_width = Some(image_width);
        self
    }

    pub fn solve(&self) -> Fallible<SolveResult> {
        let mut changes = PuzzleMarkupChanges::default();
        apply_unary_constraints(self.puzzle, &mut changes);
        let mut markup = PuzzleMarkup::new(self.puzzle);
        markup.sync_changes(self.puzzle, &mut changes);
        let mut step_writer = self.start_step_writer(&mut markup)?;
        markup.init_cage_solutions(self.puzzle);
        let mut constraints = ConstraintSet::new(self.puzzle);
        constraints.notify_changes(&changes);
        let solution = match constraints.propagate(&mut markup, &mut step_writer.as_mut())? {
            PropagateResult::Solved(solution) => Some(solution),
            PropagateResult::Unsolved => None,
            PropagateResult::Invalid => return Ok(SolveResult::Unsolvable),
        };
        let solution = match solution {
            Some(solution) => solution,
            None => {
                info!("Begin backtracking");
                match search_solution(self.puzzle, &markup, &mut constraints, &mut step_writer.as_mut())? {
                    SearchResult::NoSolutions => return Ok(SolveResult::Unsolvable),
                    SearchResult::SingleSolution(s) => s.solution,
                    SearchResult::MultipleSolutions => return Ok(SolveResult::MultipleSolutions),
                }
            }
        };
        debug_assert!(self.puzzle.verify_solution(&solution));
        Ok(SolveResult::Solved(solution))
    }

    fn start_step_writer(&self, markup: &mut PuzzleMarkup) -> Fallible<Option<StepWriter>> {
        self.steps.as_ref().map(|steps| -> Fallible<StepWriter> {
            let mut builder = StepWriterBuilder::new(self.puzzle, &steps.path);
            if let Some(image_width) = steps.image_width {
                builder.image_width(image_width);
            }
            let mut step_writer = builder.build();
            step_writer.write_next(&markup, &[])?;
            Ok(step_writer)
        }).transpose()
    }
}

struct StepsContext {
    image_width: Option<u32>,
    path: PathBuf,
}
