//! Parse puzzles from text

use std::collections::BTreeMap;
use std::fmt::Display;
use std::str;

use crate::collections::square::{SquareIndex, SquareValue};
use crate::error::{
    ParseError, ParsePuzzleError, ParsePuzzleErrorType, ParsePuzzleErrorType::*, UNEXPECTED_END,
};
use crate::parse::token_iterator::IndexedToken;
use crate::puzzle::Cage;
use crate::puzzle::Operator;
use crate::puzzle::Puzzle;
use token_iterator::TokenIterator;

pub(crate) use token::Token;

mod token;
mod token_iterator;

pub type Result<T, E = ParseError> = std::result::Result<T, E>;

const MAX_PUZZLE_SIZE: SquareValue = ((b'Z') - (b'A') + 1) as SquareValue;

/// parse a `Puzzle` from a string
pub fn parse_puzzle(s: &str) -> Result<Puzzle, ParsePuzzleError> {
    let mut s = TokenIterator::new(s);
    let size = s
        .next_skip_space()?
        .expect_token()?
        .map_or(InvalidSize, Token::number)?
        .value();
    if size > MAX_PUZZLE_SIZE {
        return Err(ParseError::from_type(SizeTooBig).into());
    }
    let cage_cells = read_cage_cells(&mut s, size)?;
    let cage_targets = read_cage_targets(&mut s, cage_cells.len())?;
    debug_assert!(cage_cells.len() == cage_targets.len());
    if let Some((i, t)) = s.next_skip_space()? {
        return Err(ParseError::new(UnexpectedToken, t, i).into());
    }
    let cages: Vec<Cage> = cage_cells
        .into_iter()
        .zip(cage_targets.into_iter())
        .map(|(cells, (target, operator))| Cage::new(cells, operator, target as i32))
        .collect::<Result<_, _>>()?;
    let puzzle = Puzzle::new(size, cages)?;
    Ok(puzzle)
}

fn read_cage_cells(s: &mut TokenIterator<'_>, width: SquareValue) -> Result<Vec<Vec<SquareIndex>>> {
    let mut cage_map: BTreeMap<char, Vec<usize>> = BTreeMap::new();
    for cell in 0..(width as SquareIndex).pow(2) {
        let letter = s
            .next_skip_space()?
            .expect_token()?
            .map_or(InvalidCageId, Token::letter)?
            .filter_or(InvalidCageId, char::is_uppercase)?
            .value();
        cage_map.entry(letter).or_default().push(cell);
    }
    let cages = cage_map.into_iter().map(|(_id, cages)| cages).collect();
    Ok(cages)
}

fn read_cage_targets(s: &mut TokenIterator<'_>, num_cages: usize) -> Result<Vec<(u32, Operator)>> {
    (0..num_cages)
        .map(|_| -> Result<_> {
            let target = s
                .next_skip_space()?
                .expect_token()?
                .map_or(InvalidCageTarget, Token::number)?
                .value();
            let operator = if let Some((i, token)) = s.next()? {
                match token {
                    Token::Operator(o) => o,
                    Token::Space => Operator::Nop,
                    _ => return Err(ParseError::new(InvalidOperator, token, i)),
                }
            } else {
                Operator::Nop
            };
            Ok((target, operator))
        })
        .collect()
}

trait TokenOption<T>: Sized {
    fn expect_token(self) -> Result<T>;
}

impl TokenOption<(usize, Token)> for Option<(usize, Token)> {
    fn expect_token(self) -> Result<IndexedToken> {
        self.ok_or(UNEXPECTED_END)
    }
}

trait IndexedTokenExt<T>: Sized + Into<(usize, T)>
where
    T: Copy + Display,
{
    fn filter_or(
        self,
        error_type: ParsePuzzleErrorType,
        predicate: impl FnOnce(T) -> bool,
    ) -> Result<(usize, T)> {
        let (index, token) = self.into();
        if !predicate(token) {
            return Err(ParseError::new(error_type, token, index));
        }
        Ok((index, token))
    }

    fn map_or<U>(
        self,
        error_type: ParsePuzzleErrorType,
        f: impl FnOnce(T) -> Option<U>,
    ) -> Result<(usize, U)> {
        let (index, token) = self.into();
        let n = f(token).ok_or_else(|| ParseError::new(error_type, token, index))?;
        Ok((index, n))
    }

    fn index(self) -> usize;

    fn value(self) -> T;
}

impl<T> IndexedTokenExt<T> for (usize, T)
where
    T: Copy + Display,
{
    fn index(self) -> usize {
        self.0
    }

    fn value(self) -> T {
        self.1
    }
}

#[cfg(test)]
mod tests {
    use crate::parse::parse_puzzle;
    use crate::puzzle::{Cage, Operator, Puzzle};

    #[test]
    fn empty() {
        assert!(parse_puzzle("").is_err());
    }

    #[test]
    fn test() {
        let str = "\
        4\n\
        A ABB\
        ACCC\
        DEEF \
        DGHH \
        4+ 2* 6* 4/ 4- 7 3 2-";
        let cages = vec![
            Cage::new(vec![0, 1, 4], Operator::Add, 4).unwrap(),
            Cage::new(vec![2, 3], Operator::Multiply, 2).unwrap(),
            Cage::new(vec![5, 6, 7], Operator::Multiply, 6).unwrap(),
            Cage::new(vec![8, 12], Operator::Divide, 4).unwrap(),
            Cage::new(vec![9, 10], Operator::Subtract, 4).unwrap(),
            Cage::new(vec![11], Operator::Nop, 7).unwrap(),
            Cage::new(vec![13], Operator::Nop, 3).unwrap(),
            Cage::new(vec![14, 15], Operator::Subtract, 2).unwrap(),
        ];
        let puzzle = Puzzle::new(4, cages).unwrap();
        assert_eq!(puzzle, parse_puzzle(str).unwrap());
    }
}
