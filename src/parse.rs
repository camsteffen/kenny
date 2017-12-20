//! Parse puzzles from text

use puzzle::Cage;
use puzzle::Operator;
use puzzle::Puzzle;
use std::collections::BTreeMap;
use std::fmt;
use std::iter::once;
use std::str;
use collections::square::SquareIndex;

struct SIndex(u32, u32);

impl fmt::Display for SIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.0, self.1)
    }
}

enum Token {
    Invalid(String),
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Token::Invalid(ref s) => write!(f, "{}", s),
            Token::Number(ref n) => write!(f, "{}", n),
            Token::Operator(ref o) => o.symbol().map_or_else(|| Ok(()), |symbol| write!(f, "{}", symbol)),
            Token::Letter(ref l) => write!(f, "{}", l),
            Token::Space => write!(f, " "),
        }
    }
}

struct StringTokenIterator<'a> {
    s: &'a str,
    line: u32,
    col: u32,
}

impl<'a> StringTokenIterator<'a> {
    fn new(s: &str) -> StringTokenIterator {
        StringTokenIterator {
            s: s,
            line: 1,
            col: 1,
        }
    }

    fn next_skip_space(&mut self) -> Option<(SIndex, Token)> {
        loop {
            let (i, t) = match self.next() {
                Some(s) => s,
                None => return None,
            };
            match t {
                Token::Space => continue,
                _ => return Some((i, t)),
            }
        }
    }
}

impl<'a> Iterator for StringTokenIterator<'a> {
    type Item = (SIndex, Token);

    fn next(&mut self) -> Option<Self::Item> {
        let mut take = 0;
        let mut decrement_take = false;
        let token = {
            let mut chars = self.s.chars().inspect(|_| take += 1);
            let mut c = match chars.next() {
                Some(c) => c,
                None => return None,
            };
            if c.is_whitespace() {
                loop {
                    if c == '\n' {
                        self.line += 1;
                        self.col = 1;
                    } else {
                        self.col += 1;
                    }
                    c = match chars.next() {
                        Some(c) => c,
                        None => break,
                    };
                    if !c.is_whitespace() {
                        break
                    }
                }
                decrement_take = true;
                Token::Space
            } else if c.is_digit(10) {
                let tail = chars.take_while(|c| c.is_digit(10));
                let s = once(c).chain(tail).collect::<String>();
                decrement_take = true;
                match s.parse() {
                    Ok(n) => Token::Number(n),
                    Err(_) => Token::Invalid(s),
                }
            } else if let Some(o) = Operator::from_symbol(c) {
                Token::Operator(o)
            } else if c >= 'A' && c <= 'Z' {
                Token::Letter(c)
            } else {
                Token::Invalid(c.to_string())
            }
        };
        if decrement_take { take -= 1; }
        self.s = &self.s[take..];
        Some((SIndex(self.line, self.col), token))
    }
}

/// parse a `Puzzle` from a string
pub fn parse_puzzle(s: &str) -> Result<Puzzle, String> {
    let mut s = StringTokenIterator::new(s);
    let (i, token) = s.next_skip_space().ok_or("unexpected EOF")?;
    let size = token.number().ok_or_else(|| format_parse_error("invalid size", &token, &i))? as usize;
    if size > (('Z' as u8) - ('A' as u8) + 1) as usize {
        return Err("size is too big".to_string())
    }
    let cage_cells = read_cage_cells(&mut s, size)?;
    let cage_targets = read_cage_targets(&mut s, cage_cells.len())?;
    debug_assert!(cage_cells.len() == cage_targets.len());
    if let Some((i, t)) = s.next_skip_space() {
        return Err(format_parse_error("unexpected token", &t, &i))
    }
    let cages = cage_cells.into_iter().zip(cage_targets.into_iter())
        .map(|((cage_id, cells), (target, operator))| {
            if cells.len() > 1 && operator.is_none() {
                return Err(format!("cage {} is missing an operator", cage_id))
            }
            let operator = match operator {
                Some(o) => o,
                None => Operator::Add,
            };
            let cage = Cage {
                cells,
                target: target as i32,
                operator: operator,
            };
            Ok(cage)
        })
        .collect::<Result<_, _>>()?;
    Ok(Puzzle::new(size, cages))
}

fn read_cage_cells(s: &mut StringTokenIterator, size: usize) -> Result<BTreeMap<char, Vec<SquareIndex>>, String> {
    let mut cages = BTreeMap::new();
    for cell in 0..(size * size) as usize {
        let (i, token) = s.next_skip_space().ok_or("unexpected EOF")?;
        let l = token.letter().ok_or_else(|| format_parse_error("invalid cage id", &token, &i))?;
        if !l.is_uppercase() {
            return Err(format_parse_error("invalid cage id", &l, &i));
        }
        cages.entry(l).or_insert_with(Vec::new).push(SquareIndex(cell));
    }
    Ok(cages)
}

fn read_cage_targets(s: &mut StringTokenIterator, num_cages: usize) -> Result<Vec<(u32, Option<Operator>)>, String> {
    (0..num_cages).map(|_| {
        let (i, token) = s.next_skip_space().ok_or("unexpected EOF")?;
        let target = token.number().ok_or_else(|| format_parse_error("invalid target", &token, &i))?;
        let (i, token) = s.next().ok_or("unexpected EOF")?;
        let operator = match token {
            Token::Operator(o) => Some(o),
            Token::Space => None,
            _ => return Err(format_parse_error("invalid operator", &token, &i)),
        };
        Ok((target, operator))
    })
    .collect()
}

fn format_parse_error<T: fmt::Display>(msg: &str, s: &T, i: &SIndex) -> String {
    format!("{}: '{}' ({})", msg, s, i)
}

