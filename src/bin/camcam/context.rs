use std::fs;
use std::path::{Path, PathBuf};

use failure::{Fallible, ResultExt};

use crate::options::Options;
use crate::puzzle_folder_builder::PuzzleFolderBuilder;
use camcam::Puzzle;

// type PathIterator = impl Iterator<Item=PathBuf> + Send + Sync + 'static;

pub struct Context {
    options: Options,
    puzzle_path_iter: Option<Box<dyn Iterator<Item=PathBuf>>>,
}

impl Context {
    pub fn new(options: Options) -> Fallible<Self> {
        if let Some(path) = options.output_path() {
            if !path.exists() {
                fs::create_dir(&path).with_context(|e|
                    format!("Error creating directory {}: {}", path.display(), e))?;
            }
        }
        let puzzle_path_iter = options.output_path()
            .map(|path| Box::new(puzzle_path_iterator(path)) as Box<dyn Iterator<Item=PathBuf>>);
        let context = Self {
            options,
            puzzle_path_iter,
        };
        Ok(context)
    }

    pub fn options(&self) -> &Options {
        &self.options
    }

    fn next_puzzle_path(&mut self) -> PathBuf {
        self.puzzle_path_iter.as_mut().unwrap().next().unwrap()
    }
}

pub struct PuzzleContext {
    options: Options,
    puzzle: Puzzle,
    folder_builder: Option<PuzzleFolderBuilder>,
}

impl PuzzleContext {
    pub fn new(options: Options, puzzle: Puzzle) -> Fallible<Self> {
        let folder_builder = if options.save_any() {
            Some(PuzzleFolderBuilder::new()?)
        } else {
            None
        };
        Ok(Self {
            options,
            puzzle,
            folder_builder,
        })
    }

    pub fn options(&self) -> &Options {
        &self.options
    }

    pub fn puzzle(&self) -> &Puzzle {
        &self.puzzle
    }

    pub fn folder_builder(&self) -> Option<&PuzzleFolderBuilder> {
        self.folder_builder.as_ref()
    }

    pub fn save_folder_if_present(&mut self, context: &mut Context) -> Fallible<()> {
        if let Some(folder_builder) = self.folder_builder.take() {
            let path = context.next_puzzle_path();
            folder_builder.save(&path)?;
            println!("Saved puzzle to {}", path.display());
        }
        Ok(())
    }
}

fn puzzle_path_iterator(root: &Path) -> impl Iterator<Item=PathBuf> {
    let root = PathBuf::from(root);
    (1..)
        .map(move |i| {
            let mut path = root.clone();
            path.push(format!("puzzle_{}", i));
            path
        })
        .filter(|path| !path.exists())
}
