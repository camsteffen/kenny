use fnv::FnvHashSet;
use std::hash::Hash;
use std::iter::FromIterator;

/// A stack that also behaves like a set (it cannot contain duplicate values).
/// It is optimized to work with small values.
pub struct SetStack<T> {
    set: FnvHashSet<T>,
    queue: Vec<T>,
}

impl<T: Hash + Eq + Copy> SetStack<T> {

    pub fn new() -> Self {
        Self {
            set: FnvHashSet::default(),
            queue: Vec::new(),
        }
    }

    pub fn insert(&mut self, e: T) -> bool {
        if self.set.insert(e) {
            self.queue.push(e);
            true
        } else {
            false
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        while let Some(e) = self.queue.pop() {
            if self.set.remove(&e) {
                return Some(e);
            }
        }
        None
    }

    pub fn remove(&mut self, e: &T) -> bool {
        self.set.remove(e)
    }

}

impl<T: Hash + Eq + Copy> FromIterator<T> for SetStack<T> {
    fn from_iter<I: IntoIterator<Item=T>>(iter: I) -> Self {
        let mut rsq = Self::new();
        for e in iter {
            rsq.insert(e);
        }
        rsq
    }
}