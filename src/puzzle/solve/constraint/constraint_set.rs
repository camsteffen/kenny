use failure::Fallible;

use crate::puzzle::{Puzzle, Solution};
use crate::puzzle::solve::constraint::cage_solution_outer_cell_domain::CageSolutionOuterCellDomainConstraint;
use crate::puzzle::solve::constraint::cell_cage_solution::CellCageSolutionConstraint;
use crate::puzzle::solve::constraint::cage_vector_value::CageVectorValueConstraint;
use crate::puzzle::solve::constraint::Constraint;
use crate::puzzle::solve::constraint::vector_solved_cell::VectorSolvedCellConstraint;
use crate::puzzle::solve::constraint::vector_subdomain::VectorSubdomainConstraint;
use crate::puzzle::solve::constraint::vector_value_cage::VectorValueCageConstraint;
use crate::puzzle::solve::constraint::vector_value_domain::VectorValueDomainConstraint;
use crate::puzzle::solve::markup::PuzzleMarkupChanges;
use crate::puzzle::solve::PuzzleMarkup;
use crate::puzzle::solve::step_writer::StepWriter;

#[derive(Clone)]
pub struct ConstraintSet<'a> {
    puzzle: &'a Puzzle,
    constraints: Vec<Box<dyn Constraint>>,
}

impl<'a> ConstraintSet<'a> {
    pub fn new(puzzle: &'a Puzzle) -> Self {
        Self { puzzle, constraints: init_constraints(puzzle) }
    }

    pub fn notify_changes(&mut self, changes: &PuzzleMarkupChanges) {
        for c in &mut self.constraints {
            c.notify_changes(self.puzzle, changes);
        }
    }

    pub fn propagate(
        &mut self,
        markup: &mut PuzzleMarkup,
        step_writer: &mut Option<&mut StepWriter<'_>>
    ) -> Fallible<PropagateResult> {
        let mut changes = PuzzleMarkupChanges::default();
        let mut loop_count = 0;
        loop {
            let ConstraintSet { puzzle, constraints } = self;
            let has_changes = constraints.iter_mut().any(|constraint|
                constraint.enforce_partial(puzzle, markup, &mut changes));
            if !has_changes { break }
            markup.sync_changes(self.puzzle, &mut changes);
            self.notify_changes(&changes);
            if let Some(step_writer) = step_writer.as_mut() {
                let changed_cells: Vec<_> = changes.cell_domain_value_removals.keys().copied().collect();
                step_writer.write_next(markup, &changed_cells)?;
            }
            changes.clear();
            loop_count += 1;
            if markup.is_completed() { break }
        }
        debug!("constraint propagation finished after {} iterations, solved={}",
               loop_count, markup.is_completed());
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

fn init_constraints(puzzle: &Puzzle) -> Vec<Box<dyn Constraint>> {
    vec![
        Box::new(VectorSolvedCellConstraint::new()),
        Box::new(VectorValueDomainConstraint::new(puzzle.width())),
        Box::new(CellCageSolutionConstraint::new(puzzle)),
        Box::new(CageVectorValueConstraint::new(puzzle)),
        Box::new(VectorSubdomainConstraint::new()),
        Box::new(VectorValueCageConstraint::new(puzzle)),
        Box::new(CageSolutionOuterCellDomainConstraint::new(puzzle)),
    ]
}
