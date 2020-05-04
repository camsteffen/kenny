use std::fs;
use std::path::{Path, PathBuf};

use failure::{Fallible, ResultExt};

use crate::options::Options;
use crate::puzzle_folder_builder::PuzzleFolderBuilder;
use camcam::Puzzle;
use std::ops::{Deref, DerefMut};
use std::panic::RefUnwindSafe;

type PathIter = dyn Iterator<Item = PathBuf> + RefUnwindSafe;

pub struct Context {
    options: Options,
    puzzle_path_iter: Option<Box<PathIter>>,
}

impl Context {
    pub fn new(options: Options) -> Fallible<Self> {
        if let Some(path) = options.output_path() {
            if !path.exists() {
                fs::create_dir(&path).with_context(|e| {
                    format!("Error creating directory {}: {}", path.display(), e)
                })?;
            }
        }

        let puzzle_path_iter = options.output_path().map(|path| {
            let iter = puzzle_path_iterator(path);
            let boxed_iter: Box<PathIter> = Box::new(iter);
            boxed_iter
        });

        Ok(Self {
            options,
            puzzle_path_iter,
        })
    }

    pub fn options(&self) -> &Options {
        &self.options
    }

    pub fn next_puzzle_path(&mut self) -> PathBuf {
        let iter = self.puzzle_path_iter.as_mut().expect("no puzzle path");
        iter.next().unwrap()
    }
}

pub struct PuzzleContext<'a> {
    context: &'a mut Context,
    puzzle: &'a Puzzle,
    folder_builder: Option<PuzzleFolderBuilder>,
}

impl<'a> PuzzleContext<'a> {
    pub fn new(context: &'a mut Context, puzzle: &'a Puzzle) -> Fallible<Self> {
        let folder_builder = if context.options().save_any() {
            Some(PuzzleFolderBuilder::new()?)
        } else {
            None
        };
        Ok(Self {
            context,
            puzzle,
            folder_builder,
        })
    }

    pub fn puzzle(&self) -> &Puzzle {
        &self.puzzle
    }

    pub fn folder_builder(&self) -> Option<&PuzzleFolderBuilder> {
        self.folder_builder.as_ref()
    }

    pub fn take_folder_builder(&mut self) -> Option<PuzzleFolderBuilder> {
        self.folder_builder.take()
    }
}

impl Deref for PuzzleContext<'_> {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        self.context
    }
}

impl DerefMut for PuzzleContext<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.context
    }
}

/// Creates an infinite iterator of paths to save puzzle data, one path per puzzle.
/// Paths are named puzzle_{n}.
/// Existing paths are automatically skipped.
fn puzzle_path_iterator(root: &Path) -> impl Iterator<Item = PathBuf> {
    let root = PathBuf::from(root);
    (1_usize..)
        .map(move |i| {
            let mut path = root.clone();
            path.push(format!("puzzle_{}", i));
            path
        })
        .filter(|path| !path.exists())
}
