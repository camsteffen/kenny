use std::path::{Path, PathBuf};
use std::{fs, io};

use failure::{Fallible, ResultExt};
use image::RgbImage;
use tempfile::TempDir;

use camcam::Puzzle;

const IMG_EXT: &str = "jpg";

pub struct PuzzleFolderBuilder {
    temp_dir: TempDir,
    saved: bool,
}

impl PuzzleFolderBuilder {
    pub fn new() -> io::Result<Self> {
        let s = Self {
            temp_dir: tempfile::tempdir()?,
            saved: false,
        };
        Ok(s)
    }

    pub fn save<P: AsRef<Path>>(mut self, path: P) -> Result<(), io::Error> {
        fs::rename(&self.temp_dir, path)?;
        self.saved = true;
        Ok(())
    }

    pub fn steps_path(&self) -> PathBuf {
        self.temp_dir.path().join("steps")
    }

    pub fn write_puzzle(&self, puzzle: &Puzzle) -> Fallible<()> {
        let path = self.temp_dir.path().join("puzzle");
        fs::write(&path, &puzzle.to_string().into_bytes())
            .with_context(|e| format!("Error saving puzzle to {}: {}", path.display(), e))?;
        Ok(())
    }

    pub fn write_puzzle_image(&self, image: RgbImage) -> Fallible<()> {
        let path = self.temp_dir.path().join(format!("image.{}", IMG_EXT));
        image
            .save(&path)
            .with_context(|e| format!("Error saving puzzle image: {}", e))?;
        Ok(())
    }

    pub fn write_solved_puzzle_image(&self, image: RgbImage) -> Fallible<()> {
        let path = self
            .temp_dir
            .path()
            .join(format!("image_solved.{}", IMG_EXT));
        image
            .save(&path)
            .with_context(|e| format!("Error saving solved puzzle image: {}", e))?;
        Ok(())
    }
}

// todo cfg?
#[cfg(not(debug_assertions))]
impl Drop for PuzzleFolderBuilder {
    fn drop(&mut self) {
        if !self.saved {
            fs::remove_dir_all(&self.temp_dir).expect("Error removing puzzle temp dir");
        }
    }
}
