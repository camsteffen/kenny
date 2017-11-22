#[derive(Clone)]
pub struct RangeSet {
    size: usize,
    domain: Vec<bool>,
}

impl RangeSet {
    pub fn with_all(size: usize) -> RangeSet {
        RangeSet {
            size: size,
            domain: vec![true; size],
        }
    }

    pub fn with_none(size: usize) -> RangeSet {
        RangeSet {
            size: 0,
            domain: vec![false; size],
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn insert(&mut self, n: usize) -> bool {
        let existed = self.domain[n];
        if !existed {
            self.domain[n] = true;
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
            index: 0,
        }
    }
}

pub struct Iter<'a> {
    domain: &'a [bool],
    index: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        for i in self.index..self.domain.len() {
            if self.domain[i] {
                self.index = i + 1;
                return Some(i)
            }
        }
        None
    }
}

#[derive(Clone)]
pub struct CellDomain {
    rd: RangeSet,
}

impl CellDomain {
    pub fn with_all(size: usize) -> CellDomain {
        CellDomain {
            rd: RangeSet::with_all(size),
        }
    }

    pub fn with_none(size: usize) -> CellDomain {
        CellDomain {
            rd: RangeSet::with_none(size),
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

    pub fn len(&self) -> usize {
        self.rd.size
    }

    pub fn single_value(&self) -> Option<i32> {
        self.rd.single_value().map(|n| n as i32 + 1)
    }
}
