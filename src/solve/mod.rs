mod cell_domain;
mod cell_variable;
mod constraint;
mod domain_change;
mod solver;
mod state_writer;
mod vector_value_domain;

pub use self::cell_domain::CellDomain;
pub use self::cell_variable::CellVariable;
pub use self::domain_change::DomainChangeSet;
pub use self::solver::Solver;

use puzzle::Puzzle;
use collections::Square;
use self::constraint::apply_unary_constraints;

fn solve_puzzle(puzzle: &Puzzle) {
    let cell_domains = Square::with_width_and_value(puzzle.size, CellDomain::with_all(puzzle.size));
    apply_unary_constraints(puzzle, &mut cell_domains);

}
