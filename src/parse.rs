//! Parse puzzles from text

use puzzle::Cage;
use puzzle::Operator;
use puzzle::Puzzle;
use std::collections::HashMap;
use std::str;
use std::iter::once;
use std::fmt;

struct SIndex(u32, u32);

impl fmt::Display for SIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.0, self.1)
    }
}

type IndexedChar = (SIndex, Token);

enum Token {
    Number(u32),
    Operator(Operator),
    Letter(char),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Token::Number(ref n) => write!(f, "{}", n),
            Token::Operator(ref o) => write!(f, "{}", o.symbol()),
            Token::Letter(ref l) => write!(f, "{}", l),
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
}

impl<'a> Iterator for StringTokenIterator<'a> {
    type Item = (SIndex, Token);

    fn next(&mut self) -> Option<(SIndex, Token)> {
        let mut take = 0;
        let mut decrement_take = false;
        let token = {
            let mut chars = self.s.chars().inspect(|_| take += 1);
            let mut c = match chars.next() {
                Some(c) => c,
                None => return None,
            };
            while c.is_whitespace() {
                if c == '\n' {
                    self.line += 1;
                    self.col = 1;
                } else {
                    self.col += 1;
                }
                c = match chars.next() {
                    Some(c) => c,
                    None => return None,
                };
            }
            if c.is_digit(10) {
                let tail = chars.take_while(|c| c.is_digit(10));
                let s = once(c).chain(tail).collect::<String>();
                let n = s.parse().unwrap_or_else(|_| panic!("Unable to parse number: {}", s));
                decrement_take = true;
                Token::Number(n)
            } else if let Some(o) = Operator::from_symbol(c) {
                Token::Operator(o)
            } else if c >= 'A' && c <= 'Z' {
                Token::Letter(c)
            } else {
                panic!("Invalid token: '{}':{}", c, SIndex(self.line, self.col))
            }
        };
        if decrement_take { take -= 1; }
        self.s = &self.s[take..];
        Some((SIndex(self.line, self.col), token))
    }
}

/// parse a `Puzzle` from a string
pub fn parse_puzzle(s: &str) -> Puzzle {
    // let mut s = s.lines().enumerate().flat_map(|(i, l)| {
        // let i = i + 1;
        // l.chars().enumerate().map(move |(j, c)| (SIndex(i, j), c))
    // });
    // let mut s = s.chars()/*.filter(move |c| !c.is_whitespace())*/;
    let mut s = StringTokenIterator::new(s);
    let size = match s.next().unwrap() {
        (_, Token::Number(n)) => n as usize,
        _ => panic!("Expected size"),
    };
    let mut cage_cells = read_cage_cells(&mut s, size);
    let mut cage_targets = read_cage_targets(&mut s, cage_cells.len());
    let cages = cage_cells.drain(..).zip(cage_targets.drain(..))
        .map(|(cells, (target, operator))|
            Cage {
                cells: cells,
                target: target as i32,
                operator: operator,
            }
        )
        .collect();
    Puzzle {
        cages: cages,
        size: size,
    }
}

fn read_cage_cells(s: &mut Iterator<Item=IndexedChar>, size: usize) -> Vec<Vec<usize>> {
    let mut cages = HashMap::new();
    let mut cell = 0 as usize;
    let mut len = 0;
    loop {
        let (i, id) = s.next().unwrap();
        let id = match id {
            Token::Letter(l) => l,
            _ => panic!("Invalid cage id: {}, ({})", id, i)
        };
        let v = cages.entry(id).or_insert_with(|| {
            /*
            if len == size {
                panic!("too many cages: '{}': {}:{}", id, (indexed_char.0).0, (indexed_char.0).1)
            }
            */
            len += 1;
            Vec::new()
        });
        v.push(cell as usize);
        cell += 1;
        if cell == size * size {
            break
        }
    }
    let a = cages.drain().map(|(_, v)| v);
    a.collect()
}

fn read_cage_targets(s: &mut Iterator<Item=IndexedChar>, num_cages: usize) -> Vec<(u32, Operator)> {
    (0..num_cages).map(|_| {
        let (i, token) = s.next().expect("unexpected EOF");
        let target = match token {
            Token::Number(n) => n,
            _ => panic!("Invalid target: '{}' ({})", token, i)
        };
        let (i, token) = s.next().expect("unexpected EOF");
        let operator = match token {
            Token::Operator(o) => o,
            _ => panic!("Invalid operator: '{}', ({})", token, i),
        };
        (target, operator)
    }).collect()
}


