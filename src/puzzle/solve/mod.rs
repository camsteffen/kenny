//! solve KenKen puzzles

use std::path::Path;
use std::path::PathBuf;

use failure::Fallible;

use crate::puzzle::Puzzle;
use crate::puzzle::solve::step_writer::StepWriterBuilder;

pub use self::cell_domain::CellDomain;
pub use self::cell_variable::CellVariable;
use self::constraint::apply_unary_constraints;
use self::constraint::constraint_propagation;
pub use self::markup::PuzzleMarkup;
pub use self::markup::PuzzleMarkupChanges;

mod cage_solutions;
mod cell_domain;
mod cell_variable;
mod constraint;
mod markup;
mod step_writer;

pub struct PuzzleSolver<'a> {
    puzzle: &'a Puzzle,
    steps: Option<StepsContext>,
}

impl<'a> PuzzleSolver<'a> {
    pub fn new(puzzle: &'a Puzzle) -> Self {
        Self {
            puzzle,
            steps: None,
        }
    }

    pub fn save_steps(&mut self, path: &Path) -> &mut Self {
        let steps_context = StepsContext {
            image_width: None,
            path: path.to_path_buf(),
        };
        self.steps = Some(steps_context);
        self
    }

    pub fn steps_image_width(&mut self, image_width: u32) -> &mut Self {
        self.steps.as_mut().unwrap().image_width = Some(image_width);
        self
    }

    pub fn solve(&self) -> Fallible<PuzzleMarkup> {
        let mut changes = PuzzleMarkupChanges::default();
        apply_unary_constraints(self.puzzle, &mut changes);
        let mut markup = PuzzleMarkup::new(self.puzzle);
        markup.sync_changes(self.puzzle, &mut changes);
        let mut step_writer = self.steps.as_ref().map(|steps| {
            let mut builder = StepWriterBuilder::new(&steps.path);
            if let Some(image_width) = steps.image_width {
                builder.image_width(image_width);
            }
            builder.build()
        });
        if let Some(step_writer) = step_writer.as_mut() {
            step_writer.write_next(self.puzzle, &markup, &[])?;
        }
        constraint_propagation(self.puzzle, &mut markup, &mut changes, step_writer.as_mut())?;
        Ok(markup)
    }
}

struct StepsContext {
    image_width: Option<u32>,
    path: PathBuf,
}
