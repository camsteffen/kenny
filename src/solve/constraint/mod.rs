mod constraint_propogation;
mod cage_solutions;
mod cage_vector_values;
mod unary_constraints;
mod vector_value;

pub use self::constraint_propogation::ConstraintPropogation;
pub use self::unary_constraints::apply_unary_constraints;

use self::cage_solutions::CageSolutionsConstraint;
use self::cage_vector_values::CageVectorValuesConstraint;

trait Constraint {
}
