use collections::range_set;
use collections::RangeSet;
use std::ops::Deref;

#[derive(Clone)]
pub struct CellDomain(RangeSet);

impl CellDomain {

    pub fn new(size: usize) -> Self {
        CellDomain(RangeSet::new(size))
    }

    pub fn with_all(size: usize) -> CellDomain {
        CellDomain(RangeSet::with_all(size))
    }

    pub fn contains(&self, n: i32) -> bool {
        self.0.contains(n as usize - 1)
    }

    pub fn insert(&mut self, n: i32) -> bool {
        self.0.insert(n as usize - 1)
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item=i32> + 'a {
        self.into_iter()
    }

    pub fn remove(&mut self, n: i32) -> bool {
        self.0.remove(n as usize - 1)
    }

    pub fn single_value(&self) -> Option<i32> {
        self.0.single_value().map(|n| n as i32 + 1)
    }
}

impl Deref for CellDomain {
    type Target = RangeSet;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Extend<i32> for CellDomain {
    fn extend<T: IntoIterator<Item=i32>>(&mut self, iter: T) {
        for i in iter {
            self.insert(i);
        }
    }
}

pub struct CellDomainIter<'a> {
    iter: range_set::Iter<'a>,
}

impl<'a> Iterator for CellDomainIter<'a> {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|i| i as i32 + 1)
    }
}

impl<'a> IntoIterator for &'a CellDomain {
    type Item = i32;
    type IntoIter = CellDomainIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        CellDomainIter {
            iter: self.0.iter(),
        }
    }
}
