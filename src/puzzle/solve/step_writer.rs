//! Save images of the puzzle in a series of solution steps

use std::path::Path;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::puzzle::image::PuzzleImageBuilder;
use crate::puzzle::solve::markup::{CellChanges, PuzzleMarkup};
use crate::puzzle::Puzzle;

static IMAGE_EXTENSION: &str = "svg";

// todo merge into PuzzleFolderBuilder?
//   need to resolve visibility with solver code
pub(crate) struct StepWriter<'a> {
    puzzle: &'a Puzzle,
    index: u32,
    path: PathBuf,
}

impl<'a> StepWriter<'a> {
    pub fn new(puzzle: &'a Puzzle, path: PathBuf) -> Self {
        Self {
            puzzle,
            index: 1,
            path: path,
        }
    }
}

impl StepWriter<'_> {
    pub fn write_step(
        &mut self,
        markup: &PuzzleMarkup<'_>,
        cell_changes: &CellChanges,
    ) -> Result<()> {
        let mut path = self.path.clone();
        path.push(format!("step_{:02}.{}", self.index, IMAGE_EXTENSION));
        self.write(&path, markup, cell_changes)?;
        self.index += 1;
        Ok(())
    }

    pub fn write_backtrack(
        &mut self,
        markup: &PuzzleMarkup<'_>,
        cell_changes: &CellChanges,
        backtrack_level: u32,
    ) -> Result<()> {
        let mut path = self.path.clone();
        path.push(format!(
            "step_{:02}_bt_{}.{}",
            self.index, backtrack_level, IMAGE_EXTENSION
        ));
        self.write(&path, markup, cell_changes)?;
        self.index += 1;
        Ok(())
    }

    fn write(
        &self,
        path: &Path,
        markup: &PuzzleMarkup<'_>,
        cell_changes: &CellChanges,
    ) -> Result<()> {
        debug!("writing step image: {}", path.display());
        let mut builder = PuzzleImageBuilder::new(self.puzzle);
        builder
            .cell_variables(Some(&markup.cells()))
            .cell_changes(cell_changes);
        let image = builder.build();
        image
            .save_svg(path)
            .with_context(|| format!("Error saving step image to {}", path.display()))?;
        Ok(())
    }
}
