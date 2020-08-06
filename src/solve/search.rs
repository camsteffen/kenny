use std::borrow::Cow;

use anyhow::Result;

use crate::puzzle::{CellId, Puzzle, Solution, Value};
use crate::solve::constraint::ConstraintList;
use crate::solve::markup::{CellChanges, PuzzleMarkup, PuzzleMarkupChanges};
use crate::solve::step_writer::StepWriter;
use crate::solve::{propagate_constraints, PropagateResult};
use crate::square::IsSquare;

pub(crate) enum SearchResult {
    NoSolutions,
    SingleSolution(SingleSolution),
    MultipleSolutions,
}

pub(crate) struct SingleSolution {
    pub solution: Solution,
    pub depth: u32,
}

struct SearchContext<'a, 'b> {
    puzzle: &'a Puzzle,
    markup: Cow<'a, PuzzleMarkup<'b>>,
    constraints: Cow<'a, ConstraintList<'b>>,
    step_writer: &'a mut Option<&'b mut StepWriter<'b>>,
    solved_once: bool,
    depth: u32,
}

pub(crate) fn search_solution<'a>(
    puzzle: &Puzzle,
    markup: &PuzzleMarkup<'a>,
    constraints: &ConstraintList<'a>,
    step_writer: &mut Option<&'a mut StepWriter<'a>>,
) -> Result<SearchResult> {
    SearchContext {
        puzzle,
        markup: Cow::Borrowed(markup),
        constraints: Cow::Borrowed(constraints),
        step_writer,
        solved_once: false,
        depth: 0,
    }
    .search_next()
}

impl SearchContext<'_, '_> {
    fn search_next(&mut self) -> Result<SearchResult> {
        let next_depth = self.depth + 1;
        debug!("Backtracking (depth={})", next_depth);
        let mut solution = None;
        if let Some(ref mut step_writer) = self.step_writer {
            step_writer.start_search_branch();
        }
        for (i, guess) in guesses(self.markup.as_ref()).enumerate() {
            if let Some(ref mut step_writer) = self.step_writer {
                step_writer.next_search_branch();
            }
            debug!(
                "Guessing with {} at {:?}, guess #: {}",
                guess.value,
                self.puzzle.cell(guess.cell_id).coord(),
                i + 1
            );
            let mut context = SearchContext {
                puzzle: self.puzzle,
                markup: Cow::Borrowed(&self.markup),
                constraints: Cow::Borrowed(&self.constraints),
                step_writer: self.step_writer,
                solved_once: self.solved_once,
                depth: next_depth,
            };
            match context.guess_cell(guess)? {
                SearchResult::NoSolutions => debug!("Guess failed"),
                SearchResult::SingleSolution(ss) => {
                    if self.solved_once {
                        return Ok(SearchResult::MultipleSolutions);
                    }
                    solution = Some(ss);
                    self.solved_once = true;
                }
                SearchResult::MultipleSolutions => return Ok(SearchResult::MultipleSolutions),
            }
        }
        if let Some(ref mut step_writer) = self.step_writer {
            step_writer.end_search_branch();
        }
        Ok(solution.map_or(SearchResult::NoSolutions, |s| {
            SearchResult::SingleSolution(s)
        }))
    }

    fn guess_cell(&mut self, guess: Guess) -> Result<SearchResult> {
        let mut changes = PuzzleMarkupChanges::default();
        changes.cells.solve(guess.cell_id, guess.value);
        if !self.markup.sync_changes(&mut changes) {
            return Ok(SearchResult::NoSolutions);
        }
        if let Some(ref mut step_writer) = self.step_writer {
            let mut changes = CellChanges::new();
            changes.solve(guess.cell_id, guess.value);
            step_writer.write_step(&self.markup, &changes)?;
        }
        let (markup, constraints) = (self.markup.to_mut(), self.constraints.to_mut());
        constraints.notify_changes(&changes, markup.cells());
        markup.apply_changes(&changes);
        match propagate_constraints(self.puzzle, constraints, markup, self.step_writer)? {
            PropagateResult::Solved(solution) => {
                return Ok(SearchResult::SingleSolution(SingleSolution {
                    solution,
                    depth: self.depth,
                }));
            }
            PropagateResult::Unsolved => (),
            PropagateResult::Invalid => {
                return Ok(SearchResult::NoSolutions);
            }
        };
        // recursive next backtracking call
        self.search_next()
    }
}

fn guesses<'a>(markup: &'a PuzzleMarkup<'_>) -> impl Iterator<Item = Guess> + 'a {
    // find one of the cells with the smallest domain
    let (cell_id, domain) = markup
        .cells()
        .iter()
        .enumerate()
        .filter_map(|(i, cell)| cell.unsolved().map(|domain| (i, domain)))
        .min_by_key(|(_, domain)| domain.len())
        .expect("No unsolved cells");
    // guess every value in the domain
    domain.iter().map(move |value| Guess { cell_id, value })
}

#[derive(Clone, Copy)]
struct Guess {
    cell_id: CellId,
    value: Value,
}
