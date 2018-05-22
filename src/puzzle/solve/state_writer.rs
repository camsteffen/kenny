//! Save images of the puzzle in a series of solution steps

use collections::square::SquareIndex;
use puzzle::Puzzle;
use puzzle::solve::PuzzleMarkup;
use std::path::Path;
use std::path::PathBuf;
use puzzle::image::PuzzleImageBuilder;

pub struct StateWriter {
    index: u32,
    path: PathBuf,
}

impl StateWriter {
    pub fn new(path: &Path) -> Self {
        Self {
            index: 1,
            path: path.to_path_buf(),
        }
    }

    pub fn write_next(&mut self, puzzle: &Puzzle, markup: &PuzzleMarkup, changed_cells: &[SquareIndex]) {
        let mut path = self.path.clone();
        path.push(format!("step_{}.jpeg", self.index));
        debug!("writing step image: {}", path.display());
        let image = PuzzleImageBuilder::new(puzzle)
            .cell_variables(Some(&markup.cell_variables))
            .highlighted_cells(Some(changed_cells))
            .build();
        image.save(&path).unwrap_or_else(|e|
            panic!(format!("unable to save step image to {}: {}", path.display(), e)));
        self.index += 1;
    }
}
