pub mod range_set;
pub mod square;

pub use self::range_set::RangeSet;
pub use self::square::Square;

use fnv::FnvBuildHasher;
use std::ops::Index;
use linked_hash_map::LinkedHashMap;
use linked_hash_set::LinkedHashSet;

pub type FnvLinkedHashMap<K, V> = LinkedHashMap<K, V, FnvBuildHasher>;
pub type FnvLinkedHashSet<T> = LinkedHashSet<T, FnvBuildHasher>;

pub trait GetIndicesCloned<T, I, R> {
    fn get_indices_cloned(&self, indices: &[I]) -> Vec<R>;
}

impl<'a, T, I, R> GetIndicesCloned<T, I, R> for T
        where T: Index<I, Output=R>,
              I: Copy,
              R: Clone {
    fn get_indices_cloned(&self, indices: &[I]) -> Vec<R> {
        let mut vec = Vec::with_capacity(indices.len());
        for &i in indices {
            vec.push(self[i].clone());
        }
        vec
    }
}

pub trait GetIndicesRef<'a, T, I, R> {
    fn get_indices_ref(&'a self, indices: &[I]) -> Vec<&'a R>;
}

impl<'a, T, I, R> GetIndicesRef<'a, T, I, R> for T
        where T: Index<I, Output=R>,
              I: Copy {
    fn get_indices_ref(&'a self, indices: &[I]) -> Vec<&'a R> {
        let mut vec = Vec::with_capacity(indices.len());
        for &i in indices {
            vec.push(&self[i]);
        }
        vec
    }
}
