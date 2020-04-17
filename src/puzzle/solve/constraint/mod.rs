mod cage_solution_vector_domain;
mod cage_solutions;
mod cage_vector_value;
mod constraint_propagation;
mod unary_constraints;
mod vector_solved_cell;
mod vector_subdomain;
mod vector_value_cage;
mod vector_value_domain;

pub use self::unary_constraints::apply_unary_constraints;
pub use self::constraint_propagation::constraint_propagation;

use super::markup::PuzzleMarkupChanges;
use crate::puzzle::Puzzle;
use crate::puzzle::solve::PuzzleMarkup;

pub trait Constraint {

    /// Notifies this constraint of changes made to the puzzle markup.
    /// This should be a low-cost method where data is cached for later processing.
    fn notify_changes(&mut self, puzzle: &Puzzle, changes: &PuzzleMarkupChanges);

    /// Partially enforces this constraint on the current puzzle. The constraint will be checked until one unit of
    /// change is found and added to `changes`. Returns `false` if no changes are found.
    fn enforce_partial(&mut self, puzzle: &Puzzle, markup: &PuzzleMarkup, changes: &mut PuzzleMarkupChanges) -> bool;
}
