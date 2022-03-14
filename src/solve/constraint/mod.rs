use super::markup::PuzzleMarkupChanges;
use crate::collections::square::Square;
use crate::puzzle::Puzzle;
use crate::solve::constraint::cage_solution::CageSolutionConstraint;
use crate::solve::constraint::cage_solution_outer_cell_domain::CageSolutionOuterCellDomainConstraint;
use crate::solve::constraint::cage_vector_value::CageVectorValueConstraint;
use crate::solve::constraint::cell_cage_solution::CellCageSolutionConstraint;
use crate::solve::constraint::vector_preemptive_set::VectorPreemptiveSetConstraint;
use crate::solve::constraint::vector_solved_cell::VectorSolvedCellConstraint;
use crate::solve::constraint::vector_value_cage::VectorValueCageConstraint;
use crate::solve::constraint::vector_value_domain::VectorValueDomainConstraint;
use crate::solve::markup::PuzzleMarkup;
use crate::solve::CellVariable;

pub(crate) use self::unary_constraints::apply_unary_constraints;

mod cage_solution;
mod cage_solution_outer_cell_domain;
mod cage_vector_value;
mod cell_cage_solution;
mod unary_constraints;
mod vector_preemptive_set;
mod vector_solved_cell;
mod vector_value_cage;
mod vector_value_domain;

pub(crate) trait Constraint {
    /// Notifies this constraint of changes made to the puzzle markup.
    /// This should be a low-cost method where data is cached for later processing.
    fn notify_changes(
        &mut self,
        changes: &PuzzleMarkupChanges,
        cell_variables: &Square<CellVariable>,
    );

    /// Partially enforces this constraint on the current puzzle. The constraint will be checked until some
    /// changes are found and added to `changes`. Returns `false` if no changes are found.
    fn enforce_partial(
        &mut self,
        markup: &PuzzleMarkup<'_>,
        changes: &mut PuzzleMarkupChanges,
    ) -> bool;
}

/// Defines `ConstraintList` which combines all the individual `Constraint`s and implements
/// `Constraint` with static dispatch.
macro_rules! constraint_list {
    ($($name:ident,)*) => {
        #[derive(Clone)]
        #[allow(non_snake_case)]
        pub(crate) struct ConstraintList<'a> {
            $($name: $name<'a>,)*
        }

        impl<'a> ConstraintList<'a> {
            pub fn new(puzzle: &'a Puzzle) -> Self {
                Self {
                    $($name: $name::new(puzzle),)*
                }
            }
        }

        impl Constraint for ConstraintList<'_> {
            fn notify_changes(
                &mut self,
                changes: &PuzzleMarkupChanges,
                cell_variables: &Square<CellVariable>,
            ) {
                $(self.$name.notify_changes(changes, cell_variables);)*
            }

            fn enforce_partial(
                &mut self,
                markup: &PuzzleMarkup<'_>,
                changes: &mut PuzzleMarkupChanges,
            ) -> bool {
                $(self.$name.enforce_partial(markup, changes))||*
            }
        }
    };
}

constraint_list! {
    // when a cell is solved, remove the value from other cells in the same vector
    VectorSolvedCellConstraint,
    // if one cage solution remains for a cage, solve the cage
    CageSolutionConstraint,
    // if a vector has only one cell with a given value, solve the cell
    VectorValueDomainConstraint,
    // If no cage solutions have a value in a cell's domain,
    // remove the cell domain value
    CellCageSolutionConstraint,
    // If all cage solutions for a cage have a value in a vector,
    // remove the value from other cells in the vector
    CageVectorValueConstraint,
    // Find a set of cells in a vector that must contain a set of values
    VectorPreemptiveSetConstraint,
    // If, within a vector, a value is known to be in a certain cage,
    // remove cage solutions without the value in the vector
    VectorValueCageConstraint,
    // Remove cage solutions that conflict with a cell's entire domain outside of the cage
    CageSolutionOuterCellDomainConstraint,
}
