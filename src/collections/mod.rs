mod range_priority_queue;
mod range_set_queue;

pub mod range_set;
pub mod square;

pub use self::square::Square;
pub use self::range_set::RangeSet;
pub use self::range_priority_queue::RangePriorityQueue;
pub use self::range_set_queue::RangeSetStack;

use std::ops::Index;

pub trait GetByIndicies<'a, T, I, R> {
    fn get_indicies(&'a self, indicies: &[I]) -> Vec<&'a R>;
}

impl<'a, T, I, R> GetByIndicies<'a, T, I, R> for T where T: Index<I, Output=R> {
    fn get_indicies(&'a self, indicies: &[I]) -> Vec<&'a R> {
        let vec = Vec::with_capacity(indicies.len());
        for &i in indicies {
            vec.push(&self[i]);
        }
        vec
    }
}