use super::CageVectorValuesConstraint;
use super::CageSolutionsConstraint;
use puzzle::Puzzle;
use collections::Square;
use solve::CellDomain;

pub fn apply_constraint_propogation(puzzle: &Puzzle, cell_domains: &mut Square<CellDomain>) {
    let constraint_propogation = ConstraintPropogation::new(puzzle.size);
    constraint_propogation.apply(puzzle, cell_domains);
}

pub struct ConstraintPropogation {
    cage_solutions: CageSolutionsConstraint,
    cage_vector_values: CageVectorValuesConstraint,
}

impl ConstraintPropogation {
    pub fn new(size: usize) -> Self {
        Self {
            cage_solutions: CageSolutionsConstraint::new(size),
            cage_vector_values: CageVectorValuesConstraint::new(),
        }
    }

    fn apply(&mut self, puzzle: &Puzzle, cell_domains: &mut Square<CellDomain>) {

    }
}