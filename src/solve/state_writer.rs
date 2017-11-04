use image::AsImage;
use std::path::Path;
use std::fs::remove_file;
use super::Solver;

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
            let path_str = StateWriter::image_path(i);
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

    pub fn write(&mut self, solver: &Solver) {
        let path = StateWriter::image_path(self.index);
        debug!("Writing {}", path);
        solver.as_image().save(path).unwrap();
        self.index += 1;
    }

    fn image_path(i: u32) -> String {
        format!("{}state{}.png", OUTPUT_DIR, i)
    }
}