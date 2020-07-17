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

pub(crate) use self::constraint_set::{ConstraintSet, PropagateResult};
pub(crate) use self::unary_constraints::apply_unary_constraints;

mod cage_solution_outer_cell_domain;
mod cage_vector_value;
mod cell_cage_solution;
mod constraint_set;
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

macro_rules! enum_dispatch {
    {
        $(#[$meta:meta])*
        enum $name:ident$(<$($lt:lifetime),+>)?: $trait:ty {
            $($ty:ident$(<$($item_lt:lifetime)+>)?),*$(,)?
        }
    } => {
        #[enum_dispatch($trait)]
        $(#[$meta])*
        enum $name$(<$($lt),+>)? {
            $($ty($ty$(<$($item_lt)*>)?),)*
        }
    }
}

enum_dispatch! {
    #[derive(Clone)]
    enum ConstraintItem<'a>: Constraint {
        VectorSolvedCellConstraint<'a>,
        VectorValueDomainConstraint<'a>,
        CellCageSolutionConstraint<'a>,
        CageVectorValueConstraint<'a>,
        VectorPreemptiveSetConstraint<'a>,
        VectorValueCageConstraint<'a>,
        CageSolutionOuterCellDomainConstraint<'a>,
    }
}
