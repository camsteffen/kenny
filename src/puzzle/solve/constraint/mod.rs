use enum_dispatch::enum_dispatch;

use super::markup::PuzzleMarkupChanges;
use crate::collections::square::Square;
use crate::puzzle::solve::constraint::cage_solution_outer_cell_domain::CageSolutionOuterCellDomainConstraint;
use crate::puzzle::solve::constraint::cage_vector_value::CageVectorValueConstraint;
use crate::puzzle::solve::constraint::cell_cage_solution::CellCageSolutionConstraint;
use crate::puzzle::solve::constraint::vector_preemptive_set::VectorPreemptiveSetConstraint;
use crate::puzzle::solve::constraint::vector_solved_cell::VectorSolvedCellConstraint;
use crate::puzzle::solve::constraint::vector_value_cage::VectorValueCageConstraint;
use crate::puzzle::solve::constraint::vector_value_domain::VectorValueDomainConstraint;
use crate::puzzle::solve::markup::PuzzleMarkup;
use crate::puzzle::solve::CellVariable;
use crate::puzzle::Puzzle;

pub(crate) use self::unary_constraints::apply_unary_constraints;

mod cage_solution_outer_cell_domain;
mod cage_vector_value;
mod cell_cage_solution;
mod unary_constraints;
mod vector_preemptive_set;
mod vector_solved_cell;
mod vector_value_cage;
mod vector_value_domain;

#[enum_dispatch]
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

// contribute to library? - https://gitlab.com/antonok/enum_dispatch/-/issues/25
macro_rules! enum_dispatch {
    {
        $(#[$meta:meta])*
        $vis:vis enum $name:ident$(<$($lt:lifetime),+>)?: $trait:ident $(+ $add_trait:ident)* {
            $($ty:ident$(<$($item_lt:lifetime)+>)?),*$(,)?
        }
    } => {
        #[enum_dispatch($trait$(, $add_trait)*)]
        $(#[$meta])*
        $vis enum $name$(<$($lt),+>)? {
            $($ty($ty$(<$($item_lt)*>)?),)*
        }
    }
}

enum_dispatch! {
    #[derive(Clone)]
    pub(crate) enum ConstraintItem<'a>: Constraint {
        VectorSolvedCellConstraint<'a>,
        VectorValueDomainConstraint<'a>,
        CellCageSolutionConstraint<'a>,
        CageVectorValueConstraint<'a>,
        VectorPreemptiveSetConstraint<'a>,
        VectorValueCageConstraint<'a>,
        CageSolutionOuterCellDomainConstraint<'a>,
    }
}

#[derive(Clone)]
pub(crate) struct ConstraintList<'a>([ConstraintItem<'a>; 7]);

pub(crate) fn init_constraints(puzzle: &Puzzle) -> ConstraintList<'_> {
    ConstraintList([
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
    ])
}

impl<'a> ConstraintList<'a> {
    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, ConstraintItem<'a>> {
        self.0.iter_mut()
    }

    pub fn notify_changes(
        &mut self,
        changes: &PuzzleMarkupChanges,
        cell_variables: &Square<CellVariable>,
    ) {
        for c in &mut self.0 {
            c.notify_changes(changes, cell_variables);
        }
    }
}
