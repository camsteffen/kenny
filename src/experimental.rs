use std::borrow::BorrowMut;
use std::ptr;

trait CollectVecs<T, U> {
    fn collect_vecs2(self) -> (Vec<T>, Vec<U>);
}

impl<I, T, U> CollectVecs<T, U> for I
where
    I: Iterator<Item = (T, U)>,
{
    fn collect_vecs2(self) -> (Vec<T>, Vec<U>) {
        let capacity = self.size_hint().0;
        self.fold(
            (Vec::with_capacity(capacity), Vec::with_capacity(capacity)),
            |(mut vec_a, mut vec_b), (a, b)| {
                vec_a.push(a);
                vec_b.push(b);
                (vec_a, vec_b)
            },
        )
    }
}

fn retain_map_indexed<'a, T, U, F>(vec: &'a mut Vec<T>, f: F) -> impl Iterator<Item = U> + 'a
where
    F: 'a + FnMut(usize, &T) -> Option<U>,
{
    RetainMapIndexed {
        vec,
        f,
        index: 0,
        del: 0,
    }
}

struct RetainMapIndexed<'a, T, F> {
    vec: &'a mut Vec<T>,
    f: F,
    index: usize,
    del: usize,
}

impl<'a, T, U, F> Iterator for RetainMapIndexed<'a, T, F>
where
    F: FnMut(usize, &T) -> Option<U>,
{
    type Item = U;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.vec.len() {
            let option = (self.f)(self.index, &self.vec[self.index]);
            if option.is_none() {
                self.del += 1;
            } else if self.del > 0 {
                self.vec.swap(self.index - self.del, self.index);
            }
            self.index += 1;
            if option.is_some() {
                return option;
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.vec.len() - self.index))
    }
}

impl<'a, T, F> Drop for RetainMapIndexed<'a, T, F> {
    fn drop(&mut self) {
        if self.del > 0 {
            self.vec.truncate(self.vec.len() - self.del);
        }
    }
}

#[test]
fn test() {
    let mut vec = vec![1, 2, 3, 4];
    let mut iter = removable_iter(&mut vec);
    while let Some(item) = iter.next() {
        if item.item() % 2 != 0 {
            item.remove();
        }
    }
    drop(iter);
    assert_eq!(vec, vec![2, 4]);
}

fn removable_iter<T>(vec: &mut Vec<T>) -> RemovableIter<'_, T> {
    RemovableIter {
        vec,
        index: 0,
        del: 0,
    }
}

struct RemovableIter<'a, T> {
    vec: &'a mut Vec<T>,
    index: usize,
    del: usize,
}

impl<'a, T> RemovableIter<'a, T> {
    fn next<'b>(&'b mut self) -> Option<Removable<'a, 'b, T>> {
        if self.index == self.vec.len() {
            return None;
        }
        let item = Removable {
            iter: self,
            removed: false,
        };
        Some(item)
    }

    fn advance(&mut self, remove: bool) {
        if remove {
            self.del += 1;
        } else if self.del > 0 {
            unsafe {
                ptr::copy_nonoverlapping(
                    self.vec.get_unchecked_mut(self.index),
                    self.vec.get_unchecked_mut(self.index - self.del),
                    1,
                );
            }
        }
        self.index += 1;
    }
}

impl<'a, T> Drop for RemovableIter<'a, T> {
    fn drop(&mut self) {
        if self.del > 0 {
            unsafe {
                if self.index < self.vec.len() {
                    ptr::copy(
                        self.vec.get_unchecked(self.index),
                        self.vec.get_unchecked_mut(self.index - self.del),
                        self.vec.len() - self.index,
                    );
                }
                self.vec.set_len(self.vec.len() - self.del)
            }
        }
    }
}

struct Removable<'a, 'b, T> {
    iter: &'b mut RemovableIter<'a, T>,
    removed: bool,
}

impl<'a, 'b, T> Removable<'a, 'b, T> {
    fn item(&self) -> &T {
        &self.iter.vec[self.iter.index]
    }

    fn item_mut(&mut self) -> &mut T {
        &mut self.iter.vec[self.iter.index]
    }

    fn remove(mut self) -> T {
        self.removed = true;
        unsafe { ptr::read(self.iter.vec.get_unchecked(self.iter.index)) }
    }
}

impl<'a, 'b, T> Drop for Removable<'a, 'b, T> {
    fn drop(&mut self) {
        self.iter.advance(self.removed);
    }
}

trait IteratorExt<T, U>
where
    T: Iterator,
{
    fn add_to(self, u: &mut U)
    where
        U: Extend<T::Item>;

    fn collect_into(self, u: U) -> U
    where
        U: Extend<T::Item>;
}

impl<T, U> IteratorExt<T, U> for T
where
    T: Iterator,
    U: Extend<T::Item>,
{
    fn add_to(self, u: &mut U)
    where
        U: Extend<T::Item>,
    {
        u.extend(self);
    }

    fn collect_into(self, mut u: U) -> U
    where
        U: Extend<T::Item>,
    {
        u.extend(self);
        u
    }
}

fn test_collect_into() {
    let v: Vec<usize> = vec![1, 2, 3]
        .into_iter()
        .collect_into(Vec::with_capacity(10));
}
