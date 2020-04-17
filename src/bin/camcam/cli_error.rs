use std::path::PathBuf;
use std::{io, fmt};
use std::fmt::{Debug, Display};
use puzzle::error::SolveError;
use camcam::puzzle;
use camcam::puzzle::error::{Error, ParsePuzzleError};

#[derive(Debug)]
pub enum CliError {
    CreateDirectory(PathBuf, io::Error),
    Io(io::Error),
    NothingToSave,
    ParsePuzzle(ParsePuzzleError),
    Solve(SolveError),
}

impl Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CliError::CreateDirectory(path, e) => write!(f, "Error creating directory {}: {}", path.display(), e),
            CliError::Io(e) => write!(f, "IO error: {}", e),
            CliError::NothingToSave => write!(f, "output path specified but nothing to save"),
            CliError::ParsePuzzle(e) => write!(f, "{}", e),
            CliError::Solve(e) => write!(f, "{}", e),
        }
    }
}

impl From<SolveError> for CliError {
    fn from(error: SolveError) -> Self {
        CliError::Solve(error)
    }
}

impl From<puzzle::error::Error> for CliError {
    fn from(err: puzzle::error::Error) -> Self {
        match err {
            Error::Io(e) => CliError::Io(e),
            Error::ParsePuzzle(e) => CliError::ParsePuzzle(e),
            Error::SolvePuzzle(e) => CliError::Solve(e),
        }
    }
}

impl From<io::Error> for CliError {
    fn from(err: io::Error) -> Self {
        CliError::Io(err)
    }
}
