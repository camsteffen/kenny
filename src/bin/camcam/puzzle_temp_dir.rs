extern crate tempfile;

use tempfile::TempDir;
use std::path::PathBuf;
use std::io;
use std::fs::File;
use std::io::Write;
use camcam::Puzzle;
use image::RgbImage;

const IMG_EXT: &str = "jpg";

pub struct PuzzleTempDir {
    temp_dir: TempDir,
}

impl PuzzleTempDir {
    pub fn new() -> io::Result<Self> {
        let s = Self {
            temp_dir: tempfile::tempdir()?,
        };
        Ok(s)
    }

    pub fn steps_path(&self) -> PathBuf {
        self.temp_dir.path().join("steps")
    }

    pub fn write_puzzle(&self, puzzle: &Puzzle) -> io::Result<()> {
        let path = self.temp_dir.path().join("puzzle");
        let mut file = File::create(&path)?;
        file.write_all(&puzzle.to_string().into_bytes())?;
        Ok(())
    }

    pub fn write_puzzle_image(&self, image: RgbImage) -> io::Result<()> {
        let path = self.temp_dir.path().join(format!("image.{}", IMG_EXT));
        image.save(&path)?;
        Ok(())
    }

    pub fn write_saved_puzzle_image(&self, image: RgbImage) -> io::Result<()> {
        let path = self.temp_dir.path().join(format!("image_solved.{}", IMG_EXT));
        image.save(&path)?;
        Ok(())
    }
}

impl Into<TempDir> for PuzzleTempDir {
    fn into(self) -> TempDir {
        self.temp_dir
    }
}

