//! Save images of the puzzle in a series of solution steps

use collections::square::SquareIndex;
use puzzle::Puzzle;
use puzzle::solve::PuzzleMarkup;
use std::fs::remove_file;
use std::fs;
use std::path::Path;

const OUTPUT_DIR: &str = "output/steps/";

pub struct StateWriter {
    index: u32,
}

impl StateWriter {
    pub fn new() -> StateWriter {
        let sw = StateWriter {
            index: 1,
        };
        for i in 1.. {
            let path_str = image_path(i);
            let path = Path::new(&path_str);
            if path.is_file() {
                debug!("removing {}", path_str);
                remove_file(path).unwrap();
            } else {
                break
            }
        }
        sw
    }

    pub fn write_next(&mut self, puzzle: &Puzzle, markup: &PuzzleMarkup, changed_cells: &[SquareIndex]) {
        let path = image_path(self.index);
        debug!("Writing {}", path);
        fs::create_dir_all(OUTPUT_DIR).unwrap();
        puzzle.image_with_markup_and_highlighted_cells(markup, changed_cells).save(path).unwrap();
        self.index += 1;
    }
}

fn image_path(i: u32) -> String {
    format!("{}state{}.jpeg", OUTPUT_DIR, i)
}
