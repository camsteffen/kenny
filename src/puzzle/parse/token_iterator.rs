use std::iter::Peekable;
use std::str::CharIndices;

use super::Result;
use crate::puzzle::error::ParseError;
use crate::puzzle::error::ParsePuzzleErrorType::*;
use crate::puzzle::parse::Token;
use crate::puzzle::Operator;

pub type IndexedToken = (usize, Token);

pub(crate) struct TokenIterator<'a> {
    chars: Peekable<CharIndices<'a>>,
}

impl TokenIterator<'_> {
    pub fn new(s: &str) -> TokenIterator<'_> {
        TokenIterator {
            chars: s.char_indices().peekable(),
        }
    }

    pub fn next_skip_space(&mut self) -> Result<Option<IndexedToken>> {
        match self.next() {
            Ok(Some((_, Token::Space))) => self.next(),
            next => next,
        }
    }

    pub fn next(&mut self) -> Result<Option<IndexedToken>> {
        let (idx, c) = match self.chars.peek() {
            Some(&v) => v,
            None => return Ok(None),
        };
        let token = if c.is_whitespace() {
            loop {
                self.chars.next().unwrap();
                if self.chars.peek().map_or(true, |(_, c)| !c.is_whitespace()) {
                    break;
                }
            }
            Token::Space
        } else if c.is_ascii_digit() {
            let mut s = c.to_string();
            loop {
                self.chars.next().unwrap();
                match self.chars.peek() {
                    Some(&(_, c)) => {
                        if c.is_ascii_digit() {
                            s.push(c);
                        } else {
                            break;
                        }
                    }
                    None => break,
                };
            }
            match s.parse() {
                Ok(n) => Token::Number(n),
                Err(_) => return Err(ParseError::new(InvalidToken, s, idx)),
            }
        } else if let Some(o) = Operator::from_symbol(c) {
            self.chars.next().unwrap();
            Token::Operator(o)
        } else if c >= 'A' && c <= 'Z' {
            self.chars.next().unwrap();
            Token::Letter(c)
        } else {
            return Err(ParseError::new(InvalidToken, c, idx));
        };
        Ok(Some((idx, token)))
    }
}

impl Iterator for TokenIterator<'_> {
    type Item = Result<IndexedToken>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next().transpose()
    }
}

#[cfg(test)]
mod test {
    use crate::puzzle::error::ParseError;
    use crate::puzzle::error::ParsePuzzleErrorType::InvalidToken;
    use crate::puzzle::parse::token_iterator::{IndexedToken, TokenIterator};
    use crate::puzzle::parse::{Result, Token};
    use crate::puzzle::Operator;

    #[test]
    fn test() {
        let result: Vec<IndexedToken> = TokenIterator::new(" AB123 \t9 +* -/")
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(
            vec![
                (0, Token::Space),
                (1, Token::Letter('A')),
                (2, Token::Letter('B')),
                (3, Token::Number(123)),
                (6, Token::Space),
                (8, Token::Number(9)),
                (9, Token::Space),
                (10, Token::Operator(Operator::Add)),
                (11, Token::Operator(Operator::Multiply)),
                (12, Token::Space),
                (13, Token::Operator(Operator::Subtract)),
                (14, Token::Operator(Operator::Divide)),
            ],
            result
        );
    }

    #[test]
    fn invalid() {
        let result: Result<Vec<IndexedToken>> = TokenIterator::new("^").collect();
        assert_eq!(ParseError::new(InvalidToken, "^", 0), result.err().unwrap());
    }
}
