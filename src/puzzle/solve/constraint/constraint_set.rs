use puzzle::Puzzle;
use super::CageSolutionsConstraint;
use super::CageVectorValueConstraint;
use super::Constraint;
use super::VectorSubdomainConstraint;
use super::VectorSolvedCellConstraint;
use super::VectorValueDomainConstraint;

pub struct ConstraintSet {
    vector_solved_cell: VectorSolvedCellConstraint,
    vector_value_domain: VectorValueDomainConstraint,
    cage_solutions: CageSolutionsConstraint,
    cage_vector_value: CageVectorValueConstraint,
    vector_subdomain: VectorSubdomainConstraint,
}

impl ConstraintSet {
    pub fn new(puzzle: &Puzzle) -> Self {
        let vector_solved_cell = VectorSolvedCellConstraint::new();
        let vector_value_domain = VectorValueDomainConstraint::new(puzzle.width);
        let cage_solutions = CageSolutionsConstraint::new(puzzle);
        let cage_vector_value = CageVectorValueConstraint::new(puzzle);
        let vector_subdomain = VectorSubdomainConstraint::new(puzzle.width);
        Self {
            vector_solved_cell,
            vector_value_domain,
            cage_solutions,
            cage_vector_value,
            vector_subdomain,
        }
    }

    pub fn len() -> u32 { 5 }

    pub fn select_map<F, T>(&mut self, index: u32, mut f: F) -> T where F: FnMut(&mut Constraint) -> T {
        match index {
            0 => f(&mut self.vector_solved_cell),
            1 => f(&mut self.vector_value_domain),
            2 => f(&mut self.cage_solutions),
            3 => f(&mut self.cage_vector_value),
            4 => f(&mut self.vector_subdomain),
            _ => panic!("invalid index"),
        }
    }

    pub fn for_each<F: Fn(&mut Constraint) -> ()>(&mut self, f: F) {
        f(&mut self.vector_solved_cell);
        f(&mut self.vector_value_domain);
        f(&mut self.cage_solutions);
        f(&mut self.cage_vector_value);
        f(&mut self.vector_subdomain);
    }
}
