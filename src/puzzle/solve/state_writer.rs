use std::path::Path;
use std::fs::remove_file;
use puzzle::Puzzle;
use puzzle::solve::PuzzleMarkup;
use std::fs;

const OUTPUT_DIR: &str = "output/state/";

/// Writes the puzzle to images as it is solved in steps
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

    pub fn write(&mut self, puzzle: &Puzzle, markup: &PuzzleMarkup) {
        let path = image_path(self.index);
        debug!("Writing {}", path);
        fs::create_dir_all(OUTPUT_DIR).unwrap();
        puzzle.image_with_markup(markup).save(path).unwrap();
        self.index += 1;
    }
}

fn image_path(i: u32) -> String {
    format!("{}state{}.jpeg", OUTPUT_DIR, i)
}
