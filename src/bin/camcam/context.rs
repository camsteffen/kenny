use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

use anyhow::{bail, Context as _, Result};
use camcam::puzzle::Puzzle;

use crate::options::Options;
use crate::puzzle_folder_builder::PuzzleFolderBuilder;

pub(crate) struct Context {
    options: Options,
    puzzle_path_iter: Option<PuzzlePathIter>,
}

impl Context {
    pub fn new(options: Options) -> Result<Self> {
        if let Some(path) = options.output_path() {
            if !path.exists() {
                if let Some(parent) = path.parent() {
                    if !parent.exists() {
                        bail!("Path does not exist: {}", parent.display());
                    }
                }
                fs::create_dir(&path)
                    .with_context(|| format!("Error creating output path: {}", path.display()))?;
            }
        }

        let puzzle_path_iter = options.output_path().map(|path| PuzzlePathIter {
            root: path.into(),
            n: 1,
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

pub(crate) struct PuzzleContext<'a> {
    context: &'a mut Context,
    puzzle: &'a Puzzle,
    folder_builder: Option<PuzzleFolderBuilder>,
}

impl<'a> PuzzleContext<'a> {
    pub fn new(context: &'a mut Context, puzzle: &'a Puzzle) -> Result<Self> {
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

/// Infinite iterator of paths to save puzzle data, one path per puzzle.
/// Paths are named puzzle_{n}.
/// Existing paths are automatically skipped.
struct PuzzlePathIter {
    root: PathBuf,
    n: usize,
}

impl Iterator for PuzzlePathIter {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.n == usize::MAX {
                break None;
            }
            let mut path = self.root.clone();
            path.push(format!("puzzle_{}", self.n));
            self.n += 1;
            if !path.exists() {
                break Some(path);
            }
        }
    }
}
