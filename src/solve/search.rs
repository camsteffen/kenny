use anyhow::Result;

use crate::collections::square::IsSquare;
use crate::puzzle::{CellId, Puzzle, Solution};
use crate::solve::constraint::{Constraint, ConstraintList};
use crate::solve::markup::{PuzzleMarkup, PuzzleMarkupChanges};
use crate::solve::step_writer::StepWriter;
use crate::solve::{propagate_constraints, PropagateResult};

pub(crate) enum SearchResult {
    NoSolutions,
    SingleSolution(Solution),
    MultipleSolutions,
}

struct SearchContext<'a, 'b> {
    puzzle: &'a Puzzle,
    step_writer: &'a mut Option<&'b mut StepWriter<'b>>,
    stack: Vec<SearchStackFrame<'b>>,
}

#[derive(Clone)]
struct SearchStackFrame<'a> {
    markup: PuzzleMarkup<'a>,
    constraints: ConstraintList<'a>,
    solved_once: bool,
    guesses: Option<Guesses>,
}

#[derive(Clone)]
struct Guesses {
    cell_id: CellId,
    index: usize,
}

pub(crate) fn search_solution<'a>(
    puzzle: &Puzzle,
    markup: PuzzleMarkup<'a>,
    constraints: ConstraintList<'a>,
    step_writer: &mut Option<&'a mut StepWriter<'a>>,
) -> Result<SearchResult> {
    SearchContext {
        puzzle,
        stack: vec![SearchStackFrame {
            markup,
            constraints,
            solved_once: false,
            guesses: None,
        }],
        step_writer,
    }
    .search()
}

impl SearchContext<'_, '_> {
    fn search(&mut self) -> Result<SearchResult> {
        debug!("Backtracking (depth={})", self.stack.len());
        let mut solution = None;
        if let Some(ref mut step_writer) = self.step_writer {
            step_writer.start_search_branch();
        }
        loop {
            let frame = match self.stack.last_mut() {
                None => break,
                Some(frame) => frame,
            };
            let guesses = frame.guesses.get_or_insert_with(|| {
                let cell_id = pick_cell_to_guess(&frame.markup);
                Guesses { cell_id, index: 0 }
            });
            let domain = &frame.markup.cells()[guesses.cell_id].unsolved().unwrap();
            let value = domain.iter().nth(guesses.index);
            let value = match value {
                None => {
                    self.stack.pop();
                    continue;
                }
                Some(value) => value,
            };
            guesses.index += 1;
            if let Some(ref mut step_writer) = self.step_writer {
                step_writer.next_search_branch();
            }
            debug!(
                "Guessing with {} at {:?}, guess #: {}",
                value,
                self.puzzle.cell(guesses.cell_id).coord(),
                guesses.index,
            );
            let mut changes = PuzzleMarkupChanges::default();
            changes.cells.solve(guesses.cell_id, value);
            if !frame.markup.sync_changes(&mut changes) {
                debug!("Guess failed");
                continue;
            }
            if let Some(ref mut step_writer) = self.step_writer {
                step_writer.write_step(&frame.markup, &changes.cells)?;
            }
            let next_frame = SearchStackFrame {
                guesses: None,
                ..frame.clone()
            };
            self.stack.push(next_frame);
            let frame = self.stack.last_mut().unwrap();
            frame
                .constraints
                .notify_changes(&changes, frame.markup.cells());
            frame.markup.apply_changes(&changes);
            match propagate_constraints(
                self.puzzle,
                &mut frame.constraints,
                &mut frame.markup,
                self.step_writer,
            )? {
                PropagateResult::Solved(p_solution) => {
                    self.stack.pop().unwrap();
                    let frame = self.stack.last_mut().unwrap();
                    if frame.solved_once {
                        return Ok(SearchResult::MultipleSolutions);
                    }
                    solution = Some(p_solution);
                    frame.solved_once = true;
                }
                PropagateResult::Unsolved => {}
                PropagateResult::Invalid => {
                    debug!("Guess failed");
                    self.stack.pop().unwrap();
                }
            }
        }
        if let Some(ref mut step_writer) = self.step_writer {
            step_writer.end_search_branch();
        }
        Ok(solution.map_or(SearchResult::NoSolutions, |s| {
            SearchResult::SingleSolution(s)
        }))
    }
}

fn pick_cell_to_guess(markup: &PuzzleMarkup<'_>) -> CellId {
    // find one of the cells with the smallest domain
    markup
        .cells()
        .iter()
        .enumerate()
        .filter_map(|(i, cell)| cell.unsolved().map(|domain| (i, domain)))
        .min_by_key(|(_, domain)| domain.len())
        .expect("No unsolved cells")
        .0
}
