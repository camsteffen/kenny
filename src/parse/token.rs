use std::fmt;

use crate::puzzle::Operator;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Token {
    Letter(char),
    Number(u32),
    Operator(Operator),
    Space,
}

impl Token {
    pub fn letter(self) -> Option<char> {
        match self {
            Token::Letter(l) => Some(l),
            _ => None,
        }
    }

    pub fn number(self) -> Option<u32> {
        match self {
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
