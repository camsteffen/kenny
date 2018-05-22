mod cage_solution_vector_domain;
mod cage_solutions;
mod cage_vector_value;
mod constraint_propagation;
mod unary_constraints;
mod vector_solved_cell;
mod vector_subdomain;
mod vector_value_cage;
mod vector_value_domain;

//pub use self::constraint_propagation::ConstraintPropogation;
pub use self::unary_constraints::apply_unary_constraints;
pub use self::constraint_propagation::constraint_propagation;

use self::cage_solution_vector_domain::CageSolutionVectorDomainConstraint;
use self::cage_solutions::CageSolutionsConstraint;
use self::cage_vector_value::CageVectorValueConstraint;
use self::vector_subdomain::VectorSubdomainConstraint;
use self::vector_value_domain::VectorValueDomainConstraint;
use self::vector_value_cage::VectorValueCageConstraint;
use self::vector_solved_cell::VectorSolvedCellConstraint;
use super::markup::PuzzleMarkupChanges;
use puzzle::Puzzle;
use puzzle::solve::PuzzleMarkup;

pub trait Constraint {
    fn enforce_partial(&mut self, puzzle: &Puzzle, markup: &PuzzleMarkup, changes: &mut PuzzleMarkupChanges) -> bool;
    fn notify_changes(&mut self, puzzle: &Puzzle, changes: &PuzzleMarkupChanges);
}
