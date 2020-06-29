use std::path::{Path, PathBuf};
use std::{fs, io};

use anyhow::{Context, Result};
use camcam::puzzle::Puzzle;
use image::RgbImage;
use tempfile::{tempdir, TempDir};

const IMG_EXT: &str = "jpg";

pub(crate) struct PuzzleFolderBuilder {
    temp_dir: TempDir,
    saved: bool,
}

impl PuzzleFolderBuilder {
    pub fn new() -> io::Result<Self> {
        let s = Self {
            temp_dir: tempdir()?,
            saved: false,
        };
        Ok(s)
    }

    pub fn save<P: AsRef<Path>>(mut self, path: P) -> io::Result<()> {
        fs::rename(&self.temp_dir, path)?;
        self.saved = true;
        Ok(())
    }

    pub fn steps_path(&self) -> PathBuf {
        self.temp_dir.path().join("steps")
    }

    pub fn write_puzzle(&self, puzzle: &Puzzle) -> Result<()> {
        let path = self.temp_dir.path().join("puzzle");
        fs::write(&path, &puzzle.to_string().into_bytes())
            .with_context(|| format!("Error saving puzzle to {}", path.display()))?;
        Ok(())
    }

    pub fn write_puzzle_image(&self, image: &RgbImage) -> Result<()> {
        let path = self.temp_dir.path().join(format!("image.{}", IMG_EXT));
        image.save(&path).context("error saving puzzle image")?;
        Ok(())
    }

    pub fn write_solved_puzzle_image(&self, image: &RgbImage) -> Result<()> {
        let path = self
            .temp_dir
            .path()
            .join(format!("image_solved.{}", IMG_EXT));
        image
            .save(&path)
            .context("error saving solved puzzle image")?;
        Ok(())
    }
}

impl Drop for PuzzleFolderBuilder {
    fn drop(&mut self) {
        if !self.saved {
            fs::remove_dir_all(&self.temp_dir).expect("Error removing puzzle temp dir");
        }
    }
}
