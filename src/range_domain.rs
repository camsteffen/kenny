use std::mem;

#[derive(Clone)]
pub struct RangeDomain {
    pub size: usize,
    domain: Vec<bool>,
}

impl RangeDomain {
    pub fn with_all(size: usize) -> RangeDomain {
        RangeDomain {
            size: size,
            domain: vec![true; size],
        }
    }

    pub fn with_none(size: usize) -> RangeDomain {
        RangeDomain {
            size: 0,
            domain: vec![false; size],
        }
    }

/*
    pub fn len(&self) -> usize {
        self.size
    }
    */

    pub fn insert(&mut self, n: usize) -> bool {
        let existed = self.domain[n];
        self.domain[n] = true;
        if !existed {
            self.size += 1
        }
        existed
    }

    pub fn remove(&mut self, n: usize) -> bool {
        let existed = self.domain[n];
        if existed {
            self.domain[n] = false;
            self.size -= 1;
        }
        existed
    }

    pub fn contains(&self, n: usize) -> bool {
        self.domain[n]
    }

    pub fn single_value(&self) -> Option<usize> {
        match self.size {
            1 => Some(self.iter().next().unwrap()),
            _ => None,
        }
    }

    pub fn iter(&self) -> Iter {
        Iter {
            domain: &self.domain,
            i: 0,
        }
    }
}

pub struct Iter<'a> {
    domain: &'a [bool],
    i: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let pos = match self.domain.iter().position(|&c| c) {
            Some(pos) => pos,
            None => return None,
        };
        let res = Some(self.i + pos);
        self.i += pos + 1;
        let domain = mem::replace(&mut self.domain, &[]);
        let (_, remaining) = domain.split_at(pos + 1);
        self.domain = remaining;
        res
    }
}

#[derive(Clone)]
pub struct CellDomain {
    rd: RangeDomain,
}

impl CellDomain {
    pub fn with_all(size: usize) -> CellDomain {
        CellDomain {
            rd: RangeDomain::with_all(size),
        }
    }

    pub fn with_none(size: usize) -> CellDomain {
        CellDomain {
            rd: RangeDomain::with_none(size),
        }
    }
    pub fn contains(&self, n: i32) -> bool {
        self.rd.contains(n as usize - 1)
    }

    pub fn insert(&mut self, n: i32) -> bool {
        self.rd.insert(n as usize - 1)
    }

    pub fn iter<'a>(&'a self) -> Box<Iterator<Item=i32> + 'a> {
        Box::new(self.rd.iter().map(|i| i as i32 + 1))
    }

    pub fn remove(&mut self, n: i32) -> bool {
        self.rd.remove(n as usize - 1)
    }

    pub fn size(&self) -> usize {
        self.rd.size
    }

    pub fn single_value(&self) -> Option<i32> {
        self.rd.single_value().map(|n| n as i32 + 1)
    }
}
