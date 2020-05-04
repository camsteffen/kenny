//! Save images of the puzzle in a series of solution steps

use crate::puzzle::image::PuzzleImageBuilder;
use crate::puzzle::solve::PuzzleMarkup;
use crate::puzzle::{CellId, Puzzle};
use failure::{Fallible, ResultExt};
use std::path::Path;
use std::path::PathBuf;

pub struct StepWriter<'a> {
    puzzle: &'a Puzzle,
    image_width: Option<u32>,
    index: u32,
    path: PathBuf,
}

impl StepWriter<'_> {
    pub fn write_next(&mut self, markup: &PuzzleMarkup, changed_cells: &[CellId]) -> Fallible<()> {
        let mut path = self.path.clone();
        path.push(format!("step_{}.jpeg", self.index));
        self.write(&path, markup, changed_cells)?;
        self.index += 1;
        Ok(())
    }

    pub fn write_backtrack(
        &mut self,
        markup: &PuzzleMarkup,
        changed_cells: &[CellId],
        backtrack_level: u32,
    ) -> Fallible<()> {
        let mut path = self.path.clone();
        path.push(format!("step_{}_bt_{}.jpeg", self.index, backtrack_level));
        self.write(&path, markup, changed_cells)?;
        self.index += 1;
        Ok(())
    }

    fn write(&self, path: &Path, markup: &PuzzleMarkup, changed_cells: &[CellId]) -> Fallible<()> {
        debug!("writing step image: {}", path.display());
        let mut builder = PuzzleImageBuilder::new(self.puzzle);
        builder
            .highlighted_cells(Some(changed_cells))
            .cell_variables(Some(&markup.cells()));
        if let Some(image_width) = self.image_width {
            builder.image_width(image_width);
        }
        let image = builder.build();
        image
            .save(&path)
            .with_context(|e| format!("Error saving step image to {}: {}", path.display(), e))?;
        Ok(())
    }
}

pub struct StepWriterBuilder<'a> {
    puzzle: &'a Puzzle,
    image_width: Option<u32>,
    path: PathBuf,
}

impl<'a> StepWriterBuilder<'a> {
    pub fn new(puzzle: &'a Puzzle, path: &Path) -> Self {
        Self {
            puzzle,
            image_width: None,
            path: path.to_path_buf(),
        }
    }

    pub fn image_width(&mut self, image_width: u32) -> &mut Self {
        self.image_width = Some(image_width);
        self
    }

    pub fn build(&self) -> StepWriter<'a> {
        StepWriter {
            puzzle: self.puzzle,
            image_width: self.image_width,
            index: 1,
            path: self.path.clone(),
        }
    }
}
