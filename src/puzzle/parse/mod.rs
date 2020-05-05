//! Parse puzzles from text

use std::collections::BTreeMap;
use std::fmt;
use std::str;

use crate::collections::square::SquareIndex;
use crate::puzzle::error::ParsePuzzleError;
use crate::puzzle::parse::token_iterator::TokenIterator;
use crate::puzzle::Cage;
use crate::puzzle::Operator;
use crate::puzzle::Puzzle;

mod token_iterator;

/// parse a `Puzzle` from a string
pub fn parse_puzzle(s: &str) -> Result<Puzzle, ParsePuzzleError> {
    let mut s = TokenIterator::new(s);
    let (i, token) = s.next_skip_space()?.ok_or("unexpected EOF")?;
    let size = token
        .number()
        .ok_or_else(|| format_parse_error("invalid size", &token, i))? as usize;
    if size > usize::from((b'Z') - (b'A') + 1) {
        return Err("size is too big".into());
    }
    let cage_cells = read_cage_cells(&mut s, size)?;
    let cage_targets = read_cage_targets(&mut s, cage_cells.len())?;
    debug_assert!(cage_cells.len() == cage_targets.len());
    if let Some((i, t)) = s.next_skip_space()? {
        return Err(format_parse_error("unexpected token", &t, i).into());
    }
    let cages = cage_cells
        .into_iter()
        .zip(cage_targets.into_iter())
        .map(|((cage_id, cells), (target, operator))| {
            match cells.len() {
                1 => {
                    if let Some(symbol) = operator.as_ref().map(|o| o.symbol().unwrap()) {
                        return Err(format!(
                            "cage {} has one cell and operator {}",
                            cage_id, symbol
                        ));
                    }
                }
                _ => {
                    if operator.is_none() {
                        return Err(format!("cage {} is missing an operator", cage_id));
                    }
                }
            }
            let operator = operator.unwrap_or(Operator::Nop);
            let cage = Cage::new(target as i32, operator, cells);
            Ok(cage)
        })
        .collect::<Result<_, _>>()?;
    Ok(Puzzle::new(size, cages))
}

pub enum Token {
    Letter(char),
    Number(u32),
    Operator(Operator),
    Space,
}

impl Token {
    fn letter(&self) -> Option<char> {
        match *self {
            Token::Letter(l) => Some(l),
            _ => None,
        }
    }

    fn number(&self) -> Option<u32> {
        match *self {
            Token::Number(n) => Some(n),
            _ => None,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Number(n) => write!(f, "{}", n),
            Token::Operator(o) => o.symbol().map_or(Ok(()), |symbol| write!(f, "{}", symbol)),
            Token::Letter(l) => write!(f, "{}", l),
            Token::Space => write!(f, " "),
        }
    }
}

fn read_cage_cells(
    s: &mut TokenIterator<'_>,
    size: usize,
) -> Result<BTreeMap<char, Vec<SquareIndex>>, ParsePuzzleError> {
    let mut cages = BTreeMap::new();
    for cell in 0..(size * size) {
        let (idx, token) = s
            .next_skip_space()?
            .ok_or(ParsePuzzleError::from("unexpected EOF"))?;
        let l = token
            .letter()
            .ok_or_else(|| format_parse_error("invalid cage id", &token, idx))?;
        if !l.is_uppercase() {
            return Err(format_parse_error("invalid cage id", &l, idx).into());
        }
        cages.entry(l).or_insert_with(Vec::new).push(cell.into());
    }
    Ok(cages)
}

fn read_cage_targets(
    s: &mut TokenIterator<'_>,
    num_cages: usize,
) -> Result<Vec<(u32, Option<Operator>)>, ParsePuzzleError> {
    (0..num_cages)
        .map(|cage_index| -> Result<_, ParsePuzzleError> {
            let (i, token) = s
                .next_skip_space()?
                .ok_or(ParsePuzzleError::from("unexpected EOF"))?;
            let target = token
                .number()
                .ok_or_else(|| format_parse_error("invalid target", &token, i))?;
            let derp = s.next()?;
            let (i, token) = match derp {
                Some(n) => n,
                None => {
                    if cage_index == num_cages - 1 {
                        return Ok((target, None));
                    }
                    return Err(ParsePuzzleError::from("unexpected EOF"));
                }
            };
            let operator = match token {
                Token::Operator(o) => Some(o),
                Token::Space => None,
                _ => return Err(format_parse_error("invalid operator", &token, i).into()),
            };
            Ok((target, operator))
        })
        .collect()
}

fn format_parse_error<T: fmt::Display>(msg: &str, s: &T, i: usize) -> String {
    format!("{}: '{}' (position: {})", msg, s, i)
}

#[cfg(test)]
mod test {
    use crate::puzzle::parse::parse_puzzle;
    use crate::puzzle::{Cage, Operator, Puzzle};

    #[test]
    fn empty() {
        assert!(parse_puzzle("").is_err());
    }

    #[test]
    fn test() {
        let s = "\
        4\n\
        A ABB\
        ACCC\
        DEEF \
        DGFF \
        4+\
        2* \
        6*\
        4/\
        4-\
        7+\
        3";
        let cages = vec![
            Cage::new(4, Operator::Add, vec![0, 1, 4]),
            Cage::new(2, Operator::Multiply, vec![2, 3]),
            Cage::new(6, Operator::Multiply, vec![5, 6, 7]),
            Cage::new(4, Operator::Divide, vec![8, 12]),
            Cage::new(4, Operator::Subtract, vec![9, 10]),
            Cage::new(7, Operator::Add, vec![11, 14, 15]),
            Cage::new(3, Operator::Nop, vec![13]),
        ];
        let puzzle = Puzzle::new(4, cages);
        assert_eq!(puzzle, parse_puzzle(s).unwrap());
    }
}
