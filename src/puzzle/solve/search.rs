use failure::Fallible;

use crate::Puzzle;
use crate::puzzle::{Value, CellId, Solution};
use crate::puzzle::solve::{PuzzleMarkup, PuzzleMarkupChanges};
use crate::puzzle::solve::constraint::PropagateResult;

use super::constraint::ConstraintSet;
use crate::puzzle::solve::step_writer::StepWriter;

pub enum SearchResult {
    NoSolutions,
    SingleSolution(SingleSolution),
    MultipleSolutions,
}

pub struct SingleSolution {
    pub solution: Solution,
    pub depth: u32,
}

pub fn search_solution(
    puzzle: &Puzzle,
    markup: &PuzzleMarkup,
    constraints: &ConstraintSet<'_>,
    step_writer: &mut Option<&mut StepWriter<'_>>,
) -> Fallible<SearchResult> {
    search_next(1, puzzle, markup, constraints, step_writer, false)
}

fn search_next(
    depth: u32,
    puzzle: &Puzzle,
    markup: &PuzzleMarkup,
    constraints: &ConstraintSet<'_>,
    step_writer: &mut Option<&mut StepWriter<'_>>,
    mut solved_once: bool,
) -> Fallible<SearchResult> {
    debug!("Backtracking (depth={})", depth);
    let mut solution = None;
    for (i, guess) in guesses(markup).enumerate() {
        debug!("Guessing with {} at {:?}, guess #: {}",
               guess.value, puzzle.cell(guess.cell_id).coord(), i + 1);
        let result = guess_cell(puzzle, markup.clone(), constraints.clone(), step_writer, guess, depth, solved_once)?;
        match &result {
            SearchResult::NoSolutions => {
                debug!("Guess failed")
            },
            SearchResult::SingleSolution(_) => {
                if solved_once {
                    return Ok(SearchResult::MultipleSolutions)
                }
                solution = Some(result);
                solved_once = true;
            },
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
) -> Fallible<SearchResult> {
    apply_guess(puzzle, guess, &mut markup, &mut constraints);
    if let Some(ref mut step_writer) = step_writer { step_writer.write_backtrack(&markup, &[guess.cell_id], depth)?; }
    match constraints.propagate(&mut markup, step_writer)? {
        PropagateResult::Solved(solution) => {
            return Ok(SearchResult::SingleSolution(SingleSolution { solution, depth }));
        },
        PropagateResult::Unsolved => (),
        PropagateResult::Invalid => {
            return Ok(SearchResult::NoSolutions);
        },
    };
    // recursive next backtracking call
    search_next(depth + 1, puzzle, &mut markup, &mut constraints, step_writer, solved_once)
}

fn guesses<'a>(markup: &'a PuzzleMarkup) -> impl Iterator<Item=Guess> + 'a {
    // find one of the cells with the smallest domain
    let (cell_id, domain) = markup.cells().iter().enumerate()
        .filter_map(|(i, cell)| cell.unsolved().map(|domain| (i, domain)))
        .min_by_key(|(_, domain)| domain.len())
        .expect("No unsolved cells");
    // guess every value in the domain
    domain.iter().map(move |value| Guess { cell_id, value })
}

fn apply_guess(puzzle: &Puzzle, guess: Guess, markup: &mut PuzzleMarkup, constraints: &mut ConstraintSet<'_>) {
    let mut changes = PuzzleMarkupChanges::default();
    changes.solve_cell(guess.cell_id, guess.value);
    markup.sync_changes(puzzle, &mut changes);
    constraints.notify_changes(&changes)
}

#[derive(Clone, Copy)]
struct Guess {
    cell_id: CellId,
    value: Value,
}
