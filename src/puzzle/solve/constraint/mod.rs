mod constraint_propogation;
mod cage_solution_vector_domain;
mod cage_solutions;
mod cage_vector_value;
mod constraint_set;
mod unary_constraints;
mod vector_solved_cell;
mod vector_value_domain;
mod vector_subdomain;

//pub use self::constraint_propogation::ConstraintPropogation;
pub use self::unary_constraints::apply_unary_constraints;
pub use self::constraint_propogation::constraint_propogation;

use self::constraint_set::ConstraintSet;
use self::cage_solutions::CageSolutionsConstraint;
use self::cage_vector_value::CageVectorValueConstraint;
use self::vector_subdomain::VectorSubdomainConstraint;
use self::vector_value_domain::VectorValueDomainConstraint;
use self::vector_solved_cell::VectorSolvedCellConstraint;
use super::markup::PuzzleMarkupChanges;
use puzzle::Puzzle;
use puzzle::solve::PuzzleMarkup;

pub trait Constraint {
    fn enforce_partial(&mut self, puzzle: &Puzzle, markup: &PuzzleMarkup, changes: &mut PuzzleMarkupChanges) -> bool;
    fn notify_changes(&mut self, changes: &PuzzleMarkupChanges);
}
