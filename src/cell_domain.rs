use std::mem;

#[derive(Clone)]
pub struct CellDomain {
    size: usize,
    domain: Vec<bool>,
}

impl CellDomain {
    pub fn with_all(size: usize) -> CellDomain {
        CellDomain {
            size: size,
            domain: vec![true; size],
        }
    }

    pub fn with_none(size: usize) -> CellDomain {
        CellDomain {
            size: 0,
            domain: vec![false; size],
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn insert(&mut self, n: i32) -> bool {
        let index = Self::hash(n);
        let existed = self.domain[index];
        self.domain[index] = true;
        if !existed {
            self.size += 1
        }
        existed
    }

    pub fn remove(&mut self, n: i32) -> bool {
        let i = Self::hash(n);
        if self.domain[i] {
            self.domain[i] = false;
            self.size = self.size - 1;
            true
        } else {
            false
        }
    }

    pub fn contains(&self, n: i32) -> bool {
        self.domain[Self::hash(n)]
    }

    pub fn single_value(&self) -> Option<i32> {
        match self.size {
            1 => Some(self.iter().next().unwrap()),
            _ => None,
        }
    }

    pub fn iter<'a>(&'a self) -> Iter<'a> {
        Iter {
            domain: &self.domain,
            i: 0,
        }
    }

    #[inline]
    fn hash(n: i32) -> usize {
        (n - 1) as usize
    }
}

pub struct Iter<'a> {
    domain: &'a [bool],
    i: i32,
}

impl<'a> Iterator for Iter<'a> {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        let pos = match self.domain.iter().position(|c| *c) {
            Some(pos) => pos,
            None => return None,
        };
        self.i = self.i + pos as i32 + 1;
        let domain = mem::replace(&mut self.domain, &mut []);

        let (_, remaining) = domain.split_at(pos + 1);
        self.domain = remaining;
        Some(self.i)
    }
}

