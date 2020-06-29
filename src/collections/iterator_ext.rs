use std::cmp;
use std::iter::Peekable;

pub(crate) trait IteratorExt: Iterator + Sized {
    fn add_to<E>(self, e: &mut E)
    where
        E: Extend<Self::Item>,
    {
        e.extend(self);
    }

    fn collect_into<E>(self, mut e: E) -> E
    where
        E: Extend<Self::Item>,
    {
        e.extend(self);
        e
    }

    /// Merge another iterator and only output values that are in the original iterator
    /// and not in the other iterator.
    /// Both iterators are assumed to be sorted.
    /// Analogous to "LEFT JOIN" in SQL.
    fn left_merge<R>(self, right: R) -> LeftMerge<Self, R::IntoIter>
    where
        R: IntoIterator,

        Self::Item: PartialOrd<R::Item>,
        R::Item: PartialOrd<Self::Item>,
    {
        let left = self;
        let right = right.into_iter().peekable();
        LeftMerge { left, right }
    }

    fn right_merge<R>(self, right: R) -> LeftMerge<R::IntoIter, Self>
    where
        R: IntoIterator,
        Self::Item: PartialOrd<R::Item>,
        R::Item: PartialOrd<Self::Item>,
    {
        right.into_iter().left_merge(self)
    }
}

pub(crate) struct LeftMerge<L, R>
where
    L: Iterator,
    R: Iterator,
    L::Item: PartialOrd<R::Item>,
{
    left: L,
    right: Peekable<R>,
}

impl<L, R> Iterator for LeftMerge<L, R>
where
    L: Iterator,
    R: Iterator,
    L::Item: PartialOrd<R::Item>,
{
    type Item = L::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let mut l = self.left.next()?;
        loop {
            match self.right.peek() {
                Some(r) if l >= *r => {
                    if l == *r {
                        l = self.left.next()?;
                    }
                    self.right.next();
                }
                _ => return Some(l),
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (l_min, l_max) = self.left.size_hint();
        let (r_min, r_max) = self.right.size_hint();
        let min = r_max.map_or(0, |r_max| l_min - cmp::min(l_min, r_max));
        let max = l_max.map(|l_max| l_max - cmp::min(l_max, r_min));
        (min, max)
    }
}

impl<T: Iterator> IteratorExt for T {}

#[cfg(test)]
mod test {
    use crate::collections::iterator_ext::IteratorExt;
    use itertools::assert_equal;
    use std::fmt::Debug;
    use std::iter;

    #[test]
    fn test() {
        assert_left_merge(&[1, 1, 2, 3], &[1, 2], &[1, 3]);
        assert_left_merge(&[1, 1, 2, 3], &[1, 2], &[1, 3]);
        assert_left_merge(&[1, 2, 3, 4], &[-1, 0], &[1, 2, 3, 4]);
        assert_left_merge(&[1, 2, 3, 4], &[-1, 0, 1], &[2, 3, 4]);
        assert_left_merge(&[1, 2, 3, 4], &[0, 1], &[2, 3, 4]);
        assert_left_merge(&[1, 2, 3, 4], &[], &[1, 2, 3, 4]);
        assert_left_merge(&[1, 2, 3, 4], &[1, 2, 3, 4], &[]);
        assert_left_merge(&[1, 2, 3, 4], &[1, 3], &[2, 4]);
        assert_left_merge(&[1, 2, 3, 4], &[1, 2], &[3, 4]);
        assert_left_merge(&[1, 2, 3, 4], &[3, 4], &[1, 2]);
        assert_left_merge(iter::empty::<&i32>(), &[], &[]);
        assert_left_merge(&[], &[1, 2], &[]);
    }

    fn assert_left_merge<L, R, E, T>(left: L, right: R, expected: E)
    where
        L: IntoIterator<Item = T>,
        R: IntoIterator<Item = T>,
        E: IntoIterator<Item = T>,
        L::IntoIter: Clone,
        R::IntoIter: Clone,
        E::IntoIter: Clone,
        T: Debug + PartialOrd,
    {
        let (left, right, expected) = (left.into_iter(), right.into_iter(), expected.into_iter());
        assert_equal(left.clone().left_merge(right.clone()), expected.clone());
        assert_equal(right.right_merge(left), expected);
    }
}
