use std::iter::Peekable;
use std::str::CharIndices;
use crate::puzzle::error::ParsePuzzleError;
use crate::puzzle::parse::Token;
use crate::puzzle::Operator;

pub struct TokenIterator<'a> {
    chars: Peekable<CharIndices<'a>>,
}

impl<'a> TokenIterator<'a> {
    pub fn new(s: &str) -> TokenIterator<'_> {
        TokenIterator { chars: s.char_indices().peekable() }
    }

    pub fn next_skip_space(&mut self) -> Result<Option<(usize, Token)>, ParsePuzzleError> {
        loop {
            match self.next() {
                Ok(Some((_, Token::Space))) => {},
                next@_ => return next,
            }
        }
    }

    pub fn next(&mut self) -> Result<Option<(usize, Token)>, ParsePuzzleError> {
        let idx: usize;
        let c: char;
        let token = loop {
            match self.chars.peek() {
                Some(v) => {
                    idx = v.0;
                    c = v.1;
                },
                None => return Ok(None),
            };
            if c.is_whitespace() {
                loop {
                    self.chars.next().unwrap();
                    if self.chars.peek().map_or(true, |(_, c)| !c.is_whitespace()) {
                        break
                    }
                }
                break Token::Space
            }
            if c.is_ascii_digit() {
                let mut s = c.to_string();
                loop {
                    self.chars.next().unwrap();
                    match self.chars.peek() {
                        Some(&(_, c)) => {
                            if c.is_ascii_digit() {
                                s.push(c);
                            } else {
                                break
                            }
                        },
                        None => break,
                    };
                }
                match s.parse() {
                    Ok(n) => break Token::Number(n),
                    Err(_) => return Err(format!("invalid token: {}", s).into()),
                }
            } else if let Some(o) = Operator::from_symbol(c) {
                self.chars.next().unwrap();
                break Token::Operator(o)
            } else if c >= 'A' && c <= 'Z' {
                self.chars.next().unwrap();
                break Token::Letter(c)
            } else {
                return Err(format!("invalid token: {}", c).into());
            }
        };
        Ok(Some((idx, token)))
    }
}
