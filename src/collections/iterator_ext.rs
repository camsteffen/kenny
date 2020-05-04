use std::cmp;

pub trait IteratorExt: Iterator {
    /// Merge another iterator and only output values that are in the original iterator and not in the other iterator.
    /// Assumes both iterators are sorted.
    /// Analogous to "LEFT JOIN" in SQL.
    fn left_merge<T>(self, other: T) -> LeftMerge<Self, T::IntoIter, Self::Item>
    where
        Self: Iterator + Sized,
        T: IntoIterator<Item = Self::Item>,
    {
        let mut right = other.into_iter();
        let r = right.next();
        LeftMerge {
            left: self,
            right,
            r_next: r,
        }
    }
}

pub struct LeftMerge<L, R, T> {
    left: L,
    right: R,
    r_next: Option<T>,
}

impl<L, R, T> Iterator for LeftMerge<L, R, T>
where
    L: Iterator<Item = T>,
    R: Iterator<Item = T>,
    T: PartialEq,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let l = match self.left.next() {
                Some(next) => next,
                None => return None,
            };
            if self.r_next.as_ref().map_or(false, |r| l == *r) {
                self.r_next = self.right.next();
            } else {
                return Some(l);
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (l_min, l_max) = self.left.size_hint();
        let (mut r_min, mut r_max) = self.right.size_hint();
        if self.r_next.is_some() {
            r_min = r_min.saturating_add(1);
            r_max = r_max.map(|v| v.saturating_add(1));
        };
        let min = r_max.map_or(0, |r_max| l_min - cmp::min(l_min, r_max));
        let max = l_max.map(|l_max| l_max - cmp::min(l_max, r_min));
        (min, max)
    }
}

impl<T: Iterator> IteratorExt for T {}

#[cfg(test)]
mod test {
    use crate::collections::iterator_ext::IteratorExt;
    use std::iter;

    #[test]
    fn test1() {
        let results: Vec<_> = vec![1, 2, 3, 4].into_iter().left_merge(vec![]).collect();
        assert_eq!(vec![1, 2, 3, 4], results);
    }

    #[test]
    fn test2() {
        let results: Vec<_> = vec![1, 2, 3, 4]
            .into_iter()
            .left_merge(vec![1, 2, 3, 4])
            .collect();
        let empty: Vec<usize> = Vec::new();
        assert_eq!(empty, results);
    }

    #[test]
    fn test3() {
        let results: Vec<_> = vec![1, 2, 3, 4]
            .into_iter()
            .left_merge(vec![1, 3])
            .collect();
        assert_eq!(vec![2, 4], results);
    }

    #[test]
    fn test4() {
        let results: Vec<_> = vec![1, 2, 3, 4]
            .into_iter()
            .left_merge(vec![1, 2])
            .collect();
        assert_eq!(vec![3, 4], results);
    }

    #[test]
    fn test5() {
        let results: Vec<_> = vec![1, 2, 3, 4]
            .into_iter()
            .left_merge(vec![3, 4])
            .collect();
        assert_eq!(vec![1, 2], results);
    }

    #[test]
    fn test6() {
        let results: Vec<_> = iter::empty().left_merge(vec![1, 2]).collect();
        let empty: Vec<usize> = vec![];
        assert_eq!(empty, results);
    }
}
