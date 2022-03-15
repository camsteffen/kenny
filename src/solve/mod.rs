//! solve KenKen puzzles

use std::path::Path;
use std::path::PathBuf;

use anyhow::Result;

use self::constraint::apply_unary_constraints;
use self::markup::{PuzzleMarkup, PuzzleMarkupChanges};
use crate::puzzle::{Puzzle, Solution};
use crate::solve::constraint::{Constraint, ConstraintList};
use crate::solve::search::{search_solution, SearchResult};
use crate::solve::step_writer::StepWriter;

pub(crate) use self::cell_variable::CellVariable;
pub(crate) use self::value_set::ValueSet;

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
    steps_path: Option<PathBuf>,
}

impl<'a> PuzzleSolver<'a> {
    pub fn new(puzzle: &'a Puzzle) -> Self {
        Self {
            puzzle,
            steps_path: None,
        }
    }

    pub fn save_steps(&mut self, path: &Path) -> &mut Self {
        self.steps_path = Some(path.into());
        self
    }

    pub fn solve(&self) -> Result<SolveResult> {
        let mut changes = PuzzleMarkupChanges::default();
        apply_unary_constraints(self.puzzle, &mut changes.cells);
        let mut markup = PuzzleMarkup::new(self.puzzle);
        let solvable = markup.sync_changes(&mut changes);
        debug_assert!(solvable);
        let mut step_writer = self.start_step_writer();
        if let Some(ref mut step_writer) = step_writer {
            step_writer.write_step(&markup, &changes.cells)?;
        }
        markup.init_cage_solutions(self.puzzle);
        let mut constraints = ConstraintList::new(self.puzzle);
        constraints.notify_changes(&changes, markup.cells());
        markup.apply_changes(&changes);
        let solution = match propagate_constraints(
            self.puzzle,
            &mut constraints,
            &mut markup,
            &mut step_writer.as_mut(),
        )? {
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
            let solution =
                match search_solution(self.puzzle, markup, constraints, &mut step_writer.as_mut())?
                {
                    SearchResult::NoSolutions => return Ok(SolveResult::Unsolvable),
                    SearchResult::SingleSolution(solution) => solution,
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
        let path = self.steps_path.as_ref()?;
        let step_writer = StepWriter::new(self.puzzle, path.into());
        Some(step_writer)
    }
}

pub(crate) fn propagate_constraints(
    puzzle: &Puzzle,
    constraints: &mut ConstraintList<'_>,
    markup: &mut PuzzleMarkup<'_>,
    step_writer: &mut Option<&mut StepWriter<'_>>,
) -> Result<PropagateResult> {
    let mut changes = PuzzleMarkupChanges::default();
    let mut loop_count = 0;
    loop {
        let has_changes = constraints.enforce_partial(markup, &mut changes);
        if !has_changes {
            break;
        }
        if !markup.sync_changes(&mut changes) {
            return Ok(PropagateResult::Invalid);
        }
        debug!("Changes: {:?}", changes);
        if let Some(step_writer) = step_writer.as_mut() {
            if !changes.cells.is_empty() {
                step_writer.write_step(markup, &changes.cells)?;
            }
        }
        constraints.notify_changes(&changes, markup.cells());
        markup.apply_changes(&changes);
        changes.clear();
        loop_count += 1;
        if markup.is_completed() {
            break;
        }
    }
    debug!(
        "constraint propagation finished after {} iterations, solved={}",
        loop_count,
        markup.is_completed()
    );
    let result = match markup.completed_values() {
        None => PropagateResult::Unsolved,
        Some(values) => {
            if puzzle.verify_solution(&values) {
                PropagateResult::Solved(values)
            } else {
                PropagateResult::Invalid
            }
        }
    };
    Ok(result)
}

pub(crate) enum PropagateResult {
    Solved(Solution),
    Unsolved,
    Invalid,
}
