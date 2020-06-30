use anyhow::Result;

use crate::puzzle::solve::constraint::cage_solution_cell::CageSolutionCellConstraint;
use crate::puzzle::solve::constraint::cage_solution_outer_cell_domain::CageSolutionOuterCellDomainConstraint;
use crate::puzzle::solve::constraint::cage_vector_value::CageVectorValueConstraint;
use crate::puzzle::solve::constraint::cell_cage_solution::CellCageSolutionConstraint;
use crate::puzzle::solve::constraint::vector_preemptive_set::VectorPreemptiveSetConstraint;
use crate::puzzle::solve::constraint::vector_solved_cell::VectorSolvedCellConstraint;
use crate::puzzle::solve::constraint::vector_value_cage::VectorValueCageConstraint;
use crate::puzzle::solve::constraint::vector_value_domain::VectorValueDomainConstraint;
use crate::puzzle::solve::constraint::Constraint;
use crate::puzzle::solve::markup::{PuzzleMarkup, PuzzleMarkupChanges};
use crate::puzzle::solve::step_writer::StepWriter;
use crate::puzzle::{Puzzle, Solution};

#[derive(Clone)]
pub(crate) struct ConstraintSet<'a> {
    puzzle: &'a Puzzle,
    constraints: Vec<Box<dyn Constraint<'a>>>,
}

impl<'a> ConstraintSet<'a> {
    pub fn new(puzzle: &'a Puzzle) -> Self {
        Self {
            puzzle,
            constraints: init_constraints(puzzle),
        }
    }

    pub fn notify_changes(&mut self, changes: &PuzzleMarkupChanges) {
        for c in &mut self.constraints {
            c.notify_changes(changes);
        }
    }

    pub fn propagate(
        &mut self,
        markup: &mut PuzzleMarkup<'_>,
        step_writer: &mut Option<&mut StepWriter<'_>>,
    ) -> Result<PropagateResult> {
        let mut changes = PuzzleMarkupChanges::default();
        let mut loop_count = 0;
        loop {
            let has_changes = self
                .constraints
                .iter_mut()
                .any(|constraint| constraint.enforce_partial(markup, &mut changes));
            if !has_changes {
                break;
            }
            if let Some(step_writer) = step_writer.as_mut() {
                if !changes.cells.is_empty() {
                    step_writer.write_step(markup, &changes.cells)?;
                }
            }
            if !markup.sync_changes(&mut changes) {
                return Ok(PropagateResult::Invalid);
            }
            self.notify_changes(&changes);
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
                if self.puzzle.verify_solution(&values) {
                    PropagateResult::Solved(values)
                } else {
                    PropagateResult::Invalid
                }
            }
        };
        Ok(result)
    }
}

pub enum PropagateResult {
    Solved(Solution),
    Unsolved,
    Invalid,
}

fn init_constraints(puzzle: &Puzzle) -> Vec<Box<dyn Constraint<'_> + '_>> {
    vec![
        // when a cell is solved, remove the value from other cells in the same vector
        Box::new(VectorSolvedCellConstraint::new(puzzle)),
        // if a vector has only one cell with a given value, solve the cell
        Box::new(VectorValueDomainConstraint::new(puzzle)),
        // If no cage solutions have a value in a cell's domain,
        // remove the cell domain value
        Box::new(CellCageSolutionConstraint::new(puzzle)),
        // When a cell's domain is reduced, remove cage solutions
        Box::new(CageSolutionCellConstraint::new(puzzle)),
        // If all cage solutions for a cage have a value in a vector,
        // remove the value from other cells in the vector
        Box::new(CageVectorValueConstraint::new(puzzle)),
        // Find a set of cells in a vector that must contain a set of values
        Box::new(VectorPreemptiveSetConstraint::new(puzzle)),
        // If, within a vector, a value is known to be in a certain cage,
        // remove cage solutions without the value in the vector
        Box::new(VectorValueCageConstraint::new(puzzle)),
        // Remove cage solutions that conflict with a cell's entire domain outside of the cage
        Box::new(CageSolutionOuterCellDomainConstraint::new(puzzle)),
    ]
}
