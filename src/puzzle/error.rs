use std::{io, fmt};
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    ParsePuzzle(ParsePuzzleError),
    SolvePuzzle(SolveError),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "{}", e),
            Error::ParsePuzzle(e) => write!(f, "{}", e),
            Error::SolvePuzzle(e) => write!(f, "{}", e),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<ParsePuzzleError> for Error {
    fn from(err: ParsePuzzleError) -> Self {
        Error::ParsePuzzle(err)
    }
}

impl From<SolveError> for Error {
    fn from(err: SolveError) -> Self {
        Error::SolvePuzzle(err)
    }
}

#[derive(Debug)]
pub struct ParsePuzzleError(String);

impl Display for ParsePuzzleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Error parsing puzzle: {}", self.0)
    }
}

impl<S: Into<String>> From<S> for ParsePuzzleError {
    fn from(reason: S) -> Self {
        Self(reason.into())
    }
}

#[derive(Debug)]
pub enum SolveError {
    CreateStepsDir(io::Error),
    RemoveStepsDir(io::Error),
}

impl std::error::Error for SolveError {}

impl Display for SolveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SolveError::CreateStepsDir(e) => write!(f, "Error creating steps directory: {}", e),
            SolveError::RemoveStepsDir(e) => write!(f, "Error removing steps directory: {}", e),
        }
    }
}

