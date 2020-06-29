//! Save images of the puzzle in a series of solution steps

use std::path::Path;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::puzzle::image::PuzzleImageBuilder;
use crate::puzzle::solve::markup::{CellChanges, PuzzleMarkup};
use crate::puzzle::Puzzle;

pub(crate) struct StepWriter<'a> {
    puzzle: &'a Puzzle,
    image_width: Option<u32>,
    index: u32,
    path: PathBuf,
}

impl StepWriter<'_> {
    pub fn write_step(&mut self, markup: &PuzzleMarkup, cell_changes: &CellChanges) -> Result<()> {
        let mut path = self.path.clone();
        path.push(format!("step_{}.jpeg", self.index));
        self.write(&path, markup, cell_changes)?;
        self.index += 1;
        Ok(())
    }

    pub fn write_backtrack(
        &mut self,
        markup: &PuzzleMarkup,
        cell_changes: &CellChanges,
        backtrack_level: u32,
    ) -> Result<()> {
        let mut path = self.path.clone();
        path.push(format!("step_{}_bt_{}.jpeg", self.index, backtrack_level));
        self.write(&path, markup, cell_changes)?;
        self.index += 1;
        Ok(())
    }

    fn write(&self, path: &Path, markup: &PuzzleMarkup, cell_changes: &CellChanges) -> Result<()> {
        debug!("writing step image: {}", path.display());
        let mut builder = PuzzleImageBuilder::new(self.puzzle);
        builder
            .cell_variables(Some(&markup.cells()))
            .cell_changes(cell_changes);
        if let Some(image_width) = self.image_width {
            builder.image_width(image_width);
        }
        let image = builder.build();
        image
            .save(&path)
            .with_context(|| format!("Error saving step image to {}", path.display()))?;
        Ok(())
    }
}

pub(crate) struct StepWriterBuilder<'a> {
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
