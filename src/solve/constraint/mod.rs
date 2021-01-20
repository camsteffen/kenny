use enum_dispatch::enum_dispatch;

use super::markup::PuzzleMarkupChanges;
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
use crate::square::Square;

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

macro_rules! constraint_list {
    ($($name:ident),* $(,)?) => {
        const CONSTRAINT_COUNT: usize = count_tts::count_tts!($($name)*);

        enum_dispatch! {
            #[derive(Clone)]
            pub(crate) enum ConstraintItem<'a>: Constraint {
                $($name<'a>),*
            }
        }

        pub(crate) fn init_constraints(puzzle: &Puzzle) -> ConstraintList<'_> {
            ConstraintList(Box::new([$($name::new(puzzle).into()),*]))
        }
    }
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

#[derive(Clone)]
pub(crate) struct ConstraintList<'a>(Box<[ConstraintItem<'a>; CONSTRAINT_COUNT]>);

impl<'a> ConstraintList<'a> {
    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, ConstraintItem<'a>> {
        self.0.iter_mut()
    }

    pub fn notify_changes(
        &mut self,
        changes: &PuzzleMarkupChanges,
        cell_variables: &Square<CellVariable>,
    ) {
        for c in self.0.deref_mut() {
            c.notify_changes(changes, cell_variables);
        }
    }
}
