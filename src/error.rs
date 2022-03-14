use std::fmt::{Display, Formatter};
use std::{fmt, io};

use thiserror::Error;

#[derive(Error, Debug)]
#[error("invalid puzzle: {}", msg)]
pub struct InvalidPuzzle {
    msg: String,
}

impl InvalidPuzzle {
    pub(crate) fn new(msg: String) -> Self {
        Self { msg }
    }
}

#[derive(Error, Debug)]
pub enum PuzzleFromFileError {
    #[error("error reading puzzle file")]
    Io(#[from] io::Error),
    #[error(transparent)]
    Parse(#[from] ParsePuzzleError),
    #[error(transparent)]
    InvalidPuzzle(#[from] InvalidPuzzle),
}

pub const UNEXPECTED_END: ParseError = ParseError::from_type(ParsePuzzleErrorType::UnexpectedEnd);

#[derive(Debug, Error)]
pub enum ParsePuzzleError {
    #[error(transparent)]
    Parse(#[from] ParseError),
    #[error(transparent)]
    InvalidPuzzle(#[from] InvalidPuzzle),
}

#[derive(Debug, Error)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ParseError {
    error_type: ParsePuzzleErrorType,
    token: Option<String>,
    index: Option<usize>,
}

impl ParseError {
    pub(crate) fn new(error_type: ParsePuzzleErrorType, token: impl Display, index: usize) -> Self {
        Self {
            error_type,
            token: Some(token.to_string()),
            index: Some(index),
        }
    }

    pub(crate) const fn from_type(error_type: ParsePuzzleErrorType) -> Self {
        Self {
            error_type,
            token: None,
            index: None,
        }
    }
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum ParsePuzzleErrorType {
    IllegalOperator,
    InvalidCageId,
    InvalidCageTarget,
    InvalidOperator,
    InvalidSize,
    InvalidToken,
    SizeTooBig,
    UnexpectedEnd,
    UnexpectedToken,
}

impl Display for ParsePuzzleErrorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            ParsePuzzleErrorType::IllegalOperator => "Illegal operator",
            ParsePuzzleErrorType::InvalidCageId => "Invalid cage ID",
            ParsePuzzleErrorType::InvalidCageTarget => "Invalid cage target",
            ParsePuzzleErrorType::InvalidOperator => "Invalid operator",
            ParsePuzzleErrorType::InvalidSize => "Invalid puzzle size",
            ParsePuzzleErrorType::InvalidToken => "Invalid token",
            ParsePuzzleErrorType::SizeTooBig => "Puzzle size too big",
            ParsePuzzleErrorType::UnexpectedEnd => "Unexpected end",
            ParsePuzzleErrorType::UnexpectedToken => "Unexpected token",
        };
        write!(f, "{}", s)
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error_type)?;
        if let Some(token) = &self.token {
            write!(f, ": \"{}\"", token)?;
        }
        if let Some(index) = &self.index {
            write!(f, " at {}", index)?;
        }
        Ok(())
    }
}
