use anyhow::Result;

use super::constraint::ConstraintSet;
use crate::puzzle::solve::constraint::PropagateResult;
use crate::puzzle::solve::markup::{CellChanges, PuzzleMarkup, PuzzleMarkupChanges};
use crate::puzzle::solve::step_writer::StepWriter;
use crate::puzzle::{CellId, Puzzle, Solution, Value};

pub(crate) enum SearchResult {
    NoSolutions,
    SingleSolution(SingleSolution),
    MultipleSolutions,
}

pub(crate) struct SingleSolution {
    pub solution: Solution,
    pub depth: u32,
}

pub(crate) fn search_solution(
    puzzle: &Puzzle,
    markup: &PuzzleMarkup,
    constraints: &ConstraintSet<'_>,
    step_writer: &mut Option<&mut StepWriter<'_>>,
) -> Result<SearchResult> {
    search_next(1, puzzle, markup, constraints, step_writer, false)
}

fn search_next(
    depth: u32,
    puzzle: &Puzzle,
    markup: &PuzzleMarkup,
    constraints: &ConstraintSet<'_>,
    step_writer: &mut Option<&mut StepWriter<'_>>,
    mut solved_once: bool,
) -> Result<SearchResult> {
    debug!("Backtracking (depth={})", depth);
    let mut solution = None;
    for (i, guess) in guesses(markup).enumerate() {
        debug!(
            "Guessing with {} at {:?}, guess #: {}",
            guess.value,
            puzzle.cell(guess.cell_id).coord(),
            i + 1
        );
        let result = guess_cell(
            puzzle,
            markup.clone(),
            constraints.clone(),
            step_writer,
            guess,
            depth,
            solved_once,
        )?;
        match &result {
            SearchResult::NoSolutions => debug!("Guess failed"),
            SearchResult::SingleSolution(_) => {
                if solved_once {
                    return Ok(SearchResult::MultipleSolutions);
                }
                solution = Some(result);
                solved_once = true;
            }
            SearchResult::MultipleSolutions => return Ok(SearchResult::MultipleSolutions),
        }
    }
    match solution {
        Some(result) => Ok(result),
        None => Ok(SearchResult::NoSolutions),
    }
}

fn guess_cell(
    puzzle: &Puzzle,
    mut markup: PuzzleMarkup,
    mut constraints: ConstraintSet<'_>,
    step_writer: &mut Option<&mut StepWriter<'_>>,
    guess: Guess,
    depth: u32,
    solved_once: bool,
) -> Result<SearchResult> {
    if let Some(ref mut step_writer) = step_writer {
        let mut changes = CellChanges::new();
        changes.solve(guess.cell_id, guess.value);
        step_writer.write_backtrack(&markup, &changes, depth)?;
    }
    apply_guess(puzzle, guess, &mut markup, &mut constraints);
    match constraints.propagate(&mut markup, step_writer)? {
        PropagateResult::Solved(solution) => {
            return Ok(SearchResult::SingleSolution(SingleSolution {
                solution,
                depth,
            }));
        }
        PropagateResult::Unsolved => (),
        PropagateResult::Invalid => {
            return Ok(SearchResult::NoSolutions);
        }
    };
    // recursive next backtracking call
    search_next(
        depth + 1,
        puzzle,
        &markup,
        &constraints,
        step_writer,
        solved_once,
    )
}

fn guesses(markup: &PuzzleMarkup) -> impl Iterator<Item = Guess> + '_ {
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

fn apply_guess(
    puzzle: &Puzzle,
    guess: Guess,
    markup: &mut PuzzleMarkup,
    constraints: &mut ConstraintSet<'_>,
) {
    let mut changes = PuzzleMarkupChanges::default();
    changes.cells.solve(guess.cell_id, guess.value);
    markup.sync_changes(puzzle, &mut changes);
    constraints.notify_changes(&changes)
}

#[derive(Clone, Copy)]
struct Guess {
    cell_id: CellId,
    value: Value,
}
