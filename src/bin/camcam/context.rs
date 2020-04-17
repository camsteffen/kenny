use puzzle_folder_builder::PuzzleFolderBuilder;
use options::Options;
use std::path::{PathBuf, Path};
use ::{DEFAULT_PATH, Result};
use std::fs;
use cli_error::CliError;

type PathIterator = impl Iterator<Item=PathBuf>;

pub struct Context {
    options: Options,
    folder_builder: Option<PuzzleFolderBuilder>,
    paths: Option<PathIterator>,
}

impl Context {
    pub fn new(options: Options) -> Result<Self> {
        let output_dir = get_output_dir(&options)?;
        let paths = output_dir.map(|p| puzzle_path_iterator(&p));
        let context = Self {
            options,
            folder_builder: None,
            paths,
        };
        Ok(context)
    }

    pub fn options(&self) -> &Options {
        &self.options
    }

    pub fn folder_builder(&self) -> Option<&PuzzleFolderBuilder> {
        self.folder_builder.as_ref()
    }

    pub fn next_path(&mut self) -> PathBuf {
        self.paths.as_mut().unwrap().next().unwrap()
    }

    pub fn set_temp_dir(&mut self, temp_dir: PuzzleFolderBuilder) {
        self.folder_builder = Some(temp_dir)
    }
}

fn get_output_dir(context: &Options) -> Result<Option<PathBuf>> {
    if context.save_any() {
        let path = match &context.output_path {
            Some(p) => p.as_ref(),
            None => DEFAULT_PATH,
        };
        let path = PathBuf::from(path);
        if !path.exists() {
            fs::create_dir(&path).map_err(|e| CliError::CreateDirectory(path.to_owned(), e))?;
        }
        Ok(Some(path))
    } else if context.output_path.is_some() {
        Err(CliError::NothingToSave)
    } else {
        Ok(None)
    }
}

fn puzzle_path_iterator(root: &Path) -> PathIterator {
    let root = root.to_path_buf();
    (1..)
        .map(move |i| {
            let mut path = root.clone();
            path.push(format!("puzzle_{}", i));
            path
        })
        .filter(|path| !path.exists())
}
