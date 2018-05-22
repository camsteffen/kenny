//! Save images of the puzzle in a series of solution steps

use collections::square::SquareIndex;
use puzzle::Puzzle;
use puzzle::image::PuzzleImageBuilder;
use puzzle::solve::PuzzleMarkup;
use std::path::Path;
use std::path::PathBuf;

pub struct StepWriter {
    image_width: Option<u32>,
    index: u32,
    path: PathBuf,
}

impl StepWriter {
    pub fn write_next(&mut self, puzzle: &Puzzle, markup: &PuzzleMarkup, changed_cells: &[SquareIndex]) {
        let mut path = self.path.clone();
        path.push(format!("step_{}.jpeg", self.index));
        debug!("writing step image: {}", path.display());
        let mut builder = PuzzleImageBuilder::new(puzzle);
        builder
            .highlighted_cells(Some(changed_cells))
            .cell_variables(Some(&markup.cell_variables));
        if let Some(image_width) = self.image_width {
            builder.image_width(image_width);
        }
        let image = builder.build();
        image.save(&path).unwrap_or_else(|e|
            panic!(format!("unable to save step image to {}: {}", path.display(), e)));
        self.index += 1;
    }
}

pub struct StepWriterBuilder {
    image_width: Option<u32>,
    path: PathBuf,
}

impl StepWriterBuilder {
    pub fn new(path: &Path) -> Self {
        Self {
            image_width: None,
            path: path.to_path_buf()
        }
    }

    pub fn image_width(&mut self, image_width: u32) -> &mut Self {
        self.image_width = Some(image_width);
        self
    }

    pub fn build(&self) -> StepWriter {
        StepWriter {
            image_width: self.image_width,
            index: 1,
            path: self.path.clone(),
        }
    }
}
