use crate::collections::range_set;
use crate::collections::RangeSet;
use std::iter::Map;

/// A small abstraction over [RangeSet] for puzzle values
#[derive(Clone)]
pub struct ValueSet(RangeSet);

impl ValueSet {
    pub fn new(max: usize) -> Self {
        ValueSet(RangeSet::new(max + 1))
    }

    pub fn with_all(max: usize) -> ValueSet {
        let mut set = RangeSet::with_all(max + 1);
        set.remove(0);
        ValueSet(set)
    }

    pub fn contains(&self, n: i32) -> bool {
        self.0.contains(n as usize)
    }

    pub fn insert(&mut self, n: i32) -> bool {
        self.0.insert(n as usize)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = i32> + '_ {
        self.into_iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn remove(&mut self, n: i32) -> bool {
        self.0.remove(n as usize)
    }

    pub fn single_value(&self) -> Option<i32> {
        self.0.single_value().map(|n| n as i32)
    }
}

impl Extend<i32> for ValueSet {
    fn extend<T: IntoIterator<Item = i32>>(&mut self, iter: T) {
        for i in iter {
            self.insert(i);
        }
    }
}

impl<'a> IntoIterator for &'a ValueSet {
    type Item = i32;
    type IntoIter = Map<range_set::Iter<'a>, fn(usize) -> i32>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().map(|n| n as i32)
    }
}
