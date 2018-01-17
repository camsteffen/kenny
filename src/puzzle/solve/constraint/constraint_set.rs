use puzzle::Puzzle;
use super::CageSolutionsConstraint;
use super::CageVectorValueConstraint;
use super::Constraint;
use super::VectorSubdomainConstraint;
use super::VectorSolvedCellConstraint;
use super::VectorValueDomainConstraint;
use super::CageSolutionVectorDomainConstraint;
use super::VectorValueCageConstraint;

pub struct ConstraintSet {
    vector_solved_cell: VectorSolvedCellConstraint,
    vector_value_domain: VectorValueDomainConstraint,
    cage_solutions: CageSolutionsConstraint,
    cage_vector_value: CageVectorValueConstraint,
    vector_subdomain: VectorSubdomainConstraint,
    vector_value_cage: VectorValueCageConstraint,
    cage_solution_vector_domain: CageSolutionVectorDomainConstraint,
}

impl ConstraintSet {
    pub fn new(puzzle: &Puzzle) -> Self {
        Self {
            vector_solved_cell: VectorSolvedCellConstraint::new(),
            vector_value_domain: VectorValueDomainConstraint::new(puzzle.width),
            cage_solutions: CageSolutionsConstraint::new(puzzle),
            cage_vector_value: CageVectorValueConstraint::new(puzzle),
            vector_subdomain: VectorSubdomainConstraint::new(),
            vector_value_cage: VectorValueCageConstraint::new(puzzle),
            cage_solution_vector_domain: CageSolutionVectorDomainConstraint::new(puzzle),
        }
    }

    pub fn len() -> u32 { 7 }

    pub fn select_map<F, T>(&mut self, index: u32, mut f: F) -> T where F: FnMut(&mut Constraint) -> T {
        match index {
            0 => f(&mut self.vector_solved_cell),
            1 => f(&mut self.vector_value_domain),
            2 => f(&mut self.cage_solutions),
            3 => f(&mut self.cage_vector_value),
            4 => f(&mut self.vector_subdomain),
            5 => f(&mut self.vector_value_cage),
            6 => f(&mut self.cage_solution_vector_domain),
            _ => panic!("invalid index"),
        }
    }

    pub fn for_each<F: Fn(&mut Constraint) -> ()>(&mut self, f: F) {
        f(&mut self.vector_solved_cell);
        f(&mut self.vector_value_domain);
        f(&mut self.cage_solutions);
        f(&mut self.cage_vector_value);
        f(&mut self.vector_subdomain);
        f(&mut self.vector_value_cage);
        f(&mut self.cage_solution_vector_domain);
    }
}
