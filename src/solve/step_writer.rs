//! Save images of the puzzle in a series of solution steps

use std::fmt::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::image::PuzzleImageBuilder;
use crate::puzzle::Puzzle;
use crate::solve::markup::{CellChanges, PuzzleMarkup};

static IMAGE_EXTENSION: &str = "svg";

// todo merge into PuzzleFolderBuilder?
//   need to resolve visibility with solver code
pub(crate) struct StepWriter<'a> {
    puzzle: &'a Puzzle,
    path: PathBuf,
    location: Vec<LocationNode>,
}

struct LocationNode {
    branch: u32,
    step: u32,
}

impl<'a> StepWriter<'a> {
    pub fn new(puzzle: &'a Puzzle, path: PathBuf) -> Self {
        Self {
            puzzle,
            path,
            location: vec![LocationNode { branch: 0, step: 0 }],
        }
    }
}

impl StepWriter<'_> {
    pub fn write_step(&mut self, markup: &PuzzleMarkup<'_>, changes: &CellChanges) -> Result<()> {
        self.location.last_mut().unwrap().step += 1;
        let name = self.file_name();
        let path = self.path.join(name);
        debug!("writing step image: {}", path.display());
        let mut builder = PuzzleImageBuilder::new(self.puzzle);
        builder
            .cell_variables(Some(&markup.cells()))
            .cell_changes(changes);
        let image = builder.build();
        image
            .save_svg(&path)
            .with_context(|| format!("Error saving step image to {}", path.display()))?;
        Ok(())
    }

    pub fn start_search_branch(&mut self) {
        self.location.push(LocationNode { branch: 0, step: 0 });
    }

    pub fn next_search_branch(&mut self) {
        let node = self.location.last_mut().unwrap();
        node.branch += 1;
        node.step = 1;
    }

    pub fn end_search_branch(&mut self) {
        self.location.pop();
        debug_assert!(!self.location.is_empty());
    }

    fn file_name(&mut self) -> String {
        let mut name = String::with_capacity(self.location.len() * 5 + 1 + IMAGE_EXTENSION.len());
        write!(name, "{:02}", self.location[0].step).unwrap();
        for &LocationNode { branch, step } in &self.location[1..] {
            write!(name, "_{}_{:02}", branch, step).unwrap();
        }
        write!(name, ".{}", IMAGE_EXTENSION).unwrap();
        name
    }
}
