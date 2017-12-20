use collections::range_set;
use collections::RangeSet;

#[derive(Clone)]
pub struct CellDomain {
    rd: RangeSet,
}

impl CellDomain {

    pub fn new(size: usize) -> CellDomain {
        CellDomain {
            rd: RangeSet::new(size),
        }
    }

    pub fn with_all(size: usize) -> CellDomain {
        CellDomain {
            rd: RangeSet::with_all(size),
        }
    }

    pub fn contains(&self, n: i32) -> bool {
        self.rd.contains(n as usize - 1)
    }

    pub fn insert(&mut self, n: i32) -> bool {
        self.rd.insert(n as usize - 1)
    }

    pub fn is_empty(&self) -> bool {
        self.rd.is_empty()
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item=i32> + 'a {
        self.rd.iter().map(|i| i as i32 + 1)
    }

    pub fn remove(&mut self, n: i32) -> bool {
        self.rd.remove(n as usize - 1)
    }

    pub fn len(&self) -> usize {
        self.rd.len()
    }

    pub fn clear(&mut self) {
        self.rd.clear();
    }

    pub fn single_value(&self) -> Option<i32> {
        self.rd.single_value().map(|n| n as i32 + 1)
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
            iter: self.rd.iter(),
        }
    }

}