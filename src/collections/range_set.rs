#[derive(Clone)]
pub struct RangeSet {
    size: usize,
    domain: Vec<bool>,
}

impl RangeSet {

    pub fn new(size: usize) -> RangeSet {
        RangeSet {
            size: 0,
            domain: vec![false; size],
        }
    }

    pub fn with_all(size: usize) -> RangeSet {
        RangeSet {
            size: size,
            domain: vec![true; size],
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn insert(&mut self, n: usize) -> bool {
        let missing = !self.domain[n];
        if missing {
            self.domain[n] = true;
            self.size += 1
        }
        missing
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
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

    pub fn clear(&mut self) {
        for e in &mut self.domain {
            *e = false;
        }
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

impl Extend<usize> for RangeSet {
    fn extend<I: IntoIterator<Item=usize>>(&mut self, iter: I) {
        for i in iter {
            self.domain[i] = true;
        }
    }
}