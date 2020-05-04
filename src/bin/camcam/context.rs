use std::fs;
use std::path::{Path, PathBuf};

use failure::{Fallible, ResultExt};

use crate::options::Options;
use crate::puzzle_folder_builder::PuzzleFolderBuilder;
use camcam::Puzzle;
use std::cell::RefCell;

pub struct Context {
    options: Options,
    puzzle_path_iter: Option<RefCell<Box<dyn Iterator<Item=PathBuf>>>>,
}

impl Context {
    pub fn new(options: Options) -> Fallible<Self> {
        if let Some(path) = options.output_path() {
            if !path.exists() {
                fs::create_dir(&path).with_context(|e|
                    format!("Error creating directory {}: {}", path.display(), e))?;
            }
        }

        let puzzle_path_iter = options.output_path().map(|path| {
            let iter = puzzle_path_iterator(path);
            let boxed_iter = Box::new(iter) as Box<dyn Iterator<Item=PathBuf>>;
            RefCell::new(boxed_iter)
        });

        Ok(Self { options, puzzle_path_iter })
    }

    pub fn options(&self) -> &Options {
        &self.options
    }

    fn next_puzzle_path(&self) -> PathBuf {
        let iter = self.puzzle_path_iter.as_ref().expect("no puzzle path");
        iter.borrow_mut().next().unwrap()
    }
}

pub struct PuzzleContext<'a> {
    options: &'a Options,
    puzzle: Puzzle,
    folder_builder: Option<PuzzleFolderBuilder>,
}

impl<'a> PuzzleContext<'a> {
    pub fn new(options: &'a Options, puzzle: Puzzle) -> Fallible<Self> {
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

    pub fn save_folder_if_present(&mut self, context: &Context) -> Fallible<()> {
        if let Some(folder_builder) = self.folder_builder.take() {
            let path = context.next_puzzle_path();
            folder_builder.save(&path)?;
            println!("Saved puzzle to {}", path.display());
        }
        Ok(())
    }
}

/// Creates an infinite iterator of paths to save puzzle data, one path per puzzle.
/// Paths are named puzzle_{n}.
/// Existing paths are automatically skipped.
fn puzzle_path_iterator(root: &Path) -> impl Iterator<Item=PathBuf> {
    let root = PathBuf::from(root);
    (1_usize..)
        .map(move |i| {
            let mut path = root.clone();
            path.push(format!("puzzle_{}", i));
            path
        })
        .filter(|path| !path.exists())
}
