#[derive(Clone)]
pub(crate) struct RangeSet {
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
            size,
            domain: vec![true; size],
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn insert(&mut self, n: usize) -> bool {
        if self.domain[n] {
            return false;
        }
        self.domain[n] = true;
        self.size += 1;
        true
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn remove(&mut self, n: usize) -> bool {
        if !self.domain[n] {
            return false;
        }
        self.domain[n] = false;
        self.size -= 1;
        true
    }

    pub fn contains(&self, n: usize) -> bool {
        self.domain[n]
    }

    /*
    pub fn clear(&mut self) {
        for e in &mut self.domain {
            *e = false;
        }
    }
    */

    pub fn single_value(&self) -> Option<usize> {
        match self.size {
            1 => Some(self.iter().next().unwrap()),
            _ => None,
        }
    }

    pub fn iter(&self) -> Iter<'_> {
        Iter {
            domain: &self.domain,
            index: 0,
        }
    }
}

pub(crate) struct Iter<'a> {
    domain: &'a [bool],
    index: usize,
}

impl Iterator for Iter<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        for i in self.index..self.domain.len() {
            if self.domain[i] {
                self.index = i + 1;
                return Some(i);
            }
        }
        None
    }
}

impl Extend<usize> for RangeSet {
    fn extend<I: IntoIterator<Item = usize>>(&mut self, iter: I) {
        for i in iter {
            self.domain[i] = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::collections::range_set::RangeSet;

    #[test]
    #[should_panic]
    fn insert_too_high() {
        let mut set = RangeSet::new(4);
        set.insert(4);
    }

    #[test]
    fn insert_remove_result() {
        let mut set = RangeSet::new(4);
        assert!(set.insert(1));
        assert!(!set.insert(1));
        assert!(set.remove(1));
        assert!(!set.remove(1));
    }

    #[test]
    fn iter() {
        let mut set = RangeSet::new(4);
        set.insert(3);
        set.insert(1);
        let vec: Vec<_> = set.iter().collect();
        assert_eq!(vec![1_usize, 3], vec);
    }

    #[test]
    fn single_value() {
        let mut set = RangeSet::new(4);
        assert_eq!(None, set.single_value());
        set.insert(1);
        assert_eq!(Some(1), set.single_value());
        assert_eq!(Some(1), set.single_value());
        set.insert(2);
        assert_eq!(None, set.single_value());
        set.remove(1);
        assert_eq!(Some(2), set.single_value());
        set.remove(2);
        assert_eq!(None, set.single_value());
    }
}
