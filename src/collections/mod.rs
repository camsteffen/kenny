pub mod range_set;
pub mod square;

pub use self::range_set::RangeSet;
pub use self::square::Square;

use fnv::FnvBuildHasher;
use std::ops::Index;
use linked_hash_set::LinkedHashSet;

pub type FnvLinkedHashSet<T> = LinkedHashSet<T, FnvBuildHasher>;

pub fn iter_indices<T, I, J, O>(data: T, indices: I) -> impl Iterator<Item=O>
    where I: IntoIterator<Item=J>,
          T: Index<J, Output=O>,
          O: Sized + Copy {
    indices.into_iter().map(move |i| data[i])
}

pub trait CloneIndices<T, I, R> {
    fn clone_indices(&self, indices: &[I]) -> Vec<R>;
}

impl<'a, T, I, R> CloneIndices<T, I, R> for T
        where T: Index<I, Output=R>,
              I: Copy,
              R: Clone {
    fn clone_indices(&self, indices: &[I]) -> Vec<R> {
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
