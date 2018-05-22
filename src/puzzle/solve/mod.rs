//! solve KenKen puzzles

pub mod constraint;
pub mod markup;

mod cage_solutions;
mod cell_domain;
mod cell_variable;
mod state_writer;

pub use self::cage_solutions::CageSolutions;
pub use self::cage_solutions::CageSolutionsSet;
pub use self::cell_domain::CellDomain;
pub use self::cell_variable::CellVariable;
pub use self::state_writer::StateWriter;

use puzzle::Puzzle;
use self::constraint::apply_unary_constraints;
use self::constraint::constraint_propagation;
use self::markup::PuzzleMarkup;
use self::markup::PuzzleMarkupChanges;
use std::path::Path;

pub fn solve_puzzle(puzzle: &Puzzle, step_images_path: Option<&Path>) -> PuzzleMarkup {
    let mut changes = PuzzleMarkupChanges::new();
    apply_unary_constraints(puzzle, &mut changes);
    let mut markup = PuzzleMarkup::new(puzzle);
    markup.sync_changes(&mut changes);
    constraint_propagation(puzzle, &mut markup, &mut changes, step_images_path);
    markup
}
