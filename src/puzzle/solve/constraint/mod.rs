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
use crate::puzzle::Puzzle;

pub(crate) trait Constraint: CloneConstraint {
    /// Notifies this constraint of changes made to the puzzle markup.
    /// This should be a low-cost method where data is cached for later processing.
    fn notify_changes(&mut self, puzzle: &Puzzle, changes: &PuzzleMarkupChanges);

    /// Partially enforces this constraint on the current puzzle. The constraint will be checked until some
    /// changes are found and added to `changes`. Returns `false` if no changes are found.
    fn enforce_partial(
        &mut self,
        puzzle: &Puzzle,
        markup: &PuzzleMarkup,
        changes: &mut PuzzleMarkupChanges,
    ) -> bool;

    // todo is this needed? default implementation?
    // fn state(&self) -> State {
    //     State::PENDING
    // }
}

/*
pub enum State {
    /// There is pending work to determine if this constraint is satisfied.
    PENDING,
    /// An inconsistency has been detected with this constraint on the puzzle markup.
    /// In other words, there are puzzle markup changes ready.
    INCONSISTENT,
    /// The constraint is satisfied (with the last seen puzzle markup). No pending work.
    SATISFIED,
    // todo unsatisfied?
}
 */

pub(crate) trait CloneConstraint {
    fn clone_constraint(&self) -> Box<dyn Constraint>;
}

impl<T> CloneConstraint for T
where
    T: 'static + Constraint + Clone,
{
    fn clone_constraint(&self) -> Box<dyn Constraint> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Constraint> {
    fn clone(&self) -> Self {
        self.clone_constraint()
    }
}
