pub mod range_set;
pub mod square;

pub use self::range_set::RangeSet;
pub use self::square::Square;

use fnv::FnvBuildHasher;
use std::ops::Index;
use linked_hash_set::LinkedHashSet;

pub type FnvLinkedHashSet<T> = LinkedHashSet<T, FnvBuildHasher>;

pub trait GetIndiciesCloned<T, I, R> {
    fn get_indices_cloned(&self, indicies: &[I]) -> Vec<R>;
}

impl<'a, T, I, R> GetIndiciesCloned<T, I, R> for T
        where T: Index<I, Output=R>,
              I: Copy,
              R: Clone {
    fn get_indices_cloned(&self, indicies: &[I]) -> Vec<R> {
        let mut vec = Vec::with_capacity(indicies.len());
        for &i in indicies {
            vec.push(self[i].clone());
        }
        vec
    }
}

pub trait GetIndiciesRef<'a, T, I, R> {
    fn get_indices_ref(&'a self, indicies: &[I]) -> Vec<&'a R>;
}

impl<'a, T, I, R> GetIndiciesRef<'a, T, I, R> for T
        where T: Index<I, Output=R>,
              I: Copy {
    fn get_indices_ref(&'a self, indicies: &[I]) -> Vec<&'a R> {
        let mut vec = Vec::with_capacity(indicies.len());
        for &i in indicies {
            vec.push(&self[i]);
        }
        vec
    }
}
