pub(crate) use self::constraint_set::{ConstraintSet, PropagateResult};
pub(crate) use self::unary_constraints::apply_unary_constraints;

mod cage_solution_cell;
mod cage_solution_outer_cell_domain;
mod cage_vector_value;
mod cell_cage_solution;
mod constraint_set;
mod unary_constraints;
mod vector_preemptive_set;
mod vector_solved_cell;
mod vector_value_cage;
mod vector_value_domain;

use super::markup::PuzzleMarkupChanges;
use crate::puzzle::solve::markup::PuzzleMarkup;

pub(crate) trait Constraint<'a>: CloneConstraint<'a> {
    /// Notifies this constraint of changes made to the puzzle markup.
    /// This should be a low-cost method where data is cached for later processing.
    fn notify_changes(&mut self, changes: &PuzzleMarkupChanges);

    /// Partially enforces this constraint on the current puzzle. The constraint will be checked until some
    /// changes are found and added to `changes`. Returns `false` if no changes are found.
    fn enforce_partial(
        &mut self,
        markup: &PuzzleMarkup<'_>,
        changes: &mut PuzzleMarkupChanges,
    ) -> bool;
}

pub(crate) trait CloneConstraint<'a>: 'a {
    fn clone_constraint(&self) -> Box<dyn Constraint<'a>>;
}

impl<'a, T> CloneConstraint<'a> for T
where
    T: Constraint<'a> + Clone,
{
    fn clone_constraint(&self) -> Box<dyn Constraint<'a>> {
        Box::new(self.clone())
    }
}

impl<'a> Clone for Box<dyn Constraint<'a>> {
    fn clone(&self) -> Self {
        self.clone_constraint()
    }
}
