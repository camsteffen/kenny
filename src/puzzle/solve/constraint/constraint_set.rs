use anyhow::Result;

use crate::collections::square::Square;
use crate::puzzle::solve::constraint::cage_solution_outer_cell_domain::CageSolutionOuterCellDomainConstraint;
use crate::puzzle::solve::constraint::cage_vector_value::CageVectorValueConstraint;
use crate::puzzle::solve::constraint::cell_cage_solution::CellCageSolutionConstraint;
use crate::puzzle::solve::constraint::vector_preemptive_set::VectorPreemptiveSetConstraint;
use crate::puzzle::solve::constraint::vector_solved_cell::VectorSolvedCellConstraint;
use crate::puzzle::solve::constraint::vector_value_cage::VectorValueCageConstraint;
use crate::puzzle::solve::constraint::vector_value_domain::VectorValueDomainConstraint;
use crate::puzzle::solve::constraint::{Constraint, ConstraintItem};
use crate::puzzle::solve::markup::{PuzzleMarkup, PuzzleMarkupChanges};
use crate::puzzle::solve::step_writer::StepWriter;
use crate::puzzle::solve::CellVariable;
use crate::puzzle::{Puzzle, Solution};

#[derive(Clone)]
pub(crate) struct ConstraintSet<'a> {
    puzzle: &'a Puzzle,
    constraints: ConstraintList<'a>,
}

impl<'a> ConstraintSet<'a> {
    pub fn new(puzzle: &'a Puzzle) -> Self {
        Self {
            puzzle,
            constraints: init_constraints(puzzle),
        }
    }

    pub fn notify_changes(
        &mut self,
        changes: &PuzzleMarkupChanges,
        cell_variables: &Square<CellVariable>,
    ) {
        for c in &mut self.constraints {
            c.notify_changes(changes, cell_variables);
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
            if !markup.sync_changes(&mut changes) {
                return Ok(PropagateResult::Invalid);
            }
            if let Some(step_writer) = step_writer.as_mut() {
                if !changes.cells.is_empty() {
                    step_writer.write_step(markup, &changes.cells)?;
                }
            }
            self.notify_changes(&changes, markup.cells());
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

type ConstraintList<'a> = [ConstraintItem<'a>; 7];

fn init_constraints(puzzle: &Puzzle) -> ConstraintList<'_> {
    [
        // when a cell is solved, remove the value from other cells in the same vector
        VectorSolvedCellConstraint::new(puzzle).into(),
        // if a vector has only one cell with a given value, solve the cell
        VectorValueDomainConstraint::new(puzzle).into(),
        // If no cage solutions have a value in a cell's domain,
        // remove the cell domain value
        CellCageSolutionConstraint::new(puzzle).into(),
        // If all cage solutions for a cage have a value in a vector,
        // remove the value from other cells in the vector
        CageVectorValueConstraint::new(puzzle).into(),
        // Find a set of cells in a vector that must contain a set of values
        VectorPreemptiveSetConstraint::new(puzzle).into(),
        // If, within a vector, a value is known to be in a certain cage,
        // remove cage solutions without the value in the vector
        VectorValueCageConstraint::new(puzzle).into(),
        // Remove cage solutions that conflict with a cell's entire domain outside of the cage
        CageSolutionOuterCellDomainConstraint::new(puzzle).into(),
    ]
}
