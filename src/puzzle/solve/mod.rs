//! solve KenKen puzzles

pub(crate) use self::cell_variable::CellVariable;
pub(crate) use self::value_set::ValueSet;

use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;

use crate::puzzle::solve::step_writer::{StepWriter, StepWriterBuilder};
use crate::puzzle::{Puzzle, Solution};

use self::constraint::apply_unary_constraints;
use self::markup::{PuzzleMarkup, PuzzleMarkupChanges};
use crate::puzzle::solve::constraint::{ConstraintSet, PropagateResult};
use crate::puzzle::solve::search::{search_solution, SearchResult};

mod cage_solutions;
mod cell_variable;
mod constraint;
pub(crate) mod markup;
mod search;
mod step_writer;
mod value_set;

pub enum SolveResult {
    /// The puzzle cannot be solved - there may be an error in the puzzle
    Unsolvable,
    /// The puzzle was solved and has exactly one solution, as it should
    Solved(SolvedData),
    /// Multiple solutions were found for the puzzle - this is not a proper puzzle
    MultipleSolutions,
}

impl SolveResult {
    pub fn is_solved(&self) -> bool {
        matches!(self, SolveResult::Solved(_))
    }

    pub fn solved(&self) -> Option<&SolvedData> {
        match self {
            SolveResult::Solved(data) => Some(data),
            _ => None,
        }
    }
}

pub struct SolvedData {
    pub solution: Solution,
    pub used_search: bool,
}

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

    pub fn solve(&self) -> Result<SolveResult> {
        let mut changes = PuzzleMarkupChanges::default();
        apply_unary_constraints(self.puzzle, &mut changes.cells);
        let mut markup = PuzzleMarkup::new(self.puzzle);
        let mut step_writer = self.start_step_writer();
        if let Some(ref mut step_writer) = step_writer {
            step_writer.write_step(&markup, &changes.cells)?;
        }
        let solvable = markup.sync_changes(&mut changes);
        debug_assert!(solvable);
        markup.init_cage_solutions(self.puzzle);
        let mut constraints = ConstraintSet::new(self.puzzle);
        constraints.notify_changes(&changes);
        let solution = match constraints.propagate(&mut markup, &mut step_writer.as_mut())? {
            PropagateResult::Solved(solution) => Some(solution),
            PropagateResult::Unsolved => None,
            PropagateResult::Invalid => return Ok(SolveResult::Unsolvable),
        };
        let result = if let Some(solution) = solution {
            SolvedData {
                solution,
                used_search: false,
            }
        } else {
            info!("Begin backtracking");
            let solution = match search_solution(
                self.puzzle,
                &markup,
                &constraints,
                &mut step_writer.as_mut(),
            )? {
                SearchResult::NoSolutions => return Ok(SolveResult::Unsolvable),
                SearchResult::SingleSolution(s) => s.solution,
                SearchResult::MultipleSolutions => return Ok(SolveResult::MultipleSolutions),
            };
            SolvedData {
                solution,
                used_search: true,
            }
        };
        debug_assert!(self.puzzle.verify_solution(&result.solution));
        Ok(SolveResult::Solved(result))
    }

    fn start_step_writer(&self) -> Option<StepWriter<'_>> {
        let steps = match self.steps {
            None => return None,
            Some(ref steps) => steps,
        };
        let mut builder = StepWriterBuilder::new(self.puzzle, &steps.path);
        if let Some(image_width) = steps.image_width {
            builder.image_width(image_width);
        }
        Some(builder.build())
    }
}

struct StepsContext {
    image_width: Option<u32>,
    path: PathBuf,
}
